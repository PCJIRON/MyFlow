# Chapter 2: Pill HUD — footprint, docking, hover, reveal

*Volume 2. Source: `src-tauri/src/main.rs` (geometry block) and `ui/pill.html`.*

*The pill is a HUD, not a window: it hugs one screen edge, its window is never larger than the bar it paints, and it must survive drags, Win+D, and a caption double-click.*

## Section 2.1: Geometry — dock, footprint, work area, anchor

#### Paragraph: current_dock

`PILL_DOCK` is an `AtomicU8`: 0 bottom, 1 left, 2 right.
`Read it and map to Dock.` Anything else is `Dock::Bottom`.

#### Paragraph: pill_footprint

Given `dock` and `state` ("idle" | "open" | "rec"), return `(width, height)` in logical pixels:

```
bottom: idle 54x10 · open 402x56 · rec 130x32
left/right: idle 10x54 · open 56x402 · rec 32x130
```

*The bar paints 52x8 / 400x54 / 128x30; the extra 2px per axis is the 1px padding the page keeps inside the window.*

#### Paragraph: work_area_logical

`**scale** is monitor.scale_factor().`
`**area** is monitor.work_area().`
Return `(pos.x, pos.y, size.width, size.height)` after `to_logical::<i32>(scale)` on both position and size.

*Tauri reports the work area in physical pixels but sizes and positions windows in logical units. Mixing them on a 1.5x display docked the pill at logical y=932 — physical y=1398 on a 1080-tall screen. The window existed, was visible, was topmost, and was simply below the desktop.*

#### Paragraph: anchor_for

Given `dock`, `area` and a `w`x`h` window, return the top-left corner:

```
bottom → (ax + (aw - w) / 2, ay + ah - h - PILL_MARGIN)
left   → (ax + PILL_MARGIN, ay + (ah - h) / 2)
right  → (ax + aw - w - PILL_MARGIN, ay + (ah - h) / 2)
```

*The work area already excludes the taskbar, so the bottom edge stays `PILL_MARGIN` above it and the window grows upward — expanding never covers the taskbar.*

## Section 2.2: Sizing — fit the window to the bar

#### Paragraph: fit_pill

`**monitor** is win.current_monitor() or, failing that, win.primary_monitor().` No monitor → return.
`**area** is work_area_logical(&monitor).`
`**x, y** is anchor_for(current_dock(), area, w, h).`
`set_size(LogicalSize(w, h))` then `set_position(LogicalPosition(x, y))`.

#### Paragraph: pill_fit (command)

Called by the page whenever the bar changes shape.
`**state** is the argument.` `**w, h** is pill_footprint(current_dock(), &state).` `fit_pill(&win, w, h).`

*This is the whole "the window is only as big as the button" rule: the page owns the shape, Rust owns the pixels.*

#### Paragraph: pill_size

Return `pill_footprint(current_dock(), "idle")` as `(f64, f64)`. *Used by the reveal nudge.*

## Section 2.3: Docking — drag, snap, re-anchor

#### Paragraph: dock_pill_to

Given `dock`:
1. find the monitor (current, else primary), else return,
2. `**area** is work_area_logical(&monitor)`,
3. `**vertical** is dock != Bottom`,
4. store `PILL_DOCK` (0/1/2) and `PILL_VERT`,
5. `unmaximize()` — *a caption double-click on a transparent always-on-top window can leave it maximised across the desktop: invisible, and it swallows every click*,
6. `set_always_on_top(true)`,
7. `**w, h** is pill_footprint(dock, "idle")`, then `fit_pill`,
8. emit `pill-orient` with "v" or "h",
9. log `pill docked {dock:?} at {x},{y} (logical) size={w}x{h} scale=… area=…`.

#### Paragraph: harden_pill_window

Windows only. Read `GWL_STYLE`, clear `WS_MAXIMIZEBOX | WS_MINIMIZEBOX | WS_THICKFRAME`, and if anything changed write it back and call `SetWindowPos` with `SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED`.

*Stripping the box styles makes maximise and minimise refuse outright, so the HUD cannot be blown up by a stray double-click.*

#### Paragraph: place_pill

`dock_pill_to(app, Dock::Bottom).` *Every launch returns the pill to the bottom strip, even if it was docked sideways last session.*

#### Paragraph: spawn_dock_watcher

Loop every 300ms:
1. get the pill window, else return,
2. `is_maximized?` → unmaximize, `dock_pill_to(Bottom)`, reset the still-timer, continue,
3. `**pos** is outer_position()`; if it changed since the last tick, remember it, reset the still-timer, continue,
4. if the still-timer is under 700ms, continue — *the drag has not clearly ended*,
5. find the monitor, `**scale**`, `**area**`, `**logical** is pos.to_logical(scale)`, `**width, height** is outer_size().to_logical(scale)`,
6. `**cx** is the cursor x in logical pixels` — `GetCursorPos`, then `p.x / scale`; if the call fails, fall back to `logical.x + width / 2`,
7. `**fifth** is area.2 / 5`; `cx < area.0 + fifth` → Left; `cx > area.0 + area.2 - fifth` → Right; otherwise Bottom,
8. `**tx, ty** is anchor_for(dock, area, width, height)`; if the dock changed or the window is more than 4px from its anchor, `dock_pill_to(dock)` and reset the still-timer.

*The dock is decided by where the user dropped it — the cursor — not by the window centre. A grabbed window keeps its grab offset, so a pill dragged to the bottom edge could still measure as "left" and snap sideways.*

