# manotz — Project Context

## What is this?
Terminal-first knowledge management editor in Rust. Helix/Kakoune selection-first modal editor + Obsidian-compatible markdown vault. Issue #1 on GitHub has the full PRD. Edition 2024.

## Current state
- **Milestone**: §6 in progress — Dangling link highlighting & vault-aware link resolution
- **Tests**: 183 lib unit tests + 12 MSRV; all green
- **Done**: `HighlightKind::DanglingLink`, `style_for(DanglingLink)` (amber: 200, 150, 100), `highlight_with_vault` (wikilinks + internal/external markdown links resolved against `VaultIndex`)

### Shipped earlier (still true)
- Editor foundation through history, select-then-act, open/save/dirty, modal keymap, pure render + crossterm adapter
- Session persistence (`.manotz/state.json`) with cursor restoration
- Vault discovery, shortest-unique-path resolution, frontmatter aliases index
- Markdown AST highlighting + live rendering integration

### Done this stretch — dangling-link highlighting (§6)
- `src/markdown/mod.rs`:
  - `HighlightKind::DanglingLink` added
  - `style_for(DanglingLink) -> Style` (amber `Colour::Rgb(200, 150, 100)`)
  - `pub fn highlight_with_vault(text: &str, vault: Option<&VaultIndex>) -> Vec<Highlight>`
    - Integrates standard markdown elements (`Heading`, `Emphasis`, `Bold`, `Code`)
    - Integrates `extract_note_links(text)`; links remain `Link` without a vault, and unresolved targets become `DanglingLink` when a vault is available
    - Integrates external markdown links from `parse_markdown_links(text)` as `HighlightKind::Link`
  - `pub fn highlight(text: &str)` defaults to `highlight_with_vault(text, None)`
- `src/vault/mod.rs`:
  - `pub(crate) aliases` on `VaultIndex` to allow crate-internal construction/testing
- Tests: added `style_for_dangling_link_is_amber`, `highlight_internal_link_without_vault_is_link`, `highlight_wikilink_without_vault_is_link`, and `highlight_distinguishes_resolved_and_dangling_links` (183 unit tests passing)

