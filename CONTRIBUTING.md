# Contributing

The project is one crate: a ratatui binary plus a small library the tests use.

## Setup

```bash
git clone https://github.com/feliperafaldini/cheatrs.git
cd cheatrs
cargo run            # debug build, opens the app
cargo test           # 43 tests
```

Stable Rust. No services, no database, no environment variables required.

## Before opening a pull request

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs exactly these three plus a release build that validates the embedded data.
A pull request that skips them will go red for the same reasons.

## Adding a topic

1. Copy `data/template.json` to `data/topics/<slug>.json`.
2. Keep `schema: 1`. The format is documented in [`docs/topic-format.md`](docs/topic-format.md).
3. Outline the sections yourself: what a reader needs in that order, not what the
   official manual happens to list first.
4. One to three snippets per section. The comment inside the code carries the
   explanation; the `description` field stays one line.
5. Validate:

```bash
./target/debug/cheatrs --validate data/topics/<slug>.json
python3 tools/check_snippets.py
```

A topic that fails validation is not merged. A snippet whose language has no checker
is accepted, but say in the pull request which ones those are.

## What gets rejected

- **Unsourced claims.** "Blazing fast", "production ready", a benchmark without a
  script. Replace with something a reader can check.
- **Snippets that do not run.** Shell snippets are syntax-checked; SQL is parsed;
  Python is compiled. A snippet with an invented flag is a bug, not a typo.
- **Copied text.** The cheatsheets are written for this project. Text lifted from
  another cheatsheet project brings its licence with it.
- **Scope creep.** A pull request that fixes one thing and reformats three files
  will be sent back.

## Commits

Imperative subject, a body that says why rather than what, and one logical change
per commit. `git log` is the changelog's raw material.

## Licence

MIT. By contributing you agree your contribution ships under it.
