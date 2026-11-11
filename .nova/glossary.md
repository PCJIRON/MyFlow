# MyFlow — Glossary

*Strict global data. If a term is not here, it does not exist in this book.*

## VOCABULARY: Provider

```
provider → google | groq
```
"google" uses the WebView's own SpeechRecognition. "groq" records a clip and uploads it.

## VOCABULARY: Language

```
language → en | hinglish | hi
```

## VOCABULARY: Mode

```
mode → smart | hold | toggle
```

## VOCABULARY: InjectMethod

```
inject → paste | type
```

## VOCABULARY: Style

```
style → off | formal | chat | casual | custom
```
"custom" is a page-only value: it saves `style_prompt` and leaves `style` at the preset the user came from. `off` pastes the transcript verbatim.

## VOCABULARY: Dock

```
dock → bottom | left | right
```
Stored in `PILL_DOCK` as 0 | 1 | 2.

## GLOSSARY: Settings

The persisted configuration. One JSON file, no database.

```
FIELD: provider      is string (Provider).
FIELD: language      is string (Language), default "hinglish".
FIELD: hotkey        is string, default "space".
FIELD: mode          is string (Mode), default "smart".
FIELD: inject        is string (InjectMethod), default "paste".
FIELD: autostart     is boolean, default false.
FIELD: groq_key      is string, default empty — never logged, never emitted.
FIELD: stt_model     is string, default empty.
FIELD: style         is string (Style), default "off".
FIELD: style_model   is string, default empty.
FIELD: style_prompt  is string, default empty — empty means "use the built-in prompt".
```

## GLOSSARY: Rec

The recording state, shared through `Arc<Rec>` inside `State`.

```
FIELD: running  is AtomicBool  — a session is open.
FIELD: sr_acc   is Mutex<String> — final results accumulate here until stop.
FIELD: gen      is AtomicU64 (private) — bumped on every start.
```

## GLOSSARY: State

Tauri-managed application state (`State::default()`).

```
FIELD: rec           is Arc<Rec>.
FIELD: registered    is Mutex<Option<String>> — the hotkey currently registered.
FIELD: last_press    is Mutex<Option<Instant>> — smart-mode tap timing.
FIELD: press_start   is Mutex<Option<Instant>> — the press still being held.
FIELD: toggled       is AtomicBool — double-tap toggle is recording.
FIELD: skip_forward  is AtomicBool — this release must not type a space.
```

## GLOSSARY: PillFootprint

```
A pair (width, height) in logical pixels for one Dock and one state.
state → idle | open | rec
```

## GLOSSARY: WorkArea

```
A tuple (x, y, width, height) of a monitor's work area, in LOGICAL pixels.
Tauri reports it in physical pixels; mixing the two is the bug this type prevents.
```

## GLOSSARY: KbdHook

The `KBDLLHOOKSTRUCT` layout used by the low-level keyboard hook.

```
FIELD: vk    is u32   — virtual key code.
FIELD: scan  is u32.
FIELD: flags is u32   — bit 0x80 (LLKHF_UP) means released.
FIELD: time  is u32.
FIELD: extra is usize — dwExtraInfo. Non-zero on an injected CapsLock toggle makes
                          Windows ignore the toggle, so this module never sends one.
```

## GLOSSARY: Monitor Rect / Point

```
Point → { x: i32, y: i32 }
Rect  → { l, t, r, b: i32 }  — physical pixels, straight from Win32.
```

## AXIOM: SMART_HOLD_MS is 200.
A hotkey press shorter than this is a tap, not push-to-talk.

## AXIOM: SMART_DOUBLE_MS is 320.
Two presses closer than this are a double-tap: toggle recording.

## AXIOM: CAPS_HOLD_MS is 260.
A CapsLock press shorter than this is a normal CapsLock tap.

## AXIOM: CAPS_STALE_MS is 30000.
A CapsLock press whose key-up never arrived expires after this, so the key cannot stay locked out.

## GLOSSARY: Caps

The keys's single state, guarded by one mutex.

```
FIELD: held      is bool            — the key is physically down.
FIELD: dictating is bool            — the timer already turned this press into a session.
FIELD: press_at  is Option<Instant> — when it went down; None when idle.
```

## AXIOM: PILL_MARGIN is 2.
Logical pixels between the pill window and the taskbar.

## AXIOM: VK_CAPITAL toggles only untagged, on release.
An injected CapsLock toggle is ignored while the key is held, and ignored outright
when it carries a non-zero dwExtraInfo.

## AXIOM: VK_CAPITAL is 0x14.
## AXIOM: LLKHF_UP is 0x80.
## AXIOM: KEYEVENTF_KEYUP is 2.
## AXIOM: WH_KEYBOARD_LL is 13.
## AXIOM: TRAY_SIZE is 32.
## AXIOM: GROQ_BASE is "https://api.groq.com/openai/v1".
## AXIOM: SR_LOG is "sr.log" in `app_data_dir()`.
## AXIOM: SETTINGS_FILE is "settings.json" in `app_config_dir()`.
## AXIOM: RUN_KEY is "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".
## AXIOM: RUN_VALUE is "MyFlow".
## AXIOM: CREATE_NO_WINDOW is 0x0800_0000.

## GLOSSARY: Event names (Rust → page)

```
record-state   "started" | "stopping" | "stopped"
partial        interim transcript, cleared to "" on every stop
final          the finished transcript, before routing
note-append    transcript bound for the Notes composer
pill-orient    "h" | "v" — the pill was re-docked
pill-hover     bool — cursor entered or left the pill window
stt-error      a user-facing failure only; never diagnostics
```

## GLOSSARY: Commands (page → Rust)

```
get_settings          → Settings
set_settings(settings)→ save, re-register hotkey, apply autostart
toggle_record         → start or stop
stop_recording        → stop
show_settings         → raise the settings window
show_notes            → toggle the notes window
style_prompt_default(style) → the built-in prompt text
pill_orient           → "h" | "v"
pill_ready            → the page painted; reveal the window
pill_fit(state)       → resize the window to the bar
insert_notes_text(text)→ paste given text
sr_final(text)        → accumulate a final result (Google path)
sr_partial(text)      → live interim preview
groq_transcribe(audio, mime) → clip → text → deliver
groq_models           → live model ids from the account
sr_error(msg)         → append a line to sr.log
debug_title(title)    → set the pill window title (probe only)
```
