# Chapter 1: App shell — boot, windows, tray, exit

*Volume 1. Source: `src-tauri/src/main.rs` (top, `main()`, `show_window`, `toggle_notes`, `setup_tray`).*

## Section 1.1: State — shared application state

`State` is created on boot and borrowed by every command and every handler.

#### Paragraph: State definition

`#[derive(Default)]`.
`STRUCT State` with `FIELD: **rec** is Arc<Rec>` (CITES Section 4.1 from ch4_capture), `FIELD: **registered** is Mutex<Option<String>>`, `FIELD: **last_press** is Mutex<Option<Instant>>`, `FIELD: **press_start** is Mutex<Option<Instant>>`, `FIELD: **toggled** is AtomicBool`, `FIELD: **skip_forward** is AtomicBool`.

*Default is enough: every field except `rec` starts empty.*

## Section 1.2: Boot — environment, plugin, manage, setup

#### Paragraph: boot

`main()` runs `std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,msSpeechRecognitionServiceUseCetoService")` before the builder. *WebView2 153+ answers 403 on its own speech service; disabling that feature puts Web Speech back on the working path. The env var is the only reliable way to pass browser arguments — window-level argument strings broke `CreateCoreWebView2Environment` on some machines.*

Then build `tauri::Builder`:
1. register `tauri_plugin_global_shortcut` with a handler (CITES Section 3.1 from ch3_input),
2. `manage(State::default())`,
3. `on_window_event` — for windows "settings" and "notes", a `CloseRequested` becomes `api.prevent_close()` + `window.hide()`,
4. `invoke_handler` with the command list in the glossary,
5. `setup` (next paragraph),
6. `run(generate_context!())`.

#### Paragraph: setup

`**handle** is app.handle().clone().` Load settings (CITES Section 6.2 from ch6_settings).
`Register the hotkey? If it fails? eprintln "hotkey register failed".`
`Set up the tray? If it fails? eprintln "tray setup failed".`
`Is the pill window present? Then: set_maximizable(false), set_minimizable(false), harden_pill_window(&w).` (CITES Section 2.3 from ch2_pill)
`place_pill(&handle).` (CITES Section 2.3 from ch2_pill) And in order: `spawn_pill_guard`, `spawn_dock_watcher`, `#[cfg(windows)] spawn_capslock_ptt`, `#[cfg(windows)] spawn_hover_watcher`.
Finally spawn a fallback thread: sleep 4000ms, and `if !PILL_SHOWN` then `reveal_pill(&fallback, "fallback timer")`. *A page that never calls back must not leave the pill invisible forever.*

## Section 1.3: Windows — show, focus, hide-on-close

#### Paragraph: show_window

Given `label`, find the webview window.
`Unminimize and show.`
`**was_topmost** is the previous state — not tracked; the window is simply forced topmost, focused, and 700ms later set back to normal.` *Windows refuses focus to a background process, so the window used to open behind everything and users had to alt-tab to it.*
`Run the 700ms reset on its own thread.`

#### Paragraph: toggle_notes

`Is the notes window visible? Then hide it. Otherwise? show it and give it focus.`

*Tray, pill button and command all call this, so one toggle rule exists.*

## Section 1.4: Tray — the only way to quit a taskbar-less app

#### Paragraph: setup_tray

Build three menu items on the app: "settings" → "Open Settings", "notes" → "Open Notes", "quit" → "Quit MyFlow".
`**icon** is tauri::image::Image::new_owned(TRAY_RGBA.to_vec(), TRAY_SIZE, TRAY_SIZE).`
`TrayIconBuilder::with_id("main")` with that icon, tooltip "MyFlow — hold your dictation key to talk", the menu, `show_menu_on_left_click(true)`, and an event handler:
`"settings" → show_window(app, "settings").`
`"notes" → toggle_notes(app).`
`"quit" → app.exit(0).`

*The pill is `skipTaskbar`, so without this tray item the process could never be closed.*

`#### Proofread: tray`
Temporarily replace TRAY_RGBA with an all-zero 32x32 buffer: the tray icon builds and the menu still opens. No panic is acceptable in the builder.
