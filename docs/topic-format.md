# Topic format

A topic is one JSON file. The binary reads `data/topics/*.json` at compile time and
`$XDG_DATA_HOME/cheatrs/topics/*.json` at runtime; the runtime file wins on `slug`.

Every file may declare `schema`. This binary understands **schema 1**. A file that
declares a higher number is refused with a message naming both numbers, instead of
being half-read — a field this version does not know about would otherwise vanish
without a trace. Omitting `schema` means 1.

## Fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `schema` | number | no | Defaults to 1. Higher than the binary supports is an error |
| `slug` | string | yes | Unique id, lowercase kebab-case. Two files with the same slug: the user's wins |
| `title` | string | yes | Shown in the topics column and as the document heading |
| `label` | string | yes | Short name kept in the data; the column uses the monogram instead |
| `group` | string | yes | One of `Language`, `Framework`, `Library`, `Tool`, `Database`, `Runtime` |
| `color` | string | yes | `#rrggbb`. Used as ink, and pulled to 4.5:1 contrast if it is too dark |
| `desc` | string | yes | One line, shown under the heading in the reader |
| `sections` | array | yes | At least one section with at least one snippet, or the topic never shows |

## Sections

| Field | Type | Required | Notes |
|---|---|---|---|
| `id` | string | yes | kebab-case, unique inside the topic |
| `title` | string | yes | Shown in the contents column |
| `description` | string | yes | One line under the title, dim |
| `snippets` | array | yes | 1–3 is the house style; more than that stops being a cheatsheet |

## Snippets

| Field | Type | Required | Notes |
|---|---|---|---|
| `language` | string | yes | Drives syntax highlight and which checker applies (`bash`, `python`, `yaml`, `sql`, `dart`, `r`, `nginx`, `text`, …) |
| `code` | string | yes | Plain text. No tab-indented markdown, no HTML |

A snippet whose `language` has no checker is still valid; it is just less verified.

## Validating

```bash
cheatrs --validate                # everything the binary can see
cheatrs --validate my-topic.json  # one file, with the reason when it fails
```

Outside the app, the repository has two mechanical checkers:

```bash
python3 tools/check_snippets.py   # bash, python, yaml, json, php, lua, rust…
python3 tools/independence.py     # similarity against the frozen original
```

## Compatibility promise

Schema 1 is frozen. Adding a field that older binaries can ignore does not need a
new number; changing the meaning of an existing field, removing one, or making a
required field optional does. When that happens, bump `SCHEMA_VERSION` in
`src/data.rs` and document the difference here.
