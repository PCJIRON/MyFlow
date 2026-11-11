# MyFlow — Table of Contents

## PREFACE

THIS BOOK SPEAKS "Rust 2021" WITH "Tauri v2 + WebView2".

  Language:    Rust 2021, Windows target, `windows_subsystem = "windows"` in release
  Shell:       Tauri v2 (`tray-icon` feature), single binary, `bundle.active = false`
  Frontend:    Plain HTML/CSS/JS in `ui/`, served as `frontendDist`, `withGlobalTauri = true`
  Storage:     one JSON file — `app_config_dir()/settings.json`
  Network:     reqwest 0.12 + rustls → Groq OpenAI-compatible API
  Clipboard:   arboard 3 (read/write) and enigo 0.6 (synthesised keys)
  OS APIs:     Win32 through raw FFI (user32) — Windows-only code behind `#[cfg(windows)]`
  Logging:     append-only text log — `app_data_dir()/sr.log`
  Platform:    Windows 10/11 desktop, per-user, never elevated

Home directories:

```
  src-tauri/src/main.rs        the shell: windows, tray, pill geometry, hotkeys
  src-tauri/src/rec.rs         recording state + transcript delivery
  src-tauri/src/ai.rs          Groq: transcription, model list, style pass
  src-tauri/src/settings.rs    settings model + persistence
  src-tauri/src/inject.rs      put text at the caret
  src-tauri/src/autostart.rs   HKCU Run key
  ui/pill.html                 the floating dictation bar
  ui/settings.html             the settings window
  ui/notes.html                the notes composer
```

---

## VOLUME 1: DESKTOP SHELL

Lifecycle, windows, tray, and the rules that keep the app alive on a desktop.

- `ch1_shell.md` — Ch 1: App shell — boot, window creation, tray, exit
  - 1.1 State — shared application state
  - 1.2 Boot — environment, plugin, manage, setup
  - 1.3 Windows — show, focus, hide-on-close
  - 1.4 Tray — the only way to quit a taskbar-less app

## VOLUME 2: THE PILL

The floating dictation bar: what shape it takes, where it sticks, how big its window is.

- `ch2_pill.md` — Ch 2: Pill HUD — footprint, docking, hover, reveal
  - 2.1 Geometry — dock, footprint, work area, anchor
  - 2.2 Sizing — fit the window to the bar
  - 2.3 Docking — drag, snap, re-anchor
  - 2.4 Hover — cursor polling instead of DOM events
  - 2.5 Reveal — paint before show
  - 2.6 Guard — keep the overlay on screen
  - 2.7 Page — `ui/pill.html`, the bar itself

## VOLUME 3: INPUT

How a key press becomes a recording session, and how text comes back out.

- `ch3_input.md` — Ch 3: Input — hotkeys, CapsLock, caret injection
  - 3.1 Hotkeys — register and re-register
  - 3.2 Smart mode — hold, tap, double-tap
  - 3.3 CapsLock — push-to-talk without breaking CapsLock
  - 3.4 Injection — paste or type at the caret
  - 3.5 Autostart — start with Windows

## VOLUME 4: CAPTURE

Recording state and the delivery route of every transcript.

- `ch4_capture.md` — Ch 4: Capture — start, stop, accumulate, deliver
  - 4.1 Rec — the state object
  - 4.2 Start — one running session
  - 4.3 Stop — flush the tail, then deliver
  - 4.4 Delivery — notes window or caret

## VOLUME 5: TRANSCRIPTION

Groq: audio in, text out, plus the optional writing-style pass.

- `ch5_groq.md` — Ch 5: Groq — transcribe, models, style
  - 5.1 Client — the shared upload path
  - 5.2 Transcribe — clip to text, with model fallback
  - 5.3 Models — the live list from the user's account
  - 5.4 Prompt — the dictation cleaner instructions
  - 5.5 Style — rewrite, then sanitise
  - 5.6 Blocking — the synchronous wrapper

## VOLUME 6: SETTINGS

The configuration model, its file, and the window that edits it.

- `ch6_settings.md` — Ch 6: Settings — model, persistence, page
  - 6.1 Model — every field and its default
  - 6.2 Load — tolerate old files
  - 6.3 Save — write the JSON
  - 6.4 Page — `ui/settings.html`, autosaving controls

## VOLUME 7: NOTES

A scratch pad that receives dictation.