*The watcher never resets the window size: size belongs to the page (idle / hover / recording).*

## Section 2.4: Hover — cursor polling instead of DOM events

#### Paragraph: spawn_hover_watcher

Loop every 90ms:
1. get the pill window, else return,
2. `PILL_SHOWN` false → continue,
3. `GetWindowRect(hwnd)` and `GetCursorPos`; either failing → continue,
4. `**inside** is the cursor within the rect`,
5. if `inside` differs from the last value, remember it and emit `pill-hover` with the boolean.

*The window is resized under the cursor, which makes the web view fire a spurious `mouseleave`; the page then fought itself — expand, collapse, expand. Polling the real rectangle is stable.*

## Section 2.5: Reveal — paint before show

#### Paragraph: reveal_pill

`PILL_SHOWN.swap(true)` already true → return.
No pill window → set `PILL_SHOWN` back to false and return.
Otherwise: `unminimize`, `set_background_color(Some(Color(0, 0, 0, 0)))`, `set_always_on_top(true)`, `show()`, log `pill revealed ({why})`.
Then on a thread, for delays `[90, 700]`: sleep, `set_size(pw, ph + 1)`, sleep 90ms, `set_size(pw, ph)`, `set_always_on_top(true)`.

*WebView2 can hand DWM an empty frame on a transparent always-on-top window shown before the page composited: the window is there, the bar is not, and the first resize suddenly paints it. Without the transparent background colour the surface paints opaque black instead of letting the desktop through.*

#### Paragraph: pill_ready (command)

`reveal_pill(&app, "page ready").` *The page calls this after two animation frames.*

## Section 2.6: Guard — keep the overlay on screen

#### Paragraph: spawn_pill_guard

Loop every 2500ms:
1. get the pill window, else return,
2. `PILL_SHOWN` false → continue,
3. `is_minimized?` → unminimize, show, focus, log "pill guard: restored from minimized",
4. `!is_visible?` → show, log "pill guard: restored from hidden",
5. `set_always_on_top(true)`.

*Win+D, a snap, or focus juggling can minimise or un-topmost an overlay. The guard only restores visibility and stacking — it never moves the window, so a dragged position survives.*

## Section 2.7: Page — `ui/pill.html`, the bar itself

#### Paragraph: pill markup

`#wrap` is fixed with 1px inset on all sides and centres its child.
`#pill` holds: `#mic` (knob, mic icon), `#wave` (16 bars, built by script), `#body` (the drag region), `#notes-btn`, `#cog`.
`#err` is a fixed box centred in the window, hidden until `.show`.

*`#body` must be a real box with `flex: 1 1 auto; min-width: 24px` — a zero-width element cannot be grabbed, and the old selector matched an attribute that never existed, so the whole pill had no draggable surface.*

#### Paragraph: pill states

`#pill` at rest: 52x8, `#000000`, 1px white-alpha border, radius 999px, no shadow.
`#pill.open`: 100% x 54, `rgba(13,13,16,.97)`, `blur(22px)`, knobs visible.
`#pill.isrec`: 128x30, `#0a0a0c`, mic and body hidden, waveform shown.
`body.vert` swaps every pair: 8x52, 54x212, 30x128, and the waveform animates width instead of height.

#### Paragraph: fit and refit

`**recordingUI** is a page flag.`
`fit(state)` invokes `pill_fit` with the state, swallowing errors.
`refit()` picks "rec" when recording, else "open" when `#pill` has `.open`, else "idle".

#### Paragraph: applyOrient

Toggle `body.vert` for "v", set the window title to "VERT-OK" / "HORZ-OK" through `debug_title`, then `refit()`.
On boot: `applyOrient(await invoke("pill_orient"))`, and listen for `pill-orient`.

#### Paragraph: boot sequence

1. install `window.onerror` and `unhandledrejection` handlers that log through `sr_error`,
2. build 16 waveform bars,
3. after two animation frames: `pill_ready`, again after 400ms, then `refit()`,
4. load settings, log the language, keep a `refreshSettings()` that re-reads them before every session.

*Settings are cached at boot, so a language changed in the Settings window would otherwise be ignored until restart.*

#### Paragraph: setRec

`**on** is the new recording flag.`
`If on? remove .open and fit("rec").`
`Toggle .isrec.`
`If not on? clear every inline bar height, then after 320ms, if the pill is not open and not recording, fit("idle").`

*The 320ms wait lets the capsule finish shrinking inside the window before the window itself gets small.*

#### Paragraph: hover listener

`pill-hover` true → if recording, ignore; else `fit("open")` and add `.open`.
`pill-hover` false → remove `.open`; if recording, ignore; else after 320ms, if still not open, `fit("idle")`.

#### Paragraph: error box

`err(msg)` sets the text, adds `.show`, calls `fit("open")` — *the message needs the wide footprint or it is clipped* — and after 4s hides it and refits if the pill is not open.
`stt-error` is ignored when the payload is empty or starts with "sr:". *Engine diagnostics belong in the log, not on screen.*

#### Paragraph: buttons

`#mic` → `toggle_record`. `#cog` → `show_settings`. `#notes-btn` → `show_notes`.

`#### Proofread: footprint`
Hover the pill: the window grows to 402x56 and its bottom edge stays 3px above the taskbar. Leave: it returns to 54x10. Drag it to the left edge and drop: it becomes 10x54 at x = work area x + 2.
