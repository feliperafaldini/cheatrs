# cheatrs

A terminal browser for programming cheatsheets: two columns to find the topic, one
column to read it, full screen to read it properly. Built for the moment you know
what you forgot but not where it lives.

```
cheatrs  52 topics · 0 skipped
▌ LANGUAGES           │▌ SH Bash                     │▌ Getting Start… Bash
  FRAMEWORKS          │   C C                        │  Comments       Unix shell and command
  LIBRARIES           │  C# C#                       │  Variables      language for scripting and
  TOOLS               │ C++ C++                      │  Quotes
  DATABASES           │ CSS CSS                      │  Special Varia… Getting Started
  RUNTIMES            │  DT Dart                     │  User Input     Create and run your first script:
                      │  GO Go                       │  Output
                      │  HT HTML                     │  Arithmetic       #!/bin/bash
                      │  JV Java                     │  Conditionals     echo "Hello World"
                      │  JS JavaScript               │  Test Operators
                      │  KT Kotlin                   │  Case Statement
                      │ LUA Lua                      │  Loops
                      │ PHP PHP                      │
                      │  PY Python                   │
                      │  RB Ruby                     │
                      │  RS Rust                     │
hjkl/←→↑↓ move · ⇥/l column · ↵ open list · / search · M mouse off · q quit  1/6
```

`Enter` on a topic opens it as a document, with its sections on the left and the
code highlighted:

```
‹ LANGUAGE / Bash
▌ Getting Started    Bash                                                                  █
  Comments           Unix shell and command language for scripting and automation.         ▏
  Variables                                                                                ▏
  Quotes             Getting Started                                                       ▏
  Special Variables  Create and run your first script:                                     ▏
  User Input                                                                               ▏
  Output               #!/bin/bash                                                         ▏
  Arithmetic           # The first line (shebang) tells the system to run this file with   ▏
  Conditionals                                                                             ▏
  Test Operators       echo "Hello World"                                                  ▏
  Case Statement                                                                           ▏
  Loops                # Save as script.sh, then:                                          ▏
  Functions            chmod +x script.sh  # Make it executable                            ▏
↑↓ scroll · ⇥ sections · g/G top/bottom · q back
```

## Build and run

```bash
cargo build --release
./target/release/cheatrs
```

Needs a terminal of at least 80x24; below that it says so and exits instead of
drawing a broken screen. The preview column appears from 110 columns, the section
list inside the reader from 120.

## Keys

| Key | Does |
|---|---|
| `j` `k` or arrows | move in the focused column; scroll the document |
| `l` `→` `Tab` | next column (groups → topics → preview) |
| `h` `←` `Esc` | previous column |
| `Enter` | in groups: step into the list; elsewhere: open the topic |
| `/` | search |
| `g` `G` | top and bottom, inside the reader |
| `M` | toggle mouse capture |
| `q` | back one level, or quit from the home screen |

## Search

One index over topic titles, section titles and the code inside every snippet.
Ranking: title prefix, title match, section's topic, then code. A code hit carries
the matching line, and `Enter` opens the reader at that section.

`data/aliases.json` maps what people type to what the titles spell:

```json
"psql": "postgres", "ts": "typescript", "py": "python", "nvim": "vim"
```

## Your own topics

Anything in `$XDG_DATA_HOME/cheatrs/topics/*.json` is loaded over the embedded
base, winning by `slug`. `data/template.json` shows the shape; `--validate` says
what is wrong instead of silently skipping:

```bash
cheatrs --validate                 # every topic the binary can see
cheatrs --validate my-topic.json   # a single file
```

A topic needs `slug`, `title`, `label`, `group`, `color` and at least one section
with a snippet. Groups in use: `Language`, `Framework`, `Library`, `Tool`,
`Database`, `Runtime`.

## Colour

The theme resolves once at startup:

| Environment | Result |
|---|---|
| `COLORTERM=truecolor` | RGB |
| `TERM=*256color` | indexed 256 |
| other `TERM` | nearest of the 16 ANSI colours |
| `NO_COLOR` or `TERM=dumb` | no colour; hierarchy on bold, dim and rules |

The accent is the topic colour with hue kept, saturation capped and lightness
pinned to a legible band, so `#264de4` (2.92:1 on the dark ground) becomes
`#627bda` (4.80:1).

## What it does not do

Read-only by design: it does not copy, does not execute commands, does not sync
over the network and has no database. What you cannot get from it, you cannot
get wrong from it either.

## Content

The 52 topics shipped here were written for this project. `data/topics/` holds
one JSON per topic; `data/template.json` describes the format.

## Licence

MIT. See `LICENSE`.
