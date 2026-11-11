//! Groq integration.
//!
//! Two jobs:
//! 1. `transcribe` — the recorded clip is uploaded to Groq Whisper, which turns
//!    the audio into text (used when the provider is set to Groq).
//! 2. `style_text` — an optional second pass that rewrites the raw transcript
//!    as clean, well-formed text in the style picked in Settings.
//!
//! Both are best-effort: if anything fails the raw transcript is what gets
//! pasted, so a dictation is never lost because of a network or key problem.

use crate::settings::Settings;
use std::time::Duration;

const BASE: &str = "https://api.groq.com/openai/v1";

/// Keeps provider error bodies readable in the pill's error line.
fn short(body: &str) -> String {
    let one: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.len() > 180 {
        format!("{}…", &one[..one.char_indices().nth(180).map(|(i, _)| i).unwrap_or(one.len())])
    } else {
        one
    }
}

async fn upload(
    cfg: &Settings,
    audio: &[u8],
    mime: &str,
    model: &str,
) -> Result<String, String> {
    let ext = match mime.split(';').next().unwrap_or("").trim() {
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/mp4" | "audio/m4a" => "m4a",
        "audio/ogg" => "ogg",
        "audio/mpeg" => "mp3",
        _ => "webm",
    };
    let part = reqwest::multipart::Part::bytes(audio.to_vec())
        .file_name(format!("dictation.{ext}"))
        .mime_str(mime.split(';').next().unwrap_or("audio/webm"))
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", model.to_string())
        .text("response_format", "json");

    let res = reqwest::Client::new()
        .post(format!("{BASE}/audio/transcriptions"))
        .bearer_auth(cfg.groq_key.trim())
        .multipart(form)
        .timeout(Duration::from_secs(90))
        .send()
        .await
        .map_err(|e| format!("Groq request failed: {e}"))?;

    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Groq {}: {}", status.as_u16(), short(&body)));
    }
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "Groq returned an unexpected response".to_string())?;
    Ok(v["text"].as_str().unwrap_or("").trim().to_string())
}

pub async fn transcribe(cfg: &Settings, audio: Vec<u8>, mime: &str) -> Result<String, String> {
    if cfg.groq_key.trim().is_empty() {
        return Err("Groq API key is missing — add it in Settings".into());
    }
    if audio.is_empty() {
        return Err("No audio was captured".into());
    }
    if cfg.stt_model.trim().is_empty() {
        return Err("Pick a transcription model in Settings".into());
    }

    let first = upload(cfg, &audio, mime, cfg.stt_model.trim()).await;
    let err = match first {
        Ok(text) => return Ok(text),
        Err(e) => e,
    };
    // A chat model cannot transcribe. Rather than making the user hunt for the
    // right pick, fall back to a whisper model from their own account.
    if err.contains("does not support") {
        if let Ok(ids) = models(cfg).await {
            if let Some(stt) = ids
                .iter()
                .find(|i| i.to_ascii_lowercase().contains("whisper"))
            {
                return upload(cfg, &audio, mime, stt).await;
            }
        }
    }
    Err(err)
}

/// Every model id available to this Groq key, so Settings can offer the real
/// live list (Llama, Qwen, GPT-OSS, …) instead of a hardcoded guess.
pub async fn models(cfg: &Settings) -> Result<Vec<String>, String> {
    if cfg.groq_key.trim().is_empty() {
        return Err("Add a Groq API key first".into());
    }
    let res = reqwest::Client::new()
        .get(format!("{BASE}/models"))
        .bearer_auth(cfg.groq_key.trim())
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("Groq request failed: {e}"))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Groq {}: {}", status.as_u16(), short(&body)));
    }
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "Groq returned an unexpected response".to_string())?;
    let mut ids: Vec<String> = v["data"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|m| m["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        return Err("Groq returned no models".into());
    }
    Ok(ids)
}

/// Instructions sent to the model. Every rule is about *cleaning up* the
/// transcript — the model must never add facts or answer the speaker.
const BASE_PROMPT: &str = "\
You are a dictation cleaner, not a chat assistant. You receive raw speech-to-text output and return that same text, cleaned.

Rules:
- Never answer, reply to, comment on or continue the text. Never ask a question back. Never say things like “Sure”, “Here is”, “Let me know” and never offer help. The text is dictation, not a message addressed to you.
- Keep every fact and the original meaning. Never add new information and never invent a reply.
- Fix grammar, punctuation, capitalisation and spacing. Split run-on sentences.
- Remove filler words, stutters, false starts and accidental repetitions (“um”, “uh”, “like”, “you know”).
- Keep names, numbers, product names and technical terms as spoken; correct a name only to its obvious standard spelling.
- Do not translate. Keep the language or language mix of the transcript (for example Hinglish stays Hinglish, in Latin script).
- If the speaker enumerates points, options or steps (e.g. “I have two options, first …, second …”), turn them into a clean list: one item per line, each starting with a dash. Keep the speaker's wording and order.
- Use paragraph breaks only when the text is long enough to need them. Do not add headings, greetings or emoji.
- Return only the cleaned text. No quotes, no preface, no explanation, no question back.

