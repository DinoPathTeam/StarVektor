#!/usr/bin/env python3
"""Verifica que las dependencias del workspace respeten architecture/layers.toml.

Uso: python3 scripts/check_layers.py
Código de salida: 0 si todo cumple, 1 si hay violaciones, 2 si falla la lectura.
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
        print(f"error al leer reglas o metadatos: {err}", file=sys.stderr)
        return 2

    crate_rules = rules.get("crates", {})
    external_rules = rules.get("external", {})
    members = {pkg["name"]: pkg for pkg in meta["packages"]}
    problems: list[str] = []

    for name in sorted(members):
        if name not in crate_rules:
            problems.append(f"{name}: crate sin regla en architecture/layers.toml")

    for name in sorted(crate_rules):
        if name not in members:
            problems.append(f"{name}: tiene regla pero no existe en el workspace")

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
                        f"{name} -> {dep_name}: dependencia entre crates no permitida"
                    )
            elif name in external_rules and dep_name not in external_rules[name]:
                problems.append(
                    f"{name} -> {dep_name}: dependencia externa no permitida"
                )

    if problems:
        print("Violaciones de capas:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print(f"capas ok ({len(members)} crates revisados)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
