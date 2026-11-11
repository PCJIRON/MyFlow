# MyFlow — voice dictation desktop app (Wispr Flow-style behavior, original implementation)

Hold a key, talk, release — your speech appears at the caret of whatever app
is focused. No accounts, no sign-in, no API keys: dictation runs on the speech
engine built into the WebView.

## Features (v0.1)
- Floating translucent **dictation pill** — drag it anywhere, live audio waveform, sits bottom-centre by default
- **Global hotkey** (default `space`), three activation modes:
  - **Smart** — hold = push-to-talk, quick tap = a normal Space (typing keeps working), double-tap = start/stop toggle
  - **Hold only** — press and hold to talk
  - **Toggle only** — press to start, press to stop
- **Google Speech (built-in)** — Hindi, English and Hinglish; no key needed. Language picked in Settings (`en` / `hinglish` / `hi`)
- **Verbatim insertion** — the final transcript is pasted (clipboard + Ctrl+V) or typed at your caret; nothing is rewritten or translated
- **Notes window** — a scratchpad that captures dictation instead of pasting it, with a live partial-transcript line, then Insert / Copy / Clear
- **Tray menu** — Open Settings, Open Notes, Quit (the pill is `skipTaskbar`, so this is the way to exit)
- **Start with Windows** — per-user `Run` key, no elevation, toggle in Settings

## Run it
```powershell
cd myflow
npm install
npm run dev      # first run compiles Rust (~5-10 min), then opens the pill
```
Focus any text box, hold the dictation key, speak, release — text lands at your caret.

## Architecture
- `src-tauri/src/rec.rs` — recording state + transcript accumulation; waits 350 ms after stop so the recogniser's last final result isn't dropped, then delivers verbatim
- `src-tauri/src/main.rs` — global-shortcut state machine (smart/hold/toggle), tray, settings commands, the Web-Speech bridge (`sr_final` / `sr_partial` / `sr_error`)
- `src-tauri/src/inject.rs` — caret insertion (arboard clipboard + enigo Ctrl+V, or enigo typing)
- `src-tauri/src/autostart.rs` — `HKCU\...\Run` helper for "Start with Windows"
- `src-tauri/src/settings.rs` — JSON settings in the app config dir
- `ui/pill.html`, `ui/settings.html`, `ui/notes.html` — no-framework UIs using `window.__TAURI__`
- `tools/make-tray-icon.mjs` — regenerates `src-tauri/icons/tray32.rgba` (`npm run icon`), the tray mark has no image-library dependency

Dictation depends on the WebView2 speech path. The app passes WebView2 browser
arguments that disable `msSpeechRecognitionServiceUseCetoService`; recent
runtimes route that service to a 403 and Web Speech fails with a "network"
error otherwise. Diagnostics land in `sr.log` inside the app data dir.

## Roadmap
1. Bootstrap installer + signed `latest.json`-style update feed (per-user NSIS, no elevation, sha256 + Authenticode check before launch)
2. Offline provider: local whisper.cpp, so dictation works with no network; optional cloud providers behind one adapter
3. Smart post-processing (punctuation/commands via an LLM of your choice), snippets
4. Multi-mic selection, VAD auto-stop, per-app insertion quirks

> Original code and branding throughout; no proprietary code, assets, or branding from Wispr Flow is used or reproduced. Behaviour-level parity only.