Example
Input: um so my name is prashant chauhan and i need two options first my primary school second mongodb
Output: My name is Prashant Chauhan, and I need two options: my primary school and MongoDB.";

fn tone_line(style: &str) -> Option<&'static str> {
    match style {
        "formal" => Some("Tone: professional and formal, suitable for an email or a work message. Complete sentences, no slang."),
        "chat" => Some("Tone: chat message. Short, natural and friendly — how a person types to a colleague. No greetings unless spoken."),
        "casual" => Some("Tone: relaxed everyday language. Conversational and tidy, contractions are fine."),
        _ => None,
    }
}

/// The exact instruction text a style sends, so Settings can show the same
/// prompt the app uses and the user can edit it.
pub fn default_prompt(style: &str) -> String {
    match tone_line(style) {
        Some(tone) => format!("{BASE_PROMPT}\n\n{tone}"),
        None => BASE_PROMPT.to_string(),
    }
}

fn is_active(cfg: &Settings) -> bool {
    cfg.style != "off"
}

/// A custom prompt replaces the built-in tone section but still gets the base
/// rules; an empty custom prompt means plain clean-up.
fn system_prompt(cfg: &Settings) -> String {
    let custom = cfg.style_prompt.trim();
    if custom.is_empty() {
        default_prompt(&cfg.style)
    } else {
        custom.to_string()
    }
}

/// Guards against the model answering the dictation instead of cleaning it:
/// prefaces, emoji, invented questions and wild length changes are rejected and
/// the raw transcript is used instead.
fn sanitize(out: &str, raw: &str) -> Option<String> {
    let mut t = out.trim().to_string();
    for wrapper in ["```", "\"\"\"", "\"", "``", "`"] {
        t = t.trim_start_matches(wrapper).trim_end_matches(wrapper).trim().to_string();
    }
    let lower = t.to_ascii_lowercase();
    for p in [
        "sure, ", "sure! ", "sure. ", "of course, ", "of course! ", "certainly, ",
        "here is the cleaned text: ", "here's the cleaned text: ", "here is: ",
        "cleaned text: ", "output: ", "transcript: ",
    ] {
        if lower.starts_with(p) {
            t = t[p.len()..].trim().to_string();
            break;
        }
    }
    t = t
        .chars()
        .filter(|c| {
            let u = *c as u32;
            !((0x1F000..=0x1FAFF).contains(&u)
                || (0x2600..=0x27BF).contains(&u)
                || (0x2190..=0x21FF).contains(&u)
                || u == 0xFE0F)
        })
        .collect::<String>();
    let t = t.trim().to_string();
    if t.is_empty() {
        return None;
    }
    // The model asked something back, or blew the text up into a reply.
    if t.contains('?') && !raw.contains('?') {
        return None;
    }
    if t.len() > raw.len() * 3 + 60 {
        return None;
    }
    if raw.len() > 12 && t.len() * 4 < raw.len() {
        return None;
    }
    Some(t)
}

/// Rewrites `raw` in the configured style. Returns `raw` untouched when the
/// style is off, the key is missing, or the request fails.
pub async fn style_text(cfg: &Settings, raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() {
        return raw.to_string();
    }
    if !is_active(cfg) {
        return raw.to_string();
    }
    if cfg.groq_key.trim().is_empty() || cfg.style_model.trim().is_empty() {
        return raw.to_string();
    }
    let body = serde_json::json!({
        "model": cfg.style_model.trim(),
        "temperature": 0.0,
        "messages": [
            { "role": "system", "content": system_prompt(cfg) },
            { "role": "user", "content": text }
        ]
    });
    let res = reqwest::Client::new()
        .post(format!("{BASE}/chat/completions"))
        .bearer_auth(cfg.groq_key.trim())
        .json(&body)
        .timeout(Duration::from_secs(45))
        .send()
        .await;
    let Ok(res) = res else {
        return raw.to_string();
    };
    if !res.status().is_success() {
        return raw.to_string();
    }
    let Ok(v) = res.json::<serde_json::Value>().await else {
        return raw.to_string();
    };
    let out = v["choices"][0]["message"]["content"].as_str().unwrap_or("");
    sanitize(out, text).unwrap_or_else(|| raw.to_string())
}

/// Blocking wrapper for the Google pipeline, which finishes on a plain thread.
pub fn style_blocking(app: &tauri::AppHandle, raw: String) -> String {
    let cfg = crate::settings::load(app);
    if !is_active(&cfg) || cfg.groq_key.trim().is_empty() {
        return raw;
    }
    tauri::async_runtime::block_on(style_text(&cfg, &raw))
}
