# cheatrs

[![ci](https://github.com/feliperafaldini/cheatrs/actions/workflows/ci.yml/badge.svg)](https://github.com/feliperafaldini/cheatrs/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/feliperafaldini/cheatrs)](https://github.com/feliperafaldini/cheatrs/releases)
[![licence](https://img.shields.io/github/license/feliperafaldini/cheatrs)](LICENSE)

A terminal browser for programming cheatsheets: 52 topics, 773 sections, 1860
snippets, one 3 MB binary that never touches the network.

It opens as a program, not as a command you pass a topic to. You land on a list of
languages, frameworks, tools and databases, walk it with the keyboard, and read the
chosen topic full screen with syntax highlighting.

```
cheatrs  52 topics · 0 skipped
▌ LANGUAGES           │▌ SH Bash
  FRAMEWORKS          │   C C
  LIBRARIES           │  C# C#
  TOOLS               │ C++ C++
  DATABASES           │ CSS CSS
  RUNTIMES            │  DT Dart
                      │  GO Go
                      │  HT HTML
                      │  JV Java
                      │  JS JavaScript
                      │  KT Kotlin
                      │ LUA Lua
                      │ PHP PHP
                      │  PY Python
                      │  RB Ruby
                      │  RS Rust
                      │  SW Swift
                      │  TS TypeScript
hjkl/←→↑↓ move · ⇥/l column · ↵ open list · / search · M mouse off · q quit  1/6
```

`Enter` on a topic opens the reader: the topic's sections on the left, the code
highlighted on the right.

```
‹ LANGUAGE / Bash
Bash                                                                                               █
The Bourne Again Shell: interactive command line and scripting language.                           ▏
                                                                                                   ▏
Setup                                                                                              ▏
Check the shell, its version and how a script starts.                                              ▏
                                                                                                   ▏
  # Check which Bash is in use and its version                                                     ▏
  bash --version | head -n 1                                                                       ▏
  echo "$BASH_VERSION"                                                                             ▏
                                                                                                   ▏
  # Start a login shell or a plain interactive subshell                                            ▏
  bash -l                                                                                          ▏
  bash --norc --noprofile                                                                          ▏
                                                                                                   ▏
↑↓ scroll · g/G top/bottom · q back
```

## Install

No Rust needed — the release ships a static x86-64 binary that runs on any Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/feliperafaldini/cheatrs/main/install.sh | bash
```

The script downloads the binary from the latest release, verifies its sha256 and
installs it to `~/.local/bin`. `INSTALL_DIR` changes the destination, `VERSION=0.1.1`
pins a release instead of the latest.

If you already have Rust, the source path works too:

```bash
cargo install --path .
cheatrs
```

Both were verified from a clean state: a single binary, no data files, no service, no
network at runtime. The terminal must be at least 80x24 — below that it prints the
size it found and exits instead of drawing a broken screen. The preview column
appears from 110 columns; the section list inside the reader from 120.

## Why not tldr, navi or cheat.sh

They are not worse; they answer a different question.

| | cheatrs | tldr | navi | cheat.sh |
|---|---|---|---|---|
| Shape | browser you enter | one page per command | picker you call | HTTP endpoint |
| Navigation | columns, keyboard | `tldr tar` | fuzzy search | `curl cht.sh/tar` |
| Offline | yes, content in the binary | after `tldr --update` | after adding cheats | no |
| Runs commands | never | never | yes, by design | never |
| Content written here | all of it | community, one-liners | community | community |

Use **tldr** when you want one command's flags inline in the shell and nothing else.
Use **navi** when you want the cheatsheet to execute the snippet. Use **cheatrs**
when the problem is that you half-remember a language or a tool and want to read the
thing properly, not grep for one flag.

## Use

| Key | Does |
|---|---|
| `j` `k` or `↓` `↑` | move in the focused column; scroll the document |
| `l` `→` `Tab` | next column: groups → topics → preview → document |
| `h` `←` `Esc` | previous column, on the home screen |
| `Enter` | in groups: step into the list; in topics and preview: open the document |
| `Tab` | inside the reader: focus the section list; `Enter` or `Esc` leaves it |
| `g` `G` | first and last line, inside the reader |
| `/` | search |
| `M` | toggle mouse capture |
| `q` `←` `Esc` | inside the reader: back to the home screen; `q` on the home screen quits |

Mouse is off by default: every screen is reachable from the keyboard alone.

## Search

One index over topic titles, section titles and the code inside every snippet.
Ranking goes title prefix, title match, the section's topic, then code. A code hit
carries the matching line, and `Enter` opens the reader at that section.

`data/aliases.json` maps what you type to what the titles spell — `psql` reaches
PostgreSQL, `ts` TypeScript, `py` Python, `nvim` Vim. An alias that matches nothing
falls back to the literal text.

## Your own topics

Anything in `$XDG_DATA_HOME/cheatrs/topics/*.json` is loaded over the embedded base
and wins by `slug`. `data/template.json` shows the shape; `--validate` tells you what
is wrong instead of silently skipping the file:

```bash
cheatrs --validate                 # every topic the binary can see
cheatrs --validate my-topic.json   # a single file
```

A topic needs `slug`, `title`, `label`, `group`, `color` and at least one section
with a snippet. Groups in use: `Language`, `Framework`, `Library`, `Tool`,
`Database`, `Runtime`.

## Terminal and colour

The colour depth is resolved once at startup:

| Environment | Result |
|---|---|
| `COLORTERM=truecolor` or `24bit` | RGB |
| `TERM=*256color` | indexed 256 |
| `TERM` unset | assumed modern, RGB |
| other `TERM` | nearest of the 16 ANSI colours |
| `NO_COLOR` or `TERM=dumb` | no colour; hierarchy on bold, dim and rules |

Each topic carries a colour, and it is used as ink, never as a fill. The hue is kept,
saturation capped and lightness pinned to a legible band, so `#264de4` (2.92:1 on the
dark ground) becomes `#627bda` (4.80:1). The theme is dark only for now.

## Status

`0.1.0`, early, one maintainer, read-only by design: it does not copy, does not run
commands, does not sync and has no database. What you cannot get from it, you cannot
get wrong from it either.

The content is hand-written for this project and checked by machine where a checker
exists: compilers for C, C++, Go, Rust, PHP, Lua and Python; `node --check` for
JavaScript; `tsc` for TypeScript; tree-sitter for Java, C#, Kotlin, Swift, Dart, Ruby,
HTML and CSS; sqlglot for PostgreSQL and MySQL; execution against numpy and pandas.
Three topics have no checker on the machine this was written on — `mongodb`, `redis`
and `windows-commands` — and are reviewed by hand; treat them as the least verified
pages.

## Licence

MIT. See [`LICENSE`](LICENSE).
