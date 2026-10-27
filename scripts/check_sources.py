#!/usr/bin/env python3
"""Checks data/sources.toml against the data policy (docs/data-policy.md).

Usage: python3 scripts/check_sources.py [path]   (default: data/sources.toml)
Exit code: 0 if compliant, 1 if there are violations, 2 if reading fails.
"""

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KINDS = {"own_scene", "open_dataset", "permitted_game"}
USES = {"train", "eval"}
REQUIRED = ("id", "kind", "use", "license", "origin", "added", "frame_generation")
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def norm(text: object) -> str:
    """Lowercase letters and digits only: "FSR 3" and "fsr3-captures" are comparable."""
    return re.sub(r"[^a-z0-9]", "", str(text).casefold())


def check(data: dict) -> list[str]:
    """Returns the list of violations; empty if the policy is met."""
    problems: list[str] = []

    if data.get("schema") != 1:
        problems.append("schema must be 1")

    evaluation_only = data.get("evaluation_only", {}).get("games", [])
    if not isinstance(evaluation_only, list) or not all(
        isinstance(g, str) and g.strip() for g in evaluation_only
    ):
        problems.append("evaluation_only.games must be a list of names")
        evaluation_only = []
    reference_only = data.get("reference_only", {}).get("systems", [])
    if not isinstance(reference_only, list) or not all(
        isinstance(s, str) and s.strip() for s in reference_only
    ):
        problems.append("reference_only.systems must be a list of names")
        reference_only = []

    eval_only_names = {norm(g) for g in evaluation_only}
    reference_names = {norm(s) for s in reference_only}
    seen_ids: set[str] = set()
    sources = data.get("source", [])

    for index, source in enumerate(sources, start=1):
        label = f"source #{index} ({source.get('id', '?')})"
        for key in REQUIRED:
            if not source.get(key):
                problems.append(f"{label}: missing `{key}`")

        source_id = source.get("id")
        if source_id in seen_ids:
            problems.append(f"{label}: duplicate id")
        if source_id:
            seen_ids.add(source_id)

        kind = source.get("kind")
        if kind and kind not in KINDS:
            problems.append(f"{label}: unknown kind `{kind}`")

        uses = source.get("use", [])
        if not isinstance(uses, list) or not uses or not set(uses) <= USES:
            problems.append(f"{label}: `use` must be a list with train and/or eval")
            uses = []

        added = source.get("added")
        if added and not DATE.match(str(added)):
            problems.append(f"{label}: `added` must be YYYY-MM-DD")

        if kind == "permitted_game":
            if not source.get("permission"):
                problems.append(f"{label}: a game needs an explicit `permission`")
            if not source.get("game"):
                problems.append(f"{label}: a game needs `game`")

        if source.get("frame_generation") and source["frame_generation"] != "off":
            problems.append(
                f"{label}: capture must be done with frame generation off "
                '(`frame_generation = "off"`)'
            )

        # An evaluation-only game is recognized by its exact name in `game`.
        # A commercial game declared as something else cannot be detected here:
        # human review covers that (see docs/data-policy.md).
        if "train" in uses and norm(source.get("game", "")) in eval_only_names:
            problems.append(f"{label}: an evaluation-only game cannot be used for training")

        # Reference generators have distinctive names: it is enough that they
        # appear inside the id or the origin.
        text = norm(source.get("id", "")) + "|" + norm(source.get("origin", ""))
        if "train" in uses and any(ref and ref in text for ref in reference_names):
            problems.append(
                f"{label}: the output of a reference generator cannot be used for training"
            )

    return problems


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else ROOT / "data" / "sources.toml"
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as err:
        print(f"error reading {path}: {err}", file=sys.stderr)
        return 2

    problems = check(data)
    if problems:
        print("Data policy violations:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print(
        f"sources ok ({len(data.get('source', []))} sources, "
        f"{len(data.get('evaluation_only', {}).get('games', []))} evaluation-only games)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
