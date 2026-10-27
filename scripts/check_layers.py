#!/usr/bin/env python3
"""Checks that the workspace dependencies respect architecture/layers.toml.

Usage: python3 scripts/check_layers.py
Exit code: 0 if everything complies, 1 if there are violations, 2 if reading fails.
"""

import json
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    try:
        rules = tomllib.loads((ROOT / "architecture" / "layers.toml").read_text())
        meta = json.loads(
            subprocess.run(
                ["cargo", "metadata", "--format-version", "1", "--no-deps"],
                cwd=ROOT,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
        )
    except (OSError, subprocess.CalledProcessError, tomllib.TOMLDecodeError) as err:
        print(f"error reading rules or metadata: {err}", file=sys.stderr)
        return 2

    crate_rules = rules.get("crates", {})
    external_rules = rules.get("external", {})
    members = {pkg["name"]: pkg for pkg in meta["packages"]}
    problems: list[str] = []

    for name in sorted(members):
        if name not in crate_rules:
            problems.append(f"{name}: crate has no rule in architecture/layers.toml")

    for name in sorted(crate_rules):
        if name not in members:
            problems.append(f"{name}: has a rule but does not exist in the workspace")

    for name, pkg in sorted(members.items()):
        rule = crate_rules.get(name)
        if rule is None:
            continue
        allow = rule.get("allow", [])
        for dep in pkg["dependencies"]:
            dep_name = dep["name"]
            if dep_name in members:
                if "*" not in allow and dep_name not in allow:
                    problems.append(
                        f"{name} -> {dep_name}: dependency between crates not allowed"
                    )
            elif name in external_rules and dep_name not in external_rules[name]:
                problems.append(
                    f"{name} -> {dep_name}: external dependency not allowed"
                )

    if problems:
        print("Layer violations:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print(f"layers ok ({len(members)} crates checked)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