- `ch7_notes.md` — Ch 7: Notes — the composer window
  - 7.1 Window — transparent, decorated by the page
  - 7.2 Composer — live line, word count, append
  - 7.3 Actions — insert, copy, clear

## VOLUME 8: DIAGNOSTICS

Everything that only writes to a file, and why nothing raw reaches the screen.

- `ch8_diagnostics.md` — Ch 8: Diagnostics — logs, errata, silence
  - 8.1 Log — `srlog`
  - 8.2 Bridge — `sr_error` and `debug_title`
  - 8.3 Errors — what the pill is allowed to show

---

## INDEX

*Reverse map: which paragraphs describe which source file. Migration wrote the book from the code; no file was recompiled over the existing tree.*

```
  src-tauri/src/main.rs
    ← ch1_shell.md § 1.1 § State definition
    ← ch1_shell.md § 1.2 § boot
    ← ch1_shell.md § 1.2 § setup
    ← ch1_shell.md § 1.3 § show_window
    ← ch1_shell.md § 1.3 § toggle_notes
    ← ch1_shell.md § 1.4 § setup_tray
    ← ch2_pill.md § 2.1 § current_dock, pill_footprint, work_area_logical, anchor_for
    ← ch2_pill.md § 2.2 § fit_pill, pill_fit, pill_size
    ← ch2_pill.md § 2.3 § dock_pill_to, harden_pill_window, place_pill, spawn_dock_watcher
    ← ch2_pill.md § 2.4 § spawn_hover_watcher
    ← ch2_pill.md § 2.5 § reveal_pill, pill_ready
    ← ch2_pill.md § 2.6 § spawn_pill_guard
    ← ch3_input.md § 3.1 § parse_shortcut, register_hotkey, hotkey handler
    ← ch3_input.md § 3.2 § arm_hold_start, handle_smart
    ← ch3_input.md § 3.3 § hook, worker, replay_toggle, double tap, why the key is swallowed,
      why INJECTING exists
    ← ch8_diagnostics.md § 8.2 § sr_error, debug_title
    ← ch8_diagnostics.md § 8.3 § speech bridge

  src-tauri/src/rec.rs
    ← ch4_capture.md § 4.1 § Rec definition, push_text
    ← ch4_capture.md § 4.2 § start
    ← ch4_capture.md § 4.3 § stop, toggle
    ← ch4_capture.md § 4.4 § deliver
    ← ch8_diagnostics.md § 8.1 § srlog

  src-tauri/src/ai.rs
    ← ch5_groq.md § 5.1 § short, upload
    ← ch5_groq.md § 5.2 § transcribe
    ← ch5_groq.md § 5.3 § models
    ← ch5_groq.md § 5.4 § BASE_PROMPT, tone_line, default_prompt, is_active, system_prompt
    ← ch5_groq.md § 5.5 § sanitize, style_text
    ← ch5_groq.md § 5.6 § style_blocking

  src-tauri/src/settings.rs
    ← ch6_settings.md § 6.1 § Settings struct, defaults, dir
    ← ch6_settings.md § 6.2 § load
    ← ch6_settings.md § 6.3 § save

  src-tauri/src/inject.rs
    ← ch3_input.md § 3.4 § inject_text, paste_text, ctrl_v, type_text, press_space

  src-tauri/src/autostart.rs
    ← ch3_input.md § 3.5 § reg, apply

  ui/pill.html
    ← ch2_pill.md § 2.7 § pill markup, pill states, fit and refit, applyOrient,
      boot sequence, setRec, hover listener, error box, buttons

  ui/settings.html
    ← ch6_settings.md § 6.4 § layout, load, paint, defaultPrompt, fillSelect,
      refreshModels, segment clicks, collect, schedule, bindings

  ui/notes.html
    ← ch7_notes.md § 7.1 § Window
    ← ch7_notes.md § 7.2 § titlebar, live line, append, word count
    ← ch7_notes.md § 7.3 § insert, copy, clear

  src-tauri/tauri.conf.json
    ← ch1_shell.md § 1.2 § boot (window creation lives in the config)
    ← ch2_pill.md § 2.7 § pill markup (window flags)

  src-tauri/Cargo.toml
    ← TOC.md § PREFACE
```

## APPENDIX

- `glossary.md` — types, vocabularies and axioms used across all volumes
- `design_glossary.md` — colours, spacing, radii, motion tokens for the three pages
- `skill.md` — pointer to the NOVA compiler directive
