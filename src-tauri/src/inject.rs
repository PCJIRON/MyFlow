//! Insert transcribed text at the current caret.
//! "paste" = clipboard + Ctrl+V (fast, reliable across apps)
//! "type"  = synthesized keystrokes (works where paste is blocked)

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub fn inject_text(text: &str, method: &str) {
    if text.trim().is_empty() {
        return;
    }
    match method {
        "type" => type_text(text),
        _ => paste_text(text),
    }
}

fn new_enigo() -> Option<Enigo> {
    Enigo::new(&Settings::default()).ok()
}

fn paste_text(text: &str) {
    // small delay so the focused app settles after the hotkey was released
    std::thread::sleep(std::time::Duration::from_millis(120));
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(text.to_string())) {
        Ok(()) => ctrl_v(),
        Err(_) => type_text(text), // clipboard blocked -> fall back
    }
}

fn ctrl_v() {
    let Some(mut enigo) = new_enigo() else { return };
    let _ = enigo.key(Key::Control, Direction::Press);
    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);
}

fn type_text(text: &str) {
    let Some(mut enigo) = new_enigo() else { return };
    let _ = enigo.text(text);
}

/// Forward a real Space keypress (used by the "smart space" hotkey so normal
/// typing keeps working — a quick tap of Space that isn't a hold or double-tap
/// is re-synthesized into the focused app).
pub fn press_space() {
    let Some(mut enigo) = new_enigo() else { return };
    let _ = enigo.key(Key::Unicode(' '), Direction::Click);
}
