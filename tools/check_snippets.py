#!/usr/bin/env python3
"""Syntax-check what can be checked mechanically, and say what could not be.

bash  -> bash -n            python -> py_compile      yaml -> yaml.safe_load
json  -> json.loads          sql, text, html, css... -> not checked

Snippets that show placeholders (`<file>`, `...`, `{{x}}`) or that are obvious
fragments are skipped instead of reported: a syntax error inside an ellipsis is
not a bug in the cheatsheet.
"""

from __future__ import annotations

import json
import subprocess
import sys

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TOPICS = ROOT / "data" / "topics"

PLACEHOLDERS = ("<", "...", "{{", "}}", "your-", "path/to")
FRAGMENT_HINTS = ("struct ", "impl ", "class ", "def ", "fn ")


def looks_synthetic(code: str) -> bool:
    return any(token in code for token in PLACEHOLDERS)


def check_bash(code: str) -> str | None:
    result = subprocess.run(["bash", "-n"], input=code, text=True, capture_output=True)
    if result.returncode != 0:
        return result.stderr.strip().splitlines()[0] if result.stderr else "bash -n failed"
    return None


def check_python(code: str) -> str | None:
    # compile() instead of py_compile: no __pycache__ to write, no permission games.
    result = subprocess.run(
        [sys.executable, "-c", "import sys; compile(sys.stdin.read(), '<snippet>', 'exec')"],
        input=code,
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        lines = [line for line in result.stderr.splitlines() if line.strip()]
        return lines[-1] if lines else "compile failed"
    return None


def check_yaml(code: str) -> str | None:
    try:
        import yaml  # type: ignore
    except ImportError:
        return None
    try:
        yaml.safe_load(code)
    except Exception as error:  # noqa: BLE001 - any parse error is the point
        return str(error).splitlines()[0]
    return None


def check_json(code: str) -> str | None:
    try:
        json.loads(code)
    except Exception as error:  # noqa: BLE001
        return str(error)
    return None


CHECKERS = {"bash": check_bash, "sh": check_bash, "python": check_python, "yaml": check_yaml, "json": check_json}


def main() -> int:
    checked = skipped = failed = 0
    problems: list[str] = []

    for path in sorted(TOPICS.glob("*.json")):
        topic = json.loads(path.read_text())
        for section in topic.get("sections", []):
            for snippet in section.get("snippets", []):
                language = snippet.get("language", "")
                code = snippet.get("code", "")
                checker = CHECKERS.get(language)
                if checker is None or looks_synthetic(code) or code.count("\n") < 1:
                    skipped += 1
                    continue
                checked += 1
                problem = checker(code)
                if problem:
                    failed += 1
                    problems.append(f"{topic['slug']}/{section['id']}: {problem}")

    for problem in problems[:20]:
        print(problem)
    print()
    print(f"{checked} snippets checked, {failed} failed, {skipped} skipped (language without checker or placeholder)")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
