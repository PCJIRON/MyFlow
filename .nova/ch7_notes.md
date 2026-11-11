# Chapter 7: Notes — the composer window

*Volume 7. Source: `ui/notes.html` (window declared in `tauri.conf.json`).*

## Section 7.1: Window — transparent, decorated by the page

`label "notes"`, 400x480, resizable, `decorations: false`, `transparent: true`, not always-on-top, `visible: false`.
The page paints its own shell: `#win` is a 16px-radius card on `rgba(19,19,23,.94)` with `blur(18px)` and one hairline border.

*The window is created hidden and toggled from Rust, so the tray entry and the pill button share one toggle.*

## Section 7.2: Composer — live line, word count, append

#### Paragraph: titlebar

Icon plate, "Notes", a "Listening…" indicator with a blinking dot, and a close button that calls `show_notes` (*which hides the window, since it is already visible*).

#### Paragraph: live line

`record-state` "started" → add `.isrec` to the shell; anything else removes it.
`partial` → while `.isrec`, write the interim text into `#live`.
`final` → clear `#live`.

*Interim text is only ever a preview here; nothing is inserted until the final transcript arrives.*

#### Paragraph: append

`note-append` → take the payload, trim, ignore empty. Append a newline first if the pad already has text and does not end with one. Append the text, scroll to the bottom, update the count.

#### Paragraph: word count

The pad's input handler sets `#count` to the number of whitespace-separated words, or "0 words".

## Section 7.3: Actions — insert, copy, clear

#### Paragraph: insert

`invoke("insert_notes_text", { text })` → clear the pad and the count. On failure the count line shows the error.

*Rust pastes the text at the caret, so the notes window can feed any other app.*

#### Paragraph: copy

`navigator.clipboard.writeText(pad.value)`; the count line shows "Copied" or the error.

#### Paragraph: clear

Empty the pad and the count.

`#### Proofread: notes`
Open Notes, dictate a sentence in three pieces: each final appears on its own line, the live line clears between phrases, and the word count matches. Press Insert and the whole pad is pasted into another window.
