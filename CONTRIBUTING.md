# Contributing to manotz

Thanks for your interest in contributing. The project is early; small, focused changes are easiest to review.

## Before you start

1. Read the product vision in [issue #1](https://github.com/emadbaqeri/manotz/issues/1).
2. Prefer opening an issue for design discussion before large work.
3. Keep mentor/private notes (`CONTEXT.md`, local agent files) out of PRs — they are gitignored on purpose.

## Development setup

```bash
git clone https://github.com/emadbaqeri/manotz.git
cd manotz
cargo test
cargo run
```

## How we work

- **TDD for core logic:** add or update a failing test, then implement the minimal fix. Tests assert behavior through public APIs, not private internals.
- **Pure cores first:** editor logic stays free of terminal I/O so it stays unit-testable. Put crossterm / filesystem code behind adapters.
- **One concern per PR:** prefer a vertical slice or a single bugfix over mixed refactors.
- **Style:** run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` before pushing. CI enforces the same.

## Pull requests

1. Fork (or branch from `master` if you have write access).
2. Make your change with tests.
3. Open a PR with a short summary and how you tested it.
4. Keep the PR focused; link related issues.

### Commit and PR wording

Write for someone who has never seen this repo:

- Prefer plain descriptions of **what changed and why** (e.g. “Allow the cursor past the last character so insert works at end of line”).
- Avoid internal shorthand (`M5`, “milestone 3”, ticket slang) unless the full phrase is also spelled out.
- PR titles and bodies should make sense on their own without reading the roadmap first.

### Atomic commits

Keep commits **atomic**: one logical change per commit.

- Do: “Add undo/redo keybindings in normal mode” as its own commit; “Add MIT license and README” as another.
- Don’t: mix unrelated edits (feature + formatting + docs + refactor) in one commit.
- Each commit should leave the tree buildable (`cargo test` / CI green) when practical.
- Commit subject: imperative, ~50 characters when you can; optional body for *why*.
- Prefer several small commits (or a PR made of small commits) over one large “everything” commit.

## Code of Conduct

By participating, you agree to uphold the [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions

Open a GitHub issue. For security reports, use [SECURITY.md](SECURITY.md) instead of a public issue.
