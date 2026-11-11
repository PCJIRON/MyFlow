# Chapter 4: Capture — start, stop, accumulate, deliver

*Volume 4. Source: `src-tauri/src/rec.rs`.*

*The pill's WebView runs the recogniser. Rust owns the session: it opens the state, accumulates finals as they arrive, and on stop delivers exactly what was said — no rewrites beyond the optional style pass, no translation.*

## Section 4.1: Rec — the state object

#### Paragraph: Rec definition

`STRUCT Rec` with `FIELD: **running** is AtomicBool`, `FIELD: **sr_acc** is Mutex<String>`, `FIELD: **gen** is AtomicU64` (private).
`Default` builds running=false, empty accumulator, gen=0.

#### Paragraph: push_text (module helper)

Trim; empty → return.
If the accumulator is non-empty, does not end with a space, and the new text does not start with one → push a single space.
Push the text.

*A shared helper: the same rule is used by the start/stop path here and by the command layer in Chapter 8.*

## Section 4.2: Start — one running session

#### Paragraph: start

`running?` → return Ok (*idempotent*).
`gen += 1`.
Clear `sr_acc`.
Set `running = true`.
Log "rec:start ok (google path)".
Emit `record-state` "started".

*The page reacts to the event: it switches the bar to the recording capsule and starts whichever engine the settings name.*

## Section 4.3: Stop — flush the tail, then deliver

#### Paragraph: stop

`running` was already false → return.
Clear `running`; log "rec:stop".
Emit `record-state` "stopping" and `partial` "".
`**first** is the accumulator, taken.`
`**generation** is gen.read().`
Spawn a thread: sleep 350ms — *give the recogniser a beat to flush its last final result instead of dropping the tail of a sentence*.
If `gen` is still `generation`: take the accumulator; if non-empty, `push_text` into `first`.
If the text is non-empty: `**styled** is ai::style_blocking(&app, text)` (CITES Section 5.6 from ch5_groq) and `deliver(&app, styled)`.

#### Paragraph: toggle

`running?` → `stop`. `Otherwise?` → `start`; an error is emitted as `stt-error`.

## Section 4.4: Delivery — notes window or caret

#### Paragraph: deliver

Log `rec:deliver len={}`.
Emit `final` with the text.
`**use_notes** is: the "notes" window exists AND is visible.`
`use_notes?` → emit `note-append`.
`Otherwise, text not empty?` → `**method** is settings.inject; `inject::inject_text(text.trim(), &method)` (CITES Section 3.4 from ch3_input).

*Two destinations, one decision point: an open Notes window wins, otherwise the text goes to the caret.*

`#### Proofread: capture`
Open Notes, dictate two sentences: both land in the pad, separated, and nothing is pasted elsewhere. Close Notes, dictate: the text appears at the caret in the focused app.
