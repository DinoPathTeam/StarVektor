"""Tests for scripts/check_sources.py. Run with:

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
        "id": "scene-01",
        "kind": "own_scene",
        "use": ["train", "eval"],
        "license": "own",
        "origin": "own scene in Godot",
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
        found = problems_for(source(), source(kind="scraping"))
        self.assertTrue(any("duplicate id" in p for p in found), found)
        self.assertTrue(any("unknown kind" in p for p in found), found)

    def test_use_must_be_train_and_or_eval(self):
        self.assertTrue(problems_for(source(use=["publish"])))
        self.assertTrue(problems_for(source(use=[])))

    def test_frame_generation_must_be_declared_off(self):
        missing = source()
        del missing["frame_generation"]
        self.assertTrue(any("frame_generation" in p for p in problems_for(missing)))
        found = problems_for(source(frame_generation="on"))
        self.assertTrue(any("frame generation" in p for p in found), found)

    def test_short_game_names_do_not_cause_false_positives(self):
        # "control" appears inside another phrase, but it is not the game Control.
        ok = source(origin="own scene under version control", game="My game")
        self.assertEqual(problems_for(ok), [])

    def test_bad_date(self):
        self.assertTrue(any("YYYY-MM-DD" in p for p in problems_for(source(added="yesterday"))))

    def test_a_permitted_game_needs_explicit_permission_and_name(self):
        found = problems_for(source(id="game-x", kind="permitted_game"))
        self.assertTrue(any("permission" in p for p in found), found)
        self.assertTrue(any("`game`" in p for p in found), found)
        ok = problems_for(
            source(id="game-x", kind="permitted_game", game="Game X", permission="letter from the studio")
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
        self.assertTrue(any("evaluation-only" in p for p in found), found)
        evaluation = source(
            id="cp-02", kind="permitted_game", game="Cyberpunk 2077", permission="x", use=["eval"]
        )
        self.assertEqual(problems_for(evaluation), [])

    def test_reference_generator_output_can_never_train(self):
        found = problems_for(source(id="fsr3-captures", use=["train"]))
        self.assertTrue(any("reference generator" in p for p in found), found)

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
            broken = Path(tmp) / "broken.toml"
            broken.write_text("this = is not toml [")
            result = subprocess.run(
                [sys.executable, str(script), str(broken)],
                capture_output=True,
                text=True,
                check=False,
            )
        self.assertEqual(result.returncode, 2)


if __name__ == "__main__":
    unittest.main()
