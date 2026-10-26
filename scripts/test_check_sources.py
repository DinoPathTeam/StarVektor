"""Pruebas de scripts/check_sources.py. Ejecutar con:

    python3 -m unittest discover -s scripts -p "test_*.py"
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import check_sources

BASE = {
    "schema": 1,
    "evaluation_only": {"games": ["Control", "Cyberpunk 2077"]},
    "reference_only": {"systems": ["FSR 3", "AFMF 2"]},
}


def source(**overrides):
    entry = {
        "id": "escena-01",
        "kind": "own_scene",
        "use": ["train", "eval"],
        "license": "propia",
        "origin": "escena propia en Godot",
        "added": "2026-10-26",
        "frame_generation": "off",
    }
    entry.update(overrides)
    return entry


def problems_for(*sources, **extra):
    return check_sources.check({**BASE, **extra, "source": list(sources)})


class CheckSources(unittest.TestCase):
    def test_empty_list_and_a_valid_source_pass(self):
        self.assertEqual(problems_for(), [])
        self.assertEqual(problems_for(source()), [])

    def test_missing_fields_are_reported(self):
        found = problems_for(source(license="", origin=""))
        self.assertTrue(any("license" in p for p in found), found)
        self.assertTrue(any("origin" in p for p in found), found)

    def test_duplicate_ids_and_unknown_kinds(self):
        found = problems_for(source(), source(kind="scrapeo"))
        self.assertTrue(any("id repetido" in p for p in found), found)
        self.assertTrue(any("kind desconocido" in p for p in found), found)

    def test_use_must_be_train_and_or_eval(self):
        self.assertTrue(problems_for(source(use=["publicar"])))
        self.assertTrue(problems_for(source(use=[])))

    def test_frame_generation_must_be_declared_off(self):
        missing = source()
        del missing["frame_generation"]
        self.assertTrue(any("frame_generation" in p for p in problems_for(missing)))
        found = problems_for(source(frame_generation="on"))
        self.assertTrue(any("generación de fotogramas" in p for p in found), found)

    def test_short_game_names_do_not_cause_false_positives(self):
        # "control" aparece dentro de otra frase, pero no es el juego Control.
        ok = source(origin="escena propia con control de versiones", game="Mi juego")
        self.assertEqual(problems_for(ok), [])

    def test_bad_date(self):
        self.assertTrue(any("AAAA-MM-DD" in p for p in problems_for(source(added="ayer"))))

    def test_a_permitted_game_needs_explicit_permission_and_name(self):
        found = problems_for(source(id="juego-x", kind="permitted_game"))
        self.assertTrue(any("permission" in p for p in found), found)
        self.assertTrue(any("`game`" in p for p in found), found)
        ok = problems_for(
            source(id="juego-x", kind="permitted_game", game="Juego X", permission="carta del estudio")
        )
        self.assertEqual(ok, [])

    def test_an_evaluation_only_game_can_never_train(self):
        sneaky = source(
            id="cp-01",
            kind="permitted_game",
            game="cyberpunk 2077",
            permission="x",
            use=["train"],
        )
        found = problems_for(sneaky)
        self.assertTrue(any("solo evaluación" in p for p in found), found)
        evaluation = source(
            id="cp-02", kind="permitted_game", game="Cyberpunk 2077", permission="x", use=["eval"]
        )
        self.assertEqual(problems_for(evaluation), [])

    def test_reference_generator_output_can_never_train(self):
        found = problems_for(source(id="fsr3-capturas", use=["train"]))
        self.assertTrue(any("generador de referencia" in p for p in found), found)

    def test_wrong_schema_version(self):
        self.assertTrue(check_sources.check({**BASE, "schema": 2}))

    def test_the_repository_file_complies(self):
        script = Path(check_sources.__file__)
        result = subprocess.run(
            [sys.executable, str(script)], capture_output=True, text=True, check=False
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_unreadable_file_exits_with_2(self):
        script = Path(check_sources.__file__)
        with tempfile.TemporaryDirectory() as tmp:
            broken = Path(tmp) / "roto.toml"
            broken.write_text("esto = no es toml [")
            result = subprocess.run(
                [sys.executable, str(script), str(broken)],
                capture_output=True,
                text=True,
                check=False,
            )
        self.assertEqual(result.returncode, 2)


if __name__ == "__main__":
    unittest.main()
