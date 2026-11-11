# Chapter 8: Diagnostics — logs, errata, silence

*Volume 8. Source: `src-tauri/src/rec.rs` (`srlog`), `src-tauri/src/main.rs` (`sr_error`, `debug_title`, `sr_final`, `sr_partial`, `push_text`, speech bridge).*

*Rule of this chapter: diagnostics go to a file; only failures the user can act on reach the screen.*

## Section 8.1: Log — `srlog`

#### Paragraph: srlog

`**path**` is `app_data_dir()/sr.log`.
`**ts**` is `"{secs}.{millis:03}"` from the system clock.
Open the file create+append; write one line: `[{ts}] RUST-{msg}`.

*Every Rust line carries the `RUST-` prefix; the page's own lines go through `sr_error` and carry `sr:js`. One file, one interleaved timeline.*

## Section 8.2: Bridge — `sr_error` and `debug_title`

#### Paragraph: sr_error (command)

Write `[{seconds}.{millis:03}] {msg}` to `sr.log`.
Do not emit anything. *It used to re-broadcast every line as `stt-error`, which made the pill pop a red box showing raw diagnostics like `sr:result final=true t=…` above the bar on every phrase.*

#### Paragraph: debug_title (command)

Set the pill window title to the given string. *A probe only: the outside can verify the page's JS is alive even when logging fails.*

## Section 8.3: Errors — what the pill is allowed to show

#### Paragraph: error sources

`stt-error` is emitted only for things the user can fix: a failed `rec.start`, a transcription error from `groq_transcribe`, a missing key or model.
The page ignores any payload that is empty or starts with "sr:".

*Speech-engine lifecycle lines (`sr:start`, `sr:end`, `sr:no-speech`) stay in the file.*

#### Paragraph: speech bridge (Google path)

`sr_final(text)` → append to the accumulator with the shared `push_text` rule.
`sr_partial(text)` → emit `partial` with the trimmed text; never inserted anywhere.
`groq_transcribe(audio, mime)` → base64-decode (*Bad audio payload: {e}*), log the clip size, `ai::transcribe`, trim, log an empty transcript and return Ok if empty, otherwise log the length, run `ai::style_text`, then `rec::deliver`.
`groq_models()` → `ai::models` for the page.
`style_prompt_default(style)` → `ai::default_prompt`.

#### Paragraph: reading the log

The file is append-only across runs. When a session looks silent, the order to check is: `pill revealed`, `pill docked …`, `rec:start ok`, `sr:start-ok`, `rec:stop`, `rec:deliver len=…`, then `groq: transcript len=…` on the Groq path.

`#### Proofread: silence`
Dictate one sentence with the Google engine. The log contains the full `sr:` trace; the pill shows no red box at any point. Then brake the engine (block the network): the pill shows one short, readable message.
