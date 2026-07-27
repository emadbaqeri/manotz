# Roadmap

Progress for **manotz** toward a terminal-first knowledge management editor
(Helix/Kakoune-style selection-first editing + Obsidian-compatible markdown vault).

Legend: ✅ done · 🚧 in progress · ⬜ planned

The long-form product vision remains in
[GitHub issue #1](https://github.com/emadbaqeri/manotz/issues/1).
This file is the living checklist for what is shipped and what comes next.

---

## 1. Editor foundation

| Status | Item |
| --- | --- |
| ✅ | Grapheme and display-width helpers (`text`) |
| ✅ | `Buffer` trait + gap-buffer storage (`buffer`) |
| ✅ | Selections and non-overlapping selection sets (`selection`) |
| ✅ | Edit transactions and cursor motions (`command`) |
| ✅ | Pure render grid + crossterm terminal adapter (`render`) |
| ✅ | Modal keymap (Normal / Insert) (`input`) |
| ✅ | Editor state + terminal event loop (`editor`) |
| ✅ | Branching undo history with insert merge window (`history`) |
| ✅ | Undo / redo wired to keys (`u` / `U`) |
| ✅ | Caret past last character (end-of-buffer insert / backspace) |

## 2. Selection-first editing

| Status | Item |
| --- | --- |
| ⬜ | Select mode (extend selection with motions) |
| ⬜ | Select-then-act delete / change / yank |
| ⬜ | Multiple cursors as a first-class workflow |
| ⬜ | Primary selection among many |
| ⬜ | Multi-key chords (e.g. `g s`) |

## 3. Files and persistence

| Status | Item |
| --- | --- |
| ⬜ | Open a file into the buffer |
| ⬜ | Save buffer to disk |
| ⬜ | Dirty / clean tracking |
| ⬜ | Restore last-open note and per-note cursor (`.manotz/state.json`) |

## 4. Markdown editing

| Status | Item |
| --- | --- |
| ⬜ | Live syntax highlighting (headings, emphasis, code, links) |
| ⬜ | Single-pass markdown parse → styles + extraction |
| ⬜ | Raw markdown editing (not WYSIWYG) |

## 5. Vault and identity

| Status | Item |
| --- | --- |
| ⬜ | Vault discovery (directory of `.md` files) |
| ⬜ | Note identity by filename stem |
| ⬜ | Shortest-unique-path resolution (Obsidian model) |
| ⬜ | Frontmatter aliases in the resolution index |
| ⬜ | File-tree sidebar |

## 6. Links and backlinks

| Status | Item |
| --- | --- |
| ⬜ | Wikilinks: `[[Note]]`, `[[Note\|alias]]`, `[[Note#heading]]` |
| ⬜ | Markdown links indexed for interop |
| ⬜ | Dangling-link highlight + follow-creates note |
| ⬜ | Forward link index + backlinks panel |
| ⬜ | Neighborhood explorer |

## 7. Tags and search

| Status | Item |
| --- | --- |
| ⬜ | Inline `#tags` + frontmatter `tags:` (unified) |
| ⬜ | Hierarchical tags (`#parent/child`) |
| ⬜ | Fuzzy name switcher (stems + aliases) |
| ⬜ | Regex / substring content search with context |

## 8. UI compositor

| Status | Item |
| --- | --- |
| ⬜ | Surface tree (splits) + overlay popups |
| ⬜ | Editor-centric layout (sidebars for tree / backlinks) |
| ⬜ | Focus cycling across panes |

## 9. Robustness and config

| Status | Item |
| --- | --- |
| ⬜ | External file watch (`notify`) + debounced re-index |
| ⬜ | Clean auto-reload / dirty prompt on external change |
| ⬜ | TOML config (global + vault-local): theme, tab width, line numbers |
| ⬜ | GraphViz `.dot` export |

## Out of scope for v1

- Windows support (emoji-width / ConPTY)
- Visual force-directed graph view
- Block refs / embeds
- Full-text inverted index (`tantivy`)
- Undo history persistence across sessions
- Keymap loading from TOML
- Three-way merge for dirty external edits

---

## Current focus

**Next up:** selection-first editing (Select mode + select-then-act), then open/save so the editor can work on real vault files.

Update this file when a checklist item lands. Prefer an atomic commit that only
touches the roadmap when nothing else changes.
