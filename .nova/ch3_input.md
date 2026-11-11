# Chapter 3: Input — hotkeys, CapsLock, caret injection

*Volume 3. Source: `src-tauri/src/main.rs` (`parse_shortcut`, `register_hotkey`, `handle_smart`, `arm_hold_start`, `spawn_capslock_ptt`), `src-tauri/src/inject.rs`, `src-tauri/src/autostart.rs`.*

## Section 3.1: Hotkeys — register and re-register

#### Paragraph: parse_shortcut

`**shortcut** is s.parse::<Shortcut>()`, mapped to `Invalid hotkey '{}': {}` on failure.

#### Paragraph: register_hotkey

1. borrow `State`,
2. `registered == Some(hotkey)?` → return Ok, *nothing to do*,
3. parse the new shortcut,
4. `matches!(sh.key, Code::CapsLock)?` → unregister the old hotkey if any, store the new one and return Ok — *CapsLock is owned by the keyboard hook (CITES Section 3.3 from ch3_input). `RegisterHotKey` takes the key away from Windows, so registering it is what stopped CapsLock toggling while the app ran, and it made the hook's replayed toggle arrive as another hotkey press.*
5. if an old one is registered, unregister it (ignoring errors),
6. `gs.register(sh)`, map the error to a string,
7. store the new hotkey.

#### Paragraph: hotkey handler

The global-shortcut plugin calls this on every event:
`**cfg** is settings::load(app).`
`**st** is app.state().`
`cfg.mode == "smart"?` → `handle_smart(app, event.state)`.
`Otherwise? cfg.mode == "toggle"?` → on Pressed, `st.rec.toggle(app, cfg)`.
`Otherwise (hold mode)?`
  Pressed → if not running, `st.rec.start(...)` and emit `stt-error` on failure; else log "hotkey press ignored (already running)".
  Released → if running, log "hotkey release -> stop" and `st.rec.stop(app)`; else log "hotkey release ignored (not running)".

## Section 3.2: Smart mode — hold, tap, double-tap

#### Paragraph: arm_hold_start

Given the press `token`, spawn a thread: sleep `SMART_HOLD_MS`, then
`**still_held** is press_start == Some(token).`
`still_held && !toggled?` → load settings and `st.rec.start(...)`, emitting `stt-error` on failure.

*The press is not treated as dictation until it survives the hold window, so ordinary Space taps never wake the microphone.*

#### Paragraph: handle_smart — pressed

`**now** is Instant::now().`
`**double** is false.`
Inside `last_press`: if the previous press is within `SMART_DOUBLE_MS`, set double and clear `last_press`; else store `now`. If there was no previous press, store `now`.
`double?`
  clear `press_start`,
  `toggled?` → `stop`, clear `toggled`, set `skip_forward` (*the release that ends a double-tap must not type a space*);
  `Otherwise?` → `start`; on Ok set `toggled`, on Err emit `stt-error`.
`Otherwise, !toggled?`
  set `press_start` to `now` and `arm_hold_start(app, now)`.

#### Paragraph: handle_smart — released

`**held** is press_start.take().`
`skip_forward?` → swap to false and return.
`toggled?` → return — *a toggle session keeps recording regardless of the release*.
`held is Some(t) and now - t >= SMART_HOLD_MS?` → `st.rec.stop(app)` (*the pipeline pastes the final text*).
`Otherwise?` → `inject::press_space()` — *a quick tap is forwarded as a real Space so typing keeps working*.

## Section 3.3: CapsLock — push-to-talk without breaking CapsLock

*CapsLock doubles as a dictation key while the app runs: the hook owns the key, a quick tap replays the toggle Windows would have made, and a hold never touches the state at all.*

#### Paragraph: hook

`code < 0` → pass through.
`**kb** is the KBDLLHOOKSTRUCT.`
`kb.vk == VK_CAPITAL and !INJECTING?` → `TX.send(kb.flags & LLKHF_UP != 0)` (*true = released*); the send succeeded → return 1 (*swallow*).
Otherwise `CallNextHookEx`.

*Swallow only while the channel accepts the event: a dead worker makes the send fail, and then the hook falls out of the way so CapsLock is native again instead of dead.*

*The hook performs no file IO, no recording call and no key synthesis. Anything slower delays its return and Windows starts dropping key events — that is what made the hook look dead.*

#### Paragraph: worker

