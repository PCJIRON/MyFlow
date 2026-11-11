# Chapter 6: Settings — model, persistence, page

*Volume 6. Source: `src-tauri/src/settings.rs`, `ui/settings.html`.*

## Section 6.1: Model — every field and its default

#### Paragraph: Settings struct

Eleven fields, see the glossary entry **Settings**. Serialised and deserialised with serde; every optional field carries `#[serde(default = "...")]` so an old file still loads.

#### Paragraph: defaults

```
def_lang   → "hinglish"
def_hotkey → "space"
def_mode   → "smart"
def_inject → "paste"
def_style  → "off"
```
`Default for Settings` uses these and empty strings for the key, models and custom prompt.

#### Paragraph: dir

`app.path().app_config_dir()`.

## Section 6.2: Load — tolerate old files

#### Paragraph: load

File `dir/settings.json` exists and parses?
`**s** is the parsed struct.`
Fix removed language values: `"auto" | "hi-en" | "en-hi" | ""` → "hinglish".
Empty provider → "google".
Return `s`.
Otherwise → `Settings::default()`.

*Removed fields (model, api_key, polish*) are ignored by serde, so an upgrade never loses the settings the user still has.*

## Section 6.3: Save — write the JSON

#### Paragraph: save

`**d** is dir`, else "no config dir".
`create_dir_all(d)`.
`**txt** is serde_json pretty`.
`write(d/settings.json, txt)`.

*No atomic rename: a failed write surfaces as a toast in the page, and the next autosave retries.*

## Section 6.4: Page — `ui/settings.html`, autosaving controls

#### Paragraph: layout

A header (icon plate, "MyFlow", one-line sub) and five cards: **Speech engine**, **Language**, **Writing style**, **Dictation keys**, **Startup**. A fixed toast sits bottom-right.

*Every control saves itself; there is deliberately no Save button.*

#### Paragraph: load

Read settings. Paint the provider, language and style segments (validating against the known vocabularies). Fill the prompt box: a saved `style_prompt` wins, otherwise the style's own prompt from `style_prompt_default`. Fill key, hotkey, mode, inject, autostart. Call `paint()`. If a key exists, refresh the model lists.

#### Paragraph: paint

Show `#groq-fields` only when provider is "groq".
Show `#style-fields` only when style is not "off".
Swap the provider hint text between "clip uploaded to Groq" and "WebView's own engine".

#### Paragraph: defaultPrompt

"off" or "custom" → "". Otherwise memoised `invoke("style_prompt_default", { style })`.

#### Paragraph: fillSelect

No list → one option with the empty text, value "".
Otherwise one option per id, and select `current` only if it is in the list, else the first.

*A model removed from the account is dropped rather than silently kept.*

#### Paragraph: refreshModels

Read the key and both current selections. No key → both selects show "— add a Groq API key to load models —" and stop.
Otherwise `invoke("groq_models")`, then split by capability: ids matching `/whisper/i` are transcription models, the rest are chat models. Fill both, rewrite `#models-hint` with the counts, and if a selection changed, persist it immediately.

#### Paragraph: segment clicks

`provider-seg`, `lang-seg`, `style-seg`: set the clicked value, and for `style-seg`
  "custom" → empty the prompt box and focus it;
  otherwise → fill the box with that style's built-in prompt.
Then `paint()` and `schedule(false)`.

#### Paragraph: collect

Build the settings object from the page. `style_prompt` is stored as "" when it equals the built-in prompt for the selected style, *so presets keep following their own text after an update*.

#### Paragraph: schedule

Debounce 450ms (or 0 when saving immediately), then `set_settings`; success → toast "Saved", failure → error toast.

#### Paragraph: bindings

`groq_key`, `hotkey`, `style_prompt`: input → `schedule(false)`, blur → `schedule(true)`.
`stt_model`, `style_model`: change → `schedule(true)`.
`groq_key` blur also refreshes the model lists.
`refresh-models` button → `refreshModels(true)`.
`mode`, `inject`, `autostart`: change → `schedule(true)`.

`#### Proofread: settings`
Change the language: a "Saved" toast appears once and the pill picks the new language on the next session. Close and reopen the window: the choice is still there, one row per setting, no duplicates.
