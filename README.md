<div align="center">

# manotz

**Terminal-first knowledge management** — Helix/Kakoune-style selection-first
editing meets an Obsidian-compatible markdown vault, all in Rust.

[![CI](https://github.com/emadbaqeri/manotz/actions/workflows/ci.yml/badge.svg)](https://github.com/emadbaqeri/manotz/actions/workflows/ci.yml)
[![Security](https://github.com/emadbaqeri/manotz/actions/workflows/security.yml/badge.svg)](https://github.com/emadbaqeri/manotz/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-black.svg)](https://doc.rust-lang.org/edition-guide/)

</div>

## About

`manotz` aims to keep writers in the terminal with a powerful modal editor **and**
a knowledge layer: `[[wikilinks]]`, tags, backlinks, fuzzy navigation, and vault
search — without shelling out to `$EDITOR` or leaving for a GUI.

**Status:** early / pre-1.0. The modal editor core (text, buffer, selections,
render, modes, undo/redo) works today. Vault and knowledge features are on the
[roadmap](ROADMAP.md).

## Features

**Available now**

- Gap-buffer text storage behind a swappable `Buffer` trait
- Grapheme-aware cursor, insert, and backspace
- Normal and Insert modes with a mode-aware keymap
- Pure render core + crossterm adapter (diffed cell grid)
- Branching undo tree with merge-window for consecutive inserts (`u` / `U`)
- Viewport that follows the cursor on all edges

**Coming next** (see [ROADMAP.md](ROADMAP.md))

- Select mode and select-then-act editing
- Open / save files
- Markdown highlighting, vault indexes, wikilinks, backlinks, search

## Requirements

- **Rust** 1.88 or newer (`rust-version` in `Cargo.toml`)
- **macOS** or **Linux** (Windows support deferred for v1)

## Quick start

```bash
git clone https://github.com/emadbaqeri/manotz.git
cd manotz
cargo run
```

### Keybindings

| Key | Mode | Action |
| --- | --- | --- |
| `i` | Normal | Enter Insert |
| `Esc` | Insert | Enter Normal |
| arrows | both | Move |
| printable / Enter / Backspace | Insert | Edit |
| `u` / `U` | Normal | Undo / Redo |
| `q` | Normal | Quit |

### Development checks

```bash
cargo test --locked
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo deny check
```

CI also runs typos, docs (`-D warnings`), MSRV, commitlint, and the
[security workflow](.github/workflows/security.yml).

## Project layout

| Module | Role |
| --- | --- |
| [`text`](src/text) | Grapheme / display-width helpers |
| [`buffer`](src/buffer) | `Buffer` trait + gap buffer |
| [`selection`](src/selection) | Selections + selection set |
| [`command`](src/command) | Motions, edits, transactions |
| [`history`](src/history) | Undo tree + merge window |
| [`render`](src/render) | Pure grid + crossterm adapter |
| [`input`](src/input) | Modes + keymap |
| [`editor`](src/editor) | `EditorState` + `update` |

## Roadmap

Track progress in **[ROADMAP.md](ROADMAP.md)**. Product background and decisions
live in [issue #1](https://github.com/emadbaqeri/manotz/issues/1).

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and
follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Please report vulnerabilities privately — see [SECURITY.md](SECURITY.md).
Dependency and workflow checks run in CI via [security.yml](.github/workflows/security.yml).

## License

Licensed under the [MIT License](LICENSE).
