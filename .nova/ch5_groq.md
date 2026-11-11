# Chapter 5: Groq — transcribe, models, style

*Volume 5. Source: `src-tauri/src/ai.rs`.*

*Two jobs: turn a recorded clip into text, and optionally rewrite that text in a chosen style. Both are best-effort — if anything fails, the raw transcript is what gets pasted, so a dictation is never lost to a network or key problem.*

## Section 5.1: Client — the shared upload path

#### Paragraph: short

Collapse whitespace, and if longer than 180 characters cut at a char boundary and append "…". *Keeps provider error bodies readable in the pill's error line.*

#### Paragraph: upload

`**ext**` from the mime: wav, m4a, ogg, mp3, else webm.
`**part**` — bytes, file name `dictation.{ext}`, mime without parameters.
`**form**` — the file part, `model`, `response_format=json`.
POST `{BASE}/audio/transcriptions` with bearer `groq_key`, timeout 90s.
Request error → `Groq request failed: {e}`.
Non-2xx → `Groq {status}: {short(body)}`.
Parse JSON; return `text`, trimmed.

## Section 5.2: Transcribe — clip to text, with model fallback

#### Paragraph: transcribe

`groq_key` empty → "Groq API key is missing — add it in Settings".
`audio` empty → "No audio was captured".
`stt_model` empty → "Pick a transcription model in Settings".
`**first** is upload(...)`. Ok → return the text; Err → keep the message.
If the message contains "does not support" — *a chat model cannot transcribe* — fetch `models(cfg)` and pick the first id containing "whisper"; if one exists, upload again with it.
Otherwise return the first error.

*The user picks from their own account, so a wrong pick is a normal mistake; the fallback saves a hunt through Settings.*

## Section 5.3: Models — the live list from the user's account

#### Paragraph: models

`groq_key` empty → "Add a Groq API key first".
GET `{BASE}/models`, bearer auth, timeout 20s.
Non-2xx → `Groq {status}: {short(body)}`.
Collect every `data[].id`, sort, dedupe.
Empty → "Groq returned no models".

*Nothing is hardcoded: Settings offers exactly what the account has.*

## Section 5.4: Prompt — the dictation cleaner instructions

#### Paragraph: BASE_PROMPT

A single string that frames the model as a **dictation cleaner, not a chat assistant**. Rules, in order: never answer or comment, never offer help; keep every fact and never add information; fix grammar, punctuation, capitalisation, spacing and run-on sentences; drop filler and stutters; keep names, numbers and technical terms, correcting only to an obvious standard spelling; do not translate, keep the language mix (Hinglish stays Hinglish in Latin script); turn spoken enumerations into a dash list, one item per line, keeping the speaker's wording and order; paragraph breaks only when needed, no headings, greetings or emoji; return only the cleaned text.

It ends with one worked example.

#### Paragraph: tone_line

```
formal → "Tone: professional and formal, suitable for an email or a work message. Complete sentences, no slang."
chat   → "Tone: chat message. Short, natural and friendly … No greetings unless spoken."
casual → "Tone: relaxed everyday language. Conversational and tidy, contractions are fine."
other  → None
```

#### Paragraph: default_prompt

A tone exists → `"{BASE_PROMPT}\n\n{tone}"`. Otherwise → `BASE_PROMPT`.

*Settings shows this exact text through the `style_prompt_default` command, so the box always matches what will be sent.*

#### Paragraph: is_active

`style != "off"`.

#### Paragraph: system_prompt

A non-empty `style_prompt` replaces the built-in text entirely; empty means `default_prompt(style)`.

## Section 5.5: Style — rewrite, then sanitise

#### Paragraph: sanitize

Trim. Strip wrapping ``` / """ / " / ` sequences.
Strip a leading assistant preface from a fixed list ("sure, ", "of course, ", "here is the cleaned text: ", "output: ", "transcript: ", …).
Remove emoji, dingbats, arrows and the variation selector.
Empty after that → None.
If the output contains "?" and the raw text does not → None (*the model asked something back*).
If output > raw*3 + 60 → None (*it blew the text up into a reply*).
If raw > 12 and output*4 < raw → None (*it dropped most of the text*).
Otherwise Some(text).

#### Paragraph: style_text

Empty → return raw.
`!is_active` → raw.
Key or model missing → raw.
POST `{BASE}/chat/completions` with `model`, `temperature: 0.0`, and the two messages (system = `system_prompt`, user = the text), timeout 45s.
Any failure — request, status, JSON — → raw.
`**out**` is `choices[0].message.content`; return `sanitize(out, text)`, falling back to raw.

## Section 5.6: Blocking — the synchronous wrapper

#### Paragraph: style_blocking

Load settings. If the style is off or the key is empty → return raw (*no await, no cost*).
`tauri::async_runtime::block_on(style_text(&cfg, &raw))`.

*The Google pipeline finishes on a plain thread, so it cannot await.*

`#### Proofread: style`
With style "off": the pasted text is the transcript, untouched. With "formal": the same dictation comes back punctuated and capitalised, with no added sentence and no answer.