Create a channel, store the sender in `TX`.
`**CAPS** is a Mutex around `Caps { held: bool, dictating: bool, press_at: Option<Instant> }`.
Thread A — the worker — loops over received `is_up` values:
`!is_up?`
  lock; `held and not stale` (*`press_at` older than `CAPS_STALE_MS`*) → continue (*key repeat*).
  `held = true`; `dictating = false`; `press_at = now`; unlock.
  Spawn a timer thread: sleep `CAPS_HOLD_MS`; lock; `held and !dictating?` → `dictating = true`, unlock, `st.rec.start(...)`, emitting `stt-error` on failure.
`is_up?`
  lock; `!held?` → continue. Otherwise `held = false`, `press_at = None`, take `dictating`.
  `dictating?` → `st.rec.stop(&worker)`.
  `Otherwise?` → `replay_toggle(&worker)`.

Thread B — `SetWindowsHookExW(WH_KEYBOARD_LL, hook, null, 0)` and a `GetMessageW` loop. *A low-level hook needs a message loop in the thread that installed it.*

#### Paragraph: replay_toggle

`INJECTING = true`.
`**before**` is `GetAsyncKeyState(VK_CAPITAL) & 1`.
Repeat twice, stopping early: `keybd_event(VK_CAPITAL, 0, 0, 0)`, `keybd_event(VK_CAPITAL, 0, KEYEVENTF_KEYUP, 0)`, sleep 40ms, `after` is the state; `after != before?` → stop.
`INJECTING = false`; log `caps tap -> state {} (was {})`.

*Untagged and sent after release: an injected toggle is ignored while the key is still held, and a toggle carrying a non-zero dwExtraInfo is ignored outright. Read back because a dropped injection used to leave the key silently dead.*

#### Paragraph: double tap

`**double** is: `last_tap` is set and younger than `SMART_DOUBLE_MS`.`
`last_tap` is cleared when it was a double, otherwise set to now.`
`double?` → `st.rec.toggle(&worker, settings.load(&worker))`.

*The same gesture smart mode gives the hotkey, kept here because CapsLock is never registered as one (CITES Section 3.1 from ch3_input). Both taps already replayed their toggle, so CapsLock ends where it started and only the session flips.*

#### Paragraph: why the key is swallowed

Windows flips CapsLock on key-down. A "toggle it back on release" design has to guess at key-down whether the press will be a tap or a hold; every press longer than `CAPS_HOLD_MS` was treated as a hold and toggled twice — net zero — so CapsLock looked dead for anyone who pressed it slowly. Owning the key removes the guess: a hold never changes the state, a tap changes it once, on release.

*One lock instead of two atomics: a release could land between the timer's check and its store, leaving `DICTATING` set with the key already up — a recording that never stops and a toggle on the wrong press.*

#### Paragraph: why INJECTING exists

The replayed tap re-enters this very hook. `INJECTING` marks that window so the hook lets its own event through instead of swallowing it as a second press — and it is a flag, not `dwExtraInfo`, because a tagged CapsLock injection never toggles.

`#### Proofread: caps`
Start with CapsLock off. Tap: on, and typing comes out uppercase. Tap: off, typing lowercase. Hold 600ms with CapsLock off: still off, typing lowercase, and the log shows `rec:start` then `rec:stop`. Hold with CapsLock on: still on, typing uppercase. Double tap: recording starts, typing case unchanged; double tap again: recording stops.

## Section 3.4: Injection — paste or type at the caret

#### Paragraph: inject_text

Empty text → return.
`method == "type"?` → `type_text`. `Otherwise?` → `paste_text`.

#### Paragraph: paste_text

Sleep 120ms — *the focused app needs a beat after the hotkey was released*.
Set the clipboard with arboard; on success `ctrl_v()`, on failure `type_text(text)` (*clipboard blocked → fall back*).

#### Paragraph: ctrl_v

New `Enigo`; press Control, click Unicode 'v', release Control.

#### Paragraph: type_text

New `Enigo`; `enigo.text(text)`.

#### Paragraph: press_space

New `Enigo`; click Unicode ' '. *Used by smart mode so a quick tap still types a space.*

## Section 3.5: Autostart — start with Windows

#### Paragraph: reg

Windows only. Run `reg` with `CREATE_NO_WINDOW`, the given args, and `output()`; a failure is `None`.

#### Paragraph: apply

`enabled?` → `reg(["add", RUN_KEY, "/v", RUN_VALUE, "/t", "REG_SZ", "/d", "\"{exe}\"", "/f"])`.
`Otherwise?` → `reg(["delete", RUN_KEY, "/v", RUN_VALUE, "/f"])`.

*Per-user Run key: no elevation, no extra crate.*

`#### Proofread: autostart`
Toggle "Start with Windows" in Settings; `reg query HKCU\...\Run /v MyFlow` shows the quoted exe path. Toggle it off; the value is gone.
