use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Speech engine: "google" (built-in Web Speech, no key) or "groq"
    // (audio clip → Groq Whisper).
    pub provider: String,
    // Language: "en" | "hinglish" | "hi"
    #[serde(default = "def_lang")]
    pub language: String,
    // UX
    #[serde(default = "def_hotkey")]
    pub hotkey: String, // e.g. "space" (smart), "ctrl+shift+space"
    #[serde(default = "def_mode")]
    pub mode: String, // smart | hold | toggle
    #[serde(default = "def_inject")]
    pub inject: String, // paste | type — final delivery
    #[serde(default)]
    pub autostart: bool, // launch at login (HKCU Run key)
    // Groq (only used when provider == "groq", or for the writing-style pass)
    #[serde(default)]
    pub groq_key: String,
    #[serde(default)]
    pub stt_model: String, // whatever the user picked from their own Groq account
    // Writing style applied after transcription: off | formal | chat | casual
    #[serde(default = "def_style")]
    pub style: String,
    #[serde(default)]
    pub style_model: String,
    // User-authored prompt. Empty means "use the built-in prompt for `style`".
    #[serde(default)]
    pub style_prompt: String,
}

fn def_lang() -> String {
    "hinglish".into()
}
fn def_hotkey() -> String {
    "space".into()
}
fn def_mode() -> String {
    "smart".into()
}
fn def_inject() -> String {
    "paste".into()
}
fn def_style() -> String {
    "off".into()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "google".into(),
            language: def_lang(),
            hotkey: def_hotkey(),
            mode: def_mode(),
            inject: def_inject(),
            autostart: false,
            groq_key: String::new(),
            stt_model: String::new(),
            style: def_style(),
            style_model: String::new(),
            style_prompt: String::new(),
        }
    }
}

pub fn dir(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    use tauri::Manager;
    app.path().app_config_dir().ok()
}

/// Old saved files may contain removed fields (model/api_key/polish*) — serde
/// ignores unknown keys, so load maps the remaining ones and fixes removed
/// language values.
pub fn load(app: &tauri::AppHandle) -> Settings {
    if let Some(d) = dir(app) {
        let p = d.join("settings.json");
        if p.exists() {
            if let Ok(txt) = std::fs::read_to_string(&p) {
                if let Ok(s) = serde_json::from_str::<Settings>(&txt) {
                    let mut s = s;
                    match s.language.as_str() {
                        "auto" | "hi-en" | "en-hi" | "" => s.language = "hinglish".into(),
                        _ => {}
                    }
                    if s.provider.trim().is_empty() {
                        s.provider = "google".into();
                    }
                    return s;
                }
            }
        }
    }
    Settings::default()
}

pub fn save(app: &tauri::AppHandle, s: &Settings) -> Result<(), String> {
    let d = dir(app).ok_or("no config dir")?;
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let txt = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(d.join("settings.json"), txt).map_err(|e| e.to_string())
}
