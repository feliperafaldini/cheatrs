# Changelog

Newest first. The format follows [Keep a Changelog](https://keepachangelog.com/);
versions follow [semantic versioning](https://semver.org/).

## 1.0.0

First stable release. The topic format and the command line are frozen: schema 1
and the flags `--help`, `--version`, `--validate` will not change meaning.

### Added

- `?` opens a key map inside the app, explaining the four columns and the keys the
  footer does not have room for.
- `packaging/`: `cheatrs-bin` PKGBUILD for Arch, installing the static release binary, with the source-build problem documented.

### Changed

- Nothing that breaks 0.2.0. A topic written for 0.2.0 loads unchanged.

## 0.2.0

### Added

- Eight topics: SQL, Kubernetes, GitHub Actions, Nginx, Jest/Vitest, shell tools
  (jq, fzf, ripgrep, fd, bat, xargs), Flutter and R. 60 topics, 833 sections.
- `--version` and `-V`.
- Search tolerates a typo: `pstgrs` reaches PostgreSQL by matching letters in order,
  ranked below an exact substring so a correct query never loses to a loose one.
- Live minimum-size warning: shrinking the terminal below 80x24 shows the message
  instead of drawing a broken screen, and recovers when it grows back.
- Release assets for macOS arm64, macOS x86_64 and Windows x86_64, next to the
  static Linux binary.
- `docs/topic-format.md`: the topic format as a contract, with `schema: 1`.
- This changelog.

### Changed

- Column widths are measured from the library at startup instead of guessed; at 132
  columns the reading column gained 11 cells.
- The section index sits to the right of the text, and the four columns are groups,
  topics, text, contents.
- The header carries the version, a rule and the size of the library.
- Install lines cover apt, dnf, pacman and apk; `redis` documents that Arch and
  Fedora now ship `valkey`.

### Fixed

- Running with stdin or stdout redirected panicked inside the terminal library;
  it now prints one line and exits 1.
- A user topic that fails to parse is counted and shown as `· N skipped`.
- `django/deployment` shipped `raw_env` with an unclosed bracket.
- `bash/parameter-expansion` documented `${file#report-}` as `20`; it is `2026.tar.gz`.
- `motion` had three snippets that were statements plus a bare JSX element, invalid
  if pasted; they are components now.

## 0.1.1

### Added

- Prebuilt static Linux binary and `install.sh` with sha256 verification.
- CI: formatting, lints, tests and a release smoke test.

## 0.1.0

First release.

- Two columns to find a topic, one to preview, a full-screen reader with syntax
  highlighting.
- Search over topic titles, section titles and code inside snippets, with aliases.
- Topics written by the user in `$XDG_DATA_HOME/cheatrs/topics/`, checked by
  `--validate`.
- 52 topics, read-only by design: no copy, no execution, no network.