### Next session — pick up here
1. ~~**Wire `VaultIndex` into `main.rs` live loop**~~ ✅ (PR #36)
   - `vault: Option<VaultIndex>` added to `EditorState`
   - Built from `current_dir()` on startup, `.ok()` for graceful degradation
   - Passed to `highlight_with_vault` in render loop via `.as_ref()`
2. **Follow link / Create note action (`Action::FollowLink` / `gd` / `Enter`)**:
   - First: `link_at_cursor(text, cursor) -> Option<NoteLink>` — pure function, TDD
   - Detect if cursor is inside a `NoteLink` span
   - If target exists in vault → switch active file to target
   - If dangling → create file `target.md` in vault root and switch to it
   - Add `Action::FollowLink` variant + keymap binding (`gd` in Normal mode)
3. Update roadmap checklist (§6) when follow-link is complete.

### Helix formatter note
- Helix `rustfmt` via stdin ignores Cargo edition; let-chains need  
  `formatter = { command = "rustfmt", args = ["--edition", "2024"] }` in `~/.config/helix/languages.toml`  
  (`cargo fmt` already OK with edition 2024)

## Next steps (M5 — history)
1. ~~Record edits in a history structure (transactions / inverse edits).~~
2. ~~Undo restores buffer + selections.~~
3. ~~Redo (move to newest child + apply).~~
4. ~~Redo returns post-edit selections.~~
5. ~~Branching: undo then record → new child; redo picks most-recent child.~~
6. ~~Merge-window: consecutive Insert merges within 200ms via `Transaction::coalesce_insert`.~~
7. ~~Wire undo/redo into EditorState + keymap (`u` / `U`).~~ Undo + redo wired: `u`/`U` in Normal; `InsertChar`/`Backspace` record via `history.record`; both arms restore selections + viewport-follow.

## Done (text module)
| Function | Tests | File |
|---|---|---|
| `grapheme_len(s) -> usize` | 4 | `src/text/mod.rs` |
| `grapheme_width(g) -> usize` | 4 | `src/text/mod.rs` |
| `display_width(s) -> usize` | 0 (covered by grapheme_width) | `src/text/mod.rs` |
| `grapheme_to_byte_offset(s, idx) -> usize` | 4 | `src/text/mod.rs` |
| `byte_to_grapheme_offset(s, byte) -> usize` | 5 | `src/text/mod.rs` |

## Done (render module — pure core)
M3 = render loop. Two layers, build the **pure core first**, then the IO adapter:
1. **Pure core** (deep, terminal-free, all tests live here): `Cell`, `Grid`, `Viewport`, `Style`, `render` fn, `diff` fn. ✅ done
2. **crossterm adapter** (IO-bound): raw mode, double-buffer diff, stdout. ✅ done

| Component | Status | File |
|---|---|---|
| `Colour` enum (Rgb, Copy/Clone/PartialEq/Debug) | ✅ done | `src/render/mod.rs` |
| `Style` struct (bold, fg, bg + Default/Clone/PartialEq/Debug) | ✅ done | `src/render/mod.rs` |
| `Cell { grapheme: String, style: Style }` + accessors + PartialEq | ✅ done | `src/render/mod.rs` |
| `Grid` (row-major `Vec<Cell>` + `width`); `new`, `width`/`height`, `cell`/`set_cell`/`set_style`, private `cell_index` | ✅ done | `src/render/mod.rs` |
| `Viewport { top, left, rows, cols }` + fail-fast guards | ✅ done | `src/render/mod.rs` |
| pure `render(Buffer, SelectionSet, Viewport) -> Grid` (text + vertical/horizontal scroll + padding + cursor) | ✅ done | `src/render/mod.rs` |
| `render` wide-char width correctness (CJK/emoji = 2 cols, grapheme-walking loop) | ✅ done | `src/render/mod.rs` |
| `byte_to_line_col(text, offset) -> Option<(line, col)>` | ✅ done | `src/render/mod.rs` |
| `diff(prev: &Grid, curr: &Grid) -> Vec<(row, col, &Cell)>` (double-buffer diff, pure) | ✅ done | `src/render/mod.rs` |
| crossterm adapter (raw mode, double-buffer diff, stdout) | ✅ done | `src/render/adapter.rs` |

## Next steps (M4 — input + event loop)
1. Read keypresses via `crossterm::event::poll`/`read`. ✅
2. Map keys to commands (movements: left/right/up/down → cursor moves). ✅
3. Event loop: read key → update state → re-render → diff → draw. ✅
4. Scroll-on-cursor: viewport follows cursor when it reaches edges. ✅
5. Modes (Normal/Insert) + insert/backspace/newline. ✅
→ **M4 done. See Current state for M5.**

## Architecture decisions
- **TDD**: Red-green-refactor, one behavior at a time. Tests assert behavior through public interfaces. No code without a test that forces it (YAGNI).
- **M1 order**: `text` before `buffer` — text helpers are pure math, buffer depends on them.
- **Test naming**: `function_name_scenario` pattern (e.g., `grapheme_to_byte_offset_combining`).
- **Module structure**: Each deep module gets its own directory (`src/text/`, `src/buffer/`, etc.).
- **Tests inline**: Unit tests live in `#[cfg(test)] mod tests` inside the module file.
- **Buffer trait**: Byte-offset interface. Start with generics, defer `dyn Buffer`.
- **GapBuffer**: Currently a `String` wrapper — true gap buffer deferred.
- **Render = two layers**: pure core (`Cell`/`Grid`/`Viewport`/`render` fn, terminal-free, all tests here) before the crossterm adapter (IO-bound). The `Grid` is the seam between them.
- **Grid storage**: flat `Vec<Cell>` in row-major order (`index = row * width + col`); store `width`, derive `height = cells.len() / width`. One allocation + cache-friendly (vs `Vec<Vec<Cell>>`).
- **Fail-fast construction**: guard `Grid::new` against 0 dimensions (make illegal states unrepresentable), not defensive checks downstream.
- **Per-cell allocation**: known future optimization. `String`-backed `Cell` is the simple obvious-correct choice now; switch to inline stack storage once the render loop exists and `cargo bench` points to it. Do NOT optimize on intuition.
- **EditorState owns all state**: `buffer`, `selections`, `viewport`, `mode` live in one struct. `update(self, Action) -> Self` consumes and returns new state. Event loop in `main.rs` owns the state and drives the cycle.
- **Cursor rendering**: cell-style only (gray bg on cursor cell). Hardware cursor explicitly hidden via `crossterm::cursor::Hide`.
- **Scroll logic location**: Scroll adjustment in `EditorState::update` after motion/edit computes new cursor position. Pure logic, no IO, fully testable.
- **Mode-aware keymap**: `map_key(key, mode)`; Insert maps printable→`InsertChar`, Enter→`InsertChar('\n')`, Backspace, arrows, Esc; Normal maps motions, `i`, `q`.
- **Coordinate helpers**: `byte_to_line_col` and `line_col_to_byte` live in `render` (paired); motions import from render.

## User context
- Rust beginner (traits/associated types/lifetimes: learning just-in-time)
- No TUI experience
- Learning TDD alongside the project
- Mentoring preferred for learning stretches; will sometimes ask to exit mentor mode and implement directly
- Mentor prefs when mentoring: teach concepts from scratch; clear near-complete Rust skeletons with 1–3 labeled gaps
- Helix: rustfmt needs `--edition 2024` for let-chains on stdin format
- Optional polish: scroll viewport after `restore_cursor` if caret is off-screen

## Durable knowledge
- Clippy `iter_nth_zero`: use `.next()` not `.nth(0)`
- CRLF (`\r\n`) = 1 grapheme cluster (UAX #29)
- `str::len()` returns byte count, not character count
- Combining characters are large in bytes (e.g., `a̐` = 4 bytes)
- `grapheme_width` can call `.width()` directly on a `&str` — trust the contract
- `str` is unsized — use `String` in structs, `&str` in function params
- `.to_owned()` on `&str` → `String`, no lifetime annotation needed
- `new()` constructors take `&str` not `String` — more flexible
- `len()` should compute from data, not store redundant field
- Clippy `len_without_is_empty`: provide `is_empty()` default in traits
- `usize`/`u8`/`u32`/... are unsigned (≥0); never compare `< 0` — Clippy `absurd_extreme_comparisons`. Signed = `i8`/`i16`/`i32`/`isize`.
- A terminal cell is never empty: a blank cell holds a space `" "` (U+0020), not `""`. Space fills the cell with bg color, draws no glyph.
- `char` = one Unicode code point, NOT a grapheme cluster — can't hold combining marks/emoji/CRLF. A grapheme field must be `String`/`&str`.
- `vec![value; n]` clones n times → requires `T: Clone` (`#[derive(Clone)]` works if all fields are `Clone`).
- Hot-path accessors borrow, not clone: `cell()->&Cell`, `cells()->&[Cell]`, `grapheme()->&str`. A `&self` method returning a reference has its lifetime elided automatically.
- `#[should_panic]` makes a test pass only when the body panics — use it to test preconditions/guards.
- "Make illegal states unrepresentable": reject invalid input at the constructor (`assert!`/`panic!`) rather than handling it defensively later.
- Compiler hints are hints, not commands — e.g., `&Vec` mismatch says "remove the borrow" but that moves out of `&self`; fix the return type instead.
- Clone vs construct = same allocation count for `String`-backed cells. The villain is per-cell heap allocation; the fix is inline storage (deferred — measure before optimizing).
- `panic!` returns `!` (never type — diverges), so a missing `;` after it compiles, but keep `;` consistent in pairs.
- `saturating_sub` clamps to 0 on underflow — wrong for bounds checks (off-screen cursor wraps to 0). Use `checked_sub` (returns `Option<usize>`, `None` on underflow) to detect "before the viewport".
- Let-chains (`if let Pat = expr && condition`) — stable since Rust 1.88. Collapses nested `if let` + `if` into one. Clippy `collapsible_if` suggests it.
- Borrow checker can't see "through" method calls in a single expression: `self.cells[self.cell_index(r,c)]` fails because both borrow `self`. Extract the index to a local first: `let i = self.cell_index(r,c); self.cells[i] = ...`.
- Struct update syntax: `Style { bg: Some(...), ..Style::default() }` — avoids listing every field; `..Default::default()` fills the rest.
- Flat index → (row, col): `row = index / width`, `col = index % width`. Don't mix width/height in the formula.
- Lifetime on `diff`: when a function returns references from two inputs but only borrows one, use a single `'a` (simpler) or two lifetimes `'a`/`'b` (more precise — output tied to the borrowed input only).
- `Drop` trait runs code when a value goes out of scope — even during panic unwinding. This is Rust's `finally`/`defer`. Use a RAII guard struct to guarantee cleanup (e.g., restoring terminal raw mode).
- `Drop::drop` returns `()` — you CANNOT use `?` inside it. Use `.expect()` for cleanup failures (e.g., `disable_raw_mode().expect(...)`).
- Error propagation: return `Result<T, E>` and use `?` to bubble errors to the caller. `io::Result<T>` is a type alias for `Result<T, io::Error>` (error type pre-pinned). `queue!` and `flush()` return `io::Result` — propagate with `?`.
- Error handling spectrum: (1) `Result<T, E>` + `?` for one error source, (2) `anyhow::Result` for application code with many error sources, (3) `thiserror` + custom `enum Error` for library code where callers match on error kinds. Pick based on need — YAGNI.
- `Option<T>` implements `Default` as `None` — so `#[derive(Default)]` on a struct with `Option<T>` fields initializes them to `None`.
- Terminal adapter must reset colors (`Color::Reset`) before setting each cell's bg/fg — otherwise colors leak from the previous cell.
- `crossterm::event::read()` returns `Event`, not `KeyEvent`. Match on `Event::Key(k)` to extract the key, ignore `Mouse`/`Resize`.
- `update(&self, Action) -> Self` (borrows) vs `update(self, Action) -> Self` (consumes). Use `&self` when the caller needs to keep the state alive (e.g., event loop: render borrows state, then update must also borrow it).
- `event::read()` blocks until a key is pressed — no `poll()` needed for a simple loop. `poll()` is for when you need timeouts or non-blocking reads.
- Raw mode hides the terminal cursor by default in some terminals — must explicitly `Show`/`Hide` crossterm cursor or handle via cell styling.
- crossterm `Hide`/`Show` are commands, not methods — must be passed to `queue!` or `execute!` macro
- `byte_to_line_col` lives in `src/render/mod.rs` — editor imports it from render for scroll logic

## Dependencies
- `unicode-segmentation = "1.13.3"`
- `unicode-width = "0.2.2"` (inferred from usage)
- `crossterm = "0.29.0"`
