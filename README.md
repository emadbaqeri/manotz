<p align="center">
  <h1 align="center">manotz</h1>
  <p align="center"><em>terminal-first knowledge management</em></p>
</p>

<p align="center">
  Helix/Kakoune-style selection-first editing meets an Obsidian-compatible
  markdown vault — written in Rust.
</p>

<p align="center">
  <a href="https://github.com/emadbaqeri/manotz/actions/workflows/rust.yml"><img src="https://img.shields.io/github/actions/workflow/status/emadbaqeri/manotz/rust.yml?style=flat-square&label=Rust" alt="Rust CI" /></a>
  <a href="https://github.com/emadbaqeri/manotz/actions/workflows/security.yml"><img src="https://img.shields.io/github/actions/workflow/status/emadbaqeri/manotz/security.yml?style=flat-square&label=Security" alt="Security" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT" /></a>
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/rust-1.97.1-orange.svg?style=flat-square" alt="Rust 1.97.1" /></a>
  <a href="https://doc.rust-lang.org/edition-guide/"><img src="https://img.shields.io/badge/edition-2024-black.svg?style=flat-square" alt="Edition 2024" /></a>
</p>

---

## About

`manotz` keeps writers in the terminal with a powerful modal editor **and** a
knowledge layer: `[[wikilinks]]`, tags, backlinks, fuzzy navigation, and vault
search — without shelling out to `$EDITOR` or leaving for a GUI.

**Status:** early / pre-1.0. The modal editor core works today. Vault features
are tracked in [ROADMAP.md](ROADMAP.md).

## Features

**Available now**

- Gap-buffer text storage behind a swappable `Buffer` trait
- Grapheme-aware cursor, insert, and backspace
- Normal and Insert modes with a mode-aware keymap
- Pure render core + crossterm adapter (diffed cell grid)
- Branching undo tree with merge-window for consecutive inserts (`u` / `U`)
- Viewport that follows the cursor on all edges

**Coming next** — see [ROADMAP.md](ROADMAP.md)

- Select mode and select-then-act editing
- Open / save files
- Markdown highlighting, vault indexes, wikilinks, backlinks, search

## Supported platforms

| Platform | Status |
| --- | --- |
| macOS | supported |
| Linux | supported |
| Windows | deferred for v1 |

## Quick start

```bash
git clone https://github.com/emadbaqeri/manotz.git
cd manotz
cargo run
```

Requires the toolchain in [`rust-toolchain.toml`](rust-toolchain.toml) (Rust **1.97.1**).
[`rustup`](https://rustup.rs) will install it automatically.

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
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo deny check
```

## Project layout

| Path | Role |
| --- | --- |
| [`src/text`](src/text) | Grapheme / display-width helpers |
| [`src/buffer`](src/buffer) | `Buffer` trait + gap buffer |
| [`src/selection`](src/selection) | Selections + selection set |
| [`src/command`](src/command) | Motions, edits, transactions |
| [`src/history`](src/history) | Undo tree + merge window |
| [`src/render`](src/render) | Pure grid + crossterm adapter |
| [`src/input`](src/input) | Modes + keymap |
| [`src/editor`](src/editor) | `EditorState` + `update` |
| [`ROADMAP.md`](ROADMAP.md) | Progress checklist |
| [`.github/workflows`](.github/workflows) | Rust + Security CI |

## Documentation

- [Roadmap](ROADMAP.md) — what is done and what is next
- [Contributing](CONTRIBUTING.md) — how to develop and open PRs
- [Security policy](SECURITY.md) — private vulnerability reports
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Product vision (issue #1)](https://github.com/emadbaqeri/manotz/issues/1)

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the [MIT License](LICENSE).
