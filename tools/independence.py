#!/usr/bin/env python3
"""Measure how much of the shipped data still descends from the frozen copy.

data/derived/<slug>.json is the original, kept only as a reference and never
shipped. data/topics/<slug>.json is what the app embeds. While a twin exists,
that topic is still derived: the MIT notice has to stay.

The ratio is the real gate. Rewriting that keeps 70% of the wording is a
paraphrase, not an original, and the licence does not care about intentions.
"""

from __future__ import annotations


import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHIPPED = ROOT / "data" / "topics"
DERIVED = ROOT / "data" / "derived"

# Above this, the two files are the same text with cosmetic edits.
PARAPHRASE = 0.55
# Below this, the topic reads as independent work.
ORIGINAL = 0.35


def normalize(text: str) -> str:
    text = text.lower()
    text = re.sub(r"[^a-z0-9 ]+", " ", text)
    return re.sub(r"\s+", " ", text).strip()


def fingerprint(topic: dict) -> str:
    parts = [topic.get("desc", "")]
    for section in topic.get("sections", []):
        parts.append(section.get("title", ""))
        parts.append(section.get("description", ""))
        for snippet in section.get("snippets", []):
            parts.append(snippet.get("code", ""))
    return normalize(" ".join(parts))


def similarity(a: str, b: str) -> float:
    """Jaccard over word trigrams: linear, and blind to word order padding.

    difflib on a megabyte of text takes minutes; this answers the same question
    ("is this the same prose with edits?") in milliseconds.
    """
    if not a and not b:
        return 1.0
    words_a, words_b = a.split(), b.split()
    if len(words_a) < 3 or len(words_b) < 3:
        return 1.0 if a == b else 0.0
    grams_a = {tuple(words_a[i : i + 3]) for i in range(len(words_a) - 2)}
    grams_b = {tuple(words_b[i : i + 3]) for i in range(len(words_b) - 2)}
    union = len(grams_a | grams_b)
    return len(grams_a & grams_b) / union if union else 1.0


def main() -> int:
    shipped = sorted(SHIPPED.glob("*.json"))
    if not shipped:
        print("no topics found", file=sys.stderr)
        return 1

    pending: list[tuple[str, float]] = []
    done: list[str] = []

    for path in shipped:
        twin = DERIVED / path.name
        if not twin.exists():
            done.append(path.stem)
            continue
        current = json.loads(path.read_text())
        original = json.loads(twin.read_text())
        ratio = similarity(fingerprint(current), fingerprint(original))
        pending.append((path.stem, ratio))

    for slug, ratio in sorted(pending, key=lambda item: -item[1]):
        verdict = (
            "COPY"
            if ratio > 0.85
            else "PARAPHRASE"
            if ratio > PARAPHRASE
            else "closer"
            if ratio > ORIGINAL
            else "original"
        )
        if verdict == "original":
            done.append(slug)
            continue
        print(f"{slug:18} {ratio:5.2f}  {verdict}")

    still_derived = len(pending) - len(done)

    print()
    print(f"{len(done)}/{len(shipped)} topics rewritten, {still_derived} still derived")
    if still_derived == 0:
        print("dependency over: data/derived/ can be deleted and NOTICE trimmed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
