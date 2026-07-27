# manotz

Terminal-first knowledge management editor in Rust.

Helix/Kakoune-style **selection-first** modal editing plus an Obsidian-compatible markdown vault (wikilinks, tags, backlinks, search) — all in the terminal. Full product vision lives in [issue #1](https://github.com/emadbaqeri/manotz/issues/1).

> **Status:** early / pre-1.0. The modal editor core through undo/redo (milestones M1–M5) is in place. Vault and knowledge-layer features are not shipped yet.

## Features (today)

- Gap-buffer text storage behind a `Buffer` trait
- Grapheme-aware cursor / insert / backspace
- Normal + Insert modes
- Pure render core + crossterm adapter (diffed cell grid)
- Undo tree with merge-window for consecutive inserts (`u` / `U`)

## Requirements

- macOS or Linux (Windows deferred for v1)
- Rust **1.88+** (see `rust-version` in `Cargo.toml`)

## Build & run

```bash
cargo run
```

| Key | Mode | Action |
|---|---|---|
| `i` | Normal | Enter Insert |
| `Esc` | Insert | Enter Normal |
| arrows | both | Move |
| printable / Enter / Backspace | Insert | Edit |
| `u` / `U` | Normal | Undo / Redo |
| `q` | Normal | Quit |

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Project layout

| Module | Role |
|---|---|
| `text` | Grapheme / display-width helpers |
| `buffer` | `Buffer` trait + gap buffer |
| `selection` | Selections + selection set |
| `command` | Motions, edits, transactions |
| `history` | Undo tree + merge window |
| `render` | Pure grid + crossterm adapter |
| `input` | Modes + keymap |
| `editor` | `EditorState` + `update` |

## Roadmap

See the suggested build order in [issue #1](https://github.com/emadbaqeri/manotz/issues/1) (M6 markdown → vault → links → compositor → search → watch → config).

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

See [SECURITY.md](SECURITY.md) for how to report vulnerabilities.

## License

Licensed under the [MIT License](LICENSE).
