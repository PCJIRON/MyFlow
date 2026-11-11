# NOVA Compiler Directive — pointer

This book is compiled with the **NOVA v2.0.0** skill, installed globally as the account skill `nova`
(source: <https://github.com/PCJIRON/NOVA>, file `.nova/skill.md` in that repo).

Nothing is duplicated here: load the `nova` skill for the rules, the syntax dictionary, the four
modes and the anti-hallucination shield.

## Project-specific binding

- **Stack:** see `## PREFACE` in `TOC.md`. Never assume anything outside it.
- **Source of truth:** `TOC.md` → `ch*.md` → `glossary.md` → `design_glossary.md`.
- **Output directory:** the existing `src-tauri/src/` and `ui/` trees. This migration is
  read-only over them (Rule 2); compiled output overwrites nothing without the user asking.
- **Log file:** `%APPDATA%\com.myflow.dev\sr.log` — the only place raw diagnostics belong.
