//! Google Web Speech pipeline: the pill's WebView runs SpeechRecognition
//! (Google engine). Rust keeps recording state, accumulates final results as
//! they arrive, and on stop delivers the text **verbatim** — no rewrites, no
//! polish, no translation. What the user said is what gets pasted.

use crate::settings::Settings;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub struct Rec {
    pub running: AtomicBool,
    /// Google path: final results accumulate here until stop delivers them.
    pub sr_acc: Mutex<String>,
    /// Bumped on every start so a delivery thread from an older session can
    /// tell it has been superseded.
    gen: AtomicU64,
}

impl Default for Rec {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            sr_acc: Mutex::new(String::new()),
            gen: AtomicU64::new(0),
        }
    }
}

pub fn srlog(app: &AppHandle, msg: &str) {
    if let Some(p) = app
        .path()
        .app_data_dir()
        .ok()
        .map(|d| d.join("sr.log"))
    {
        use std::io::Write;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let ts = format!("{}.{:03}", now.as_secs(), now.subsec_millis());
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
            let _ = writeln!(f, "[{}] RUST-{}", ts, msg);
        }
    }
}

fn push_text(acc: &mut String, t: &str) {
    let t = t.trim();
    if t.is_empty() {
        return;
    }
    if !acc.is_empty() && !acc.ends_with(' ') && !t.starts_with(' ') {
        acc.push(' ');
    }
    acc.push_str(t);
}

impl Rec {
    pub fn start(self: &Arc<Self>, app: AppHandle, _cfg: Settings) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.gen.fetch_add(1, Ordering::SeqCst);
        self.sr_acc.lock().unwrap().clear();
        self.running.store(true, Ordering::SeqCst);
        srlog(&app, "rec:start ok (google path)");
        let _ = app.emit("record-state", "started");
        Ok(())
    }

    pub fn stop(self: &Arc<Self>, app: &AppHandle) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }
        srlog(app, "rec:stop");
        let _ = app.emit("record-state", "stopping");
        let _ = app.emit("partial", String::new());
        // Snapshot what we have, then give the recogniser a beat to flush its
        // last final result instead of dropping the tail of the sentence.
        let first = {
            let mut g = self.sr_acc.lock().unwrap();
            std::mem::take(&mut *g)
        };
        let generation = self.gen.load(Ordering::SeqCst);
        let this = self.clone();
        let app2 = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(350));
            let mut text = first;
            if this.gen.load(Ordering::SeqCst) == generation {
                let late = {
                    let mut a = this.sr_acc.lock().unwrap();
                    std::mem::take(&mut *a)
                };
                if !late.trim().is_empty() {
                    push_text(&mut text, &late);
                }
            }
            if !text.trim().is_empty() {
                // Optional writing-style pass (Groq). Falls back to the raw
                // transcript when the pass is off or the request fails.
                let styled = crate::ai::style_blocking(&app2, text);
                deliver(&app2, styled);
            }
        });
    }

    pub fn toggle(self: &Arc<Self>, app: &AppHandle, cfg: Settings) {
        if self.running.load(Ordering::SeqCst) {
            self.stop(app);
        } else if let Err(e) = self.start(app.clone(), cfg) {
            let _ = app.emit("stt-error", e);
        }
    }
}

/// Finish path: emit "final" and route the text — into the Notes composer when
/// that window is open, otherwise straight to the caret (clipboard paste).
pub fn deliver(app: &AppHandle, raw: String) {
    srlog(app, &format!("rec:deliver len={}", raw.len()));
    let _ = app.emit("final", raw.clone());
    let use_notes = app
        .get_webview_window("notes")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    if use_notes {
        let _ = app.emit("note-append", raw);
    } else if !raw.trim().is_empty() {
        let method = crate::settings::load(app).inject;
        crate::inject::inject_text(raw.trim(), &method);
    }
}
