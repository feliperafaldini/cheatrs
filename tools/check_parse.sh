#!/usr/bin/env bash
# Parse-only check for languages whose compiler is not installed here.
#
# tree_sitter answers "does this parse as Ruby/C#/Kotlin/Swift/Java/Dart?", not
# "does it type-check". For a cheatsheet that is the useful question: a snippet
# that does not parse is wrong no matter what it means.
#
# uso: tools/check_parse.sh ruby java        (sem argumento: todos os mapeados)
set -euo pipefail
cd "$(dirname "$0")/.."
exec uv run --quiet \
  --with tree-sitter \
  --with tree-sitter-ruby \
  --with tree-sitter-c-sharp \
  --with tree-sitter-kotlin \
  --with tree-sitter-swift \
  --with tree-sitter-java \
  --with tree-sitter-dart \
  python - "$@" <<'PY'
import json
import sys
from pathlib import Path

from tree_sitter import Language, Parser

import tree_sitter_ruby
import tree_sitter_c_sharp
import tree_sitter_kotlin
import tree_sitter_swift
import tree_sitter_java
import tree_sitter_dart

GRAMMARS = {
    "ruby": (Language(tree_sitter_ruby.language()), {"ruby", "rb"}),
    "csharp": (Language(tree_sitter_c_sharp.language()), {"csharp", "c#", "cs"}),
    "kotlin": (Language(tree_sitter_kotlin.language()), {"kotlin", "kt"}),
    "swift": (Language(tree_sitter_swift.language()), {"swift"}),
    "java": (Language(tree_sitter_java.language()), {"java"}),
    "dart": (Language(tree_sitter_dart.language()), {"dart"}),
}

slugs = sys.argv[1:] or list(GRAMMARS)
failed = 0

for slug in slugs:
    if slug not in GRAMMARS:
        print(f"{slug}: sem gramatica mapeada")
        continue
    language, names = GRAMMARS[slug]
    parser = Parser(language)
    topic = json.loads(Path(f"data/topics/{slug}.json").read_text())
    ok, bad = 0, []
    for section in topic["sections"]:
        for snippet in section["snippets"]:
            if snippet["language"].lower() not in names:
                continue
            if parser.parse(snippet["code"].encode()).root_node.has_error:
                bad.append((section["id"], snippet["code"].splitlines()[0][:70]))
            else:
                ok += 1
    failed += len(bad)
    print(f"{slug:7} {ok:3} ok, {len(bad)} com erro de parse")
    for section_id, first in bad[:5]:
        print(f"    {section_id} | {first}")

sys.exit(1 if failed else 0)
PY
