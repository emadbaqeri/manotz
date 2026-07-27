# Contributing to manotz

Thanks for your interest in contributing. The project is early; small, focused changes are easiest to review.

## Before you start

1. Skim [ROADMAP.md](ROADMAP.md) for what is done and what is next.
2. Read the product vision in [issue #1](https://github.com/emadbaqeri/manotz/issues/1) for background decisions.
3. Prefer opening an issue for design discussion before large work.
4. Keep mentor/private notes (`CONTEXT.md`, local agent files) out of PRs — they are gitignored on purpose.

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
- **One concern per PR:** prefer a vertical slice or a single bug fix over mixed refactors.
- **Style:** run `cargo fmt --all --check` and `cargo clippy --locked --all-targets -- -D warnings` before pushing. CI covers formatting, clippy, tests (Linux + macOS), MSRV, docs, typos, and commitlint. The [security workflow](.github/workflows/security.yml) runs `cargo-deny`, `cargo-audit`, CodeQL, dependency review (PRs), and zizmor.

## Pull requests

1. Fork (or branch from `master` if you have write access).
2. Make your change with tests.
3. Open a PR with a short summary and how you tested it.
4. Keep the PR focused; link related issues.
5. PR titles should also follow the Conventional Commits format below when practical.

## Commit messages

We follow **[Conventional Commits](https://www.conventionalcommits.org/)** as enforced by
[`@commitlint/config-conventional`](https://github.com/conventional-changelog/commitlint/tree/master/@commitlint/config-conventional)
(see also the [commitlint rules](https://commitlint.js.org/reference/rules.html)).

### Format

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Rules that matter here

- **type** (required): one of `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`
- **scope** (optional): lowercase area of the code, e.g. `feat(editor): …`, `fix(history): …`
- **description**: imperative, lowercase, no trailing period — readable without project jargon
- **header**: keep under 100 characters
- **body / footer**: optional; separate from the header with a blank line
- **breaking change**: add `!` after type/scope (`feat(api)!: …`) and/or a `BREAKING CHANGE:` footer

### Examples

```text
feat(text): add grapheme and display-width helpers

fix(editor): allow cursor past last character for end inserts

docs: add security policy for private vulnerability reports

ci: add format, lint, and test workflow

refactor(history): use question-mark operator in undo redo
```

### Atomic commits

Keep commits **atomic**: one logical change per commit.

- Do: `feat(input): map u and U to undo and redo` as its own commit
- Don’t: mix unrelated feature, docs, and refactor work in one commit
- Each commit should leave the tree buildable (`cargo test` / CI green) when practical
- Prefer several small commits (or a PR made of small commits) over one large “everything” commit

Write for someone who has never seen this repo: plain language, no bare milestone shorthand.

CI validates commit messages on pushes and pull requests with commitlint.

## Code of Conduct

By participating, you agree to uphold the [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions

Open a GitHub issue. For security reports, use [SECURITY.md](SECURITY.md) instead of a public issue.
