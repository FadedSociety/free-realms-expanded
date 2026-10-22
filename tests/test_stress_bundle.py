"""T431: the stress harness runs from a bundle, fights always-on enemies and scores honestly."""

from __future__ import annotations

import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "bundle"))

import check_bundle  # noqa: E402
import make_bundle  # noqa: E402

spec = importlib.util.spec_from_file_location("stress_run", ROOT / "scripts" / "stress" / "run.py")
run = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run)


def write(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding="utf-8")


def enemy(guid: int, x: float, damage: int, quest_ids=(), boss=False, position=None, level=1) -> dict:
    return {"guid": guid, "position": position or [x, 0.0, 0.0], "damage": damage,
        "boss": boss, "quest_ids": list(quest_ids), "level": level}


class StressBundleTest(unittest.TestCase):
    def setUp(self):
        self.temp = Path(tempfile.mkdtemp(prefix="osfr-stress-bundle-"))
        self.content = self.temp / "content"
        write(self.content / "npcs" / "a.json", {"records": [
            {"npc_id": 1, "position": [0.0, 0.0, 0.0], "quest_ids_given": [10]},
            {"npc_id": 2, "position": [5.0, 0.0, 0.0], "roles": ["vendor"]},
        ]})
        write(self.content / "quests" / "a.json", {"records": [{"id": 10, "giver_npc_id": 1}]})
        write(self.content / "stores.json", {"items": [{"item_id": 500, "purchase_price_coins": 10}]})
        write(self.content / "item-catalogue.json", {"items": [{"item_id": 500}]})
        write(self.content / "vendors.json", {"vendors": [{"npc_id": 2, "item_ids": [500]}]})

    def tearDown(self):
        shutil.rmtree(self.temp)

    def scenario(self, enemies: list[dict]) -> dict:
        write(self.content / "field_enemies.json", {"enemies": enemies})
        return run.scenario_from_content(self.content)

    def test_only_always_on_enemies_nearest_eight_softest_first(self):
        rows = [enemy(1, 1, 1, quest_ids=[3030007]), enemy(2, 2, 1, boss=True),
            enemy(3, 2.5, 1, position=[2.5, 0.0])]
        # Nine non-boss rows; the farther, the softer. The farthest (and softest) is not near enough.
        rows += [enemy(100 + d, d, 30 - d) for d in range(3, 12)]
        # Authored damage is a level-one baseline the server scales by level (encounter/field.rs),
        # so level outranks it. A level-12 row stacked on 103 is picked but goes after every
        # level-1 row; a level-5 row stacked on 109 loses the eighth slot to it, pushing 110 out.
        rows += [enemy(200, 3, 1, level=12), enemy(201, 9, 1, level=5)]
        picked = self.scenario(rows)["enemies"]
        self.assertEqual([row["guid"] for row in picked], [109, 108, 107, 106, 105, 104, 103, 200])
        self.assertEqual(picked[-1]["level"], 12)
        self.assertTrue(all(row["quest_ids"] == [] and not row["boss"] for row in picked))

    def test_a_fixture_with_only_gated_enemies_is_refused_clearly(self):
        gated = [enemy(1, 1, 1, quest_ids=[3030007]), enemy(2, 2, 1, quest_ids=[3030006])]
        with self.assertRaisesRegex(RuntimeError, "no always-on enemy"):
            self.scenario(gated)

    def test_missing_scenario_files_are_named(self):
        (self.content / "stores.json").unlink()
        with self.assertRaisesRegex(RuntimeError, "lacks field_enemies.json, stores.json"):
            run.scenario_from_content(self.content)

    def test_a_single_phase_sample_is_unmeasured_and_fails_the_broker_exit(self):
        run_dir = self.temp / "run"
        guids = (1001, 1002, 1003)
        for guid in guids:
            write(run_dir / "state" / "characters" / f"{guid}.json",
                {"wallet": {"coins": 4925, "items": [{"definition_id": 500, "count": 1}]}})
        records = [{"guid": guid} for guid in guids]
        phases = {"1001": [{"item_count": 1}], "1002": [{"item_count": 1}, {"item_count": 1}],
            "1003": [{"item_count": 1}, {"item_count": 0}]}
        saves = run.save_summary(run_dir, records, {"item_id": 500}, phases)
        self.assertEqual([row["purchase_persistence"] for row in saves], ["unmeasured", "persisted", "lost"])

        clean = {"broker_refusals": 0, "login_failures": 0}
        loaded = [{"loaded": True}]
        passing = [dict(saves[1])]
        self.assertFalse(run.run_failed(False, True, [], clean, loaded, passing))
        for row in (saves[0], saves[2], dict(saves[1], coins_nonnegative=False)):
            self.assertTrue(run.run_failed(False, True, [], clean, loaded, [row]), row)
        self.assertFalse(run.run_failed(False, False, [], clean, loaded, [saves[0]]), "direct mode reports only")

    def test_knockouts_are_an_explained_category(self):
        bots = self.temp / "run" / "bots"
        bots.mkdir(parents=True)
        (bots / "bot-01-session-01.stdout.log").write_text("\n".join([
            "[Bot01    10ms] <- 36/1 len=8",
            "[Bot01    20ms] <- 41/125 len=20",
            "[Bot01    30ms] <- 36/1 len=8",
            "[Bot01    40ms] <- 36/10 len=40",
            "[Bot01    50ms] <- 41/126 len=12",
        ]), encoding="utf-8")
        (bots / "save-load-1001.log").write_text("[Load1001   5ms] <- 41/125 len=20\n", encoding="utf-8")
        (bots / "save-load-1002.log").write_text("[Load1002   5ms] <- 41/126 len=12\n", encoding="utf-8")
        self.assertEqual(run.knockout_summary(self.temp / "run"), {"knockouts": 1, "revives": 1,
            "refusals_while_down": 1, "other_refusals": 1, "reloaded_down": 1})

    def test_bundle_paths_are_the_readme_launch_lines(self):
        text = make_bundle.readme("playtest-x", "0" * 40, None)
        for key, relative in run.BUNDLE_LAUNCH.items():
            windows = relative.replace("/", "\\")
            expected = (f"$env:{key} = Join-Path $bundle '{windows}'" if key.startswith("OSFR_")
                else f'{key} "$bundle\\{windows}"')
            self.assertIn(expected, text)
        self.assertIn('--identity "$runtime\\identity.json"', text, "the identity stays in the run folder")

    def test_bin_dir_refuses_an_incomplete_release_set(self):
        (self.temp / "osfr-gateway.exe").write_bytes(b"")
        with self.assertRaisesRegex(FileNotFoundError, "lacks: osfr-login, osfr-bot"):
            check_bundle.existing_binaries(self.temp)
        for name in ("osfr-login", "osfr-bot"):
            (self.temp / f"{name}.exe").write_bytes(b"")
        self.assertEqual(set(check_bundle.existing_binaries(self.temp)), set(check_bundle.BINARIES))


if __name__ == "__main__":
    unittest.main()
