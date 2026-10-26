#!/usr/bin/env python3
"""Verifica data/sources.toml contra la política de datos (docs/data-policy.md).

Uso: python3 scripts/check_sources.py [ruta]   (por defecto, data/sources.toml)
Código de salida: 0 si cumple, 1 si hay violaciones, 2 si falla la lectura.
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
    """Minúsculas y solo letras y dígitos: "FSR 3" y "fsr3-capturas" son comparables."""
    return re.sub(r"[^a-z0-9]", "", str(text).casefold())


def check(data: dict) -> list[str]:
    """Devuelve la lista de violaciones; vacía si la política se cumple."""
    problems: list[str] = []

    if data.get("schema") != 1:
        problems.append("schema debe ser 1")

    evaluation_only = data.get("evaluation_only", {}).get("games", [])
    if not isinstance(evaluation_only, list) or not all(
        isinstance(g, str) and g.strip() for g in evaluation_only
    ):
        problems.append("evaluation_only.games debe ser una lista de nombres")
        evaluation_only = []
    reference_only = data.get("reference_only", {}).get("systems", [])
    if not isinstance(reference_only, list) or not all(
        isinstance(s, str) and s.strip() for s in reference_only
    ):
        problems.append("reference_only.systems debe ser una lista de nombres")
        reference_only = []

    eval_only_names = {norm(g) for g in evaluation_only}
    reference_names = {norm(s) for s in reference_only}
    seen_ids: set[str] = set()
    sources = data.get("source", [])

    for index, source in enumerate(sources, start=1):
        label = f"source #{index} ({source.get('id', '?')})"
        for key in REQUIRED:
            if not source.get(key):
                problems.append(f"{label}: falta `{key}`")

        source_id = source.get("id")
        if source_id in seen_ids:
            problems.append(f"{label}: id repetido")
        if source_id:
            seen_ids.add(source_id)

        kind = source.get("kind")
        if kind and kind not in KINDS:
            problems.append(f"{label}: kind desconocido `{kind}`")

        uses = source.get("use", [])
        if not isinstance(uses, list) or not uses or not set(uses) <= USES:
            problems.append(f"{label}: `use` debe ser una lista con train y/o eval")
            uses = []

        added = source.get("added")
        if added and not DATE.match(str(added)):
            problems.append(f"{label}: `added` debe ser AAAA-MM-DD")

        if kind == "permitted_game":
            if not source.get("permission"):
                problems.append(f"{label}: un juego necesita `permission` explícito")
            if not source.get("game"):
                problems.append(f"{label}: un juego necesita `game`")

        if source.get("frame_generation") and source["frame_generation"] != "off":
            problems.append(
                f"{label}: la captura debe hacerse con la generación de fotogramas "
                '(`frame_generation = "off"`)'
            )

        # Un juego de solo evaluación se reconoce por su nombre exacto en `game`.
        # Un juego comercial declarado como otra cosa no se puede detectar aquí:
        # eso lo cubre la revisión humana (ver docs/data-policy.md).
        if "train" in uses and norm(source.get("game", "")) in eval_only_names:
            problems.append(f"{label}: un juego de solo evaluación no puede usarse para entrenar")

        # Los generadores de referencia tienen nombres distintivos: basta con que
        # aparezcan dentro del id o del origen.
        text = norm(source.get("id", "")) + "|" + norm(source.get("origin", ""))
        if "train" in uses and any(ref and ref in text for ref in reference_names):
            problems.append(
                f"{label}: la salida de un generador de referencia no puede usarse para entrenar"
            )

    return problems


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else ROOT / "data" / "sources.toml"
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as err:
        print(f"error al leer {path}: {err}", file=sys.stderr)
        return 2

    problems = check(data)
    if problems:
        print("Violaciones de la política de datos:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print(
        f"fuentes ok ({len(data.get('source', []))} fuentes, "
        f"{len(data.get('evaluation_only', {}).get('games', []))} juegos solo-evaluación)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
