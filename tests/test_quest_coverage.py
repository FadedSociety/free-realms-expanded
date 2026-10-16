"""Tests for the original quest coverage audit (MMO work queue item 066).

Run from the project root with the pinned lab Python:

  & "$lab\\tools\\universal-modder-env\\Scripts\\python.exe" -m unittest discover -s tests -p "test_quest_coverage.py" -v

The audit reads pinned, read-only sources only: the supplied emulator quest
records, the pinned FreeRealms-Legacy starting-zone journal, the read-only
wiki corpus, the supplied 2014 locale export and the local reconstructed
opening content. It never writes to production content.
"""

from __future__ import annotations

import importlib.util
import pathlib
import unittest

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
LAB_ROOT = PROJECT_ROOT.parent / "lab"
SCRIPT_PATH = PROJECT_ROOT / "scripts" / "audit-quest-coverage.py"


def load_audit_module():
    spec = importlib.util.spec_from_file_location("audit_quest_coverage", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load audit script at {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AuditApiTests(unittest.TestCase):
    """The script must exist and expose a small stdlib-only audit API."""

    def test_module_loads_and_has_api(self):
        audit = load_audit_module()
        for name in (
            "SCHEMA_VERSION",
            "load_json",
            "sha256_file",
            "find_duplicate_ids",
            "check_catalogue",
            "build_catalogue",
        ):
            self.assertTrue(hasattr(audit, name), f"audit module is missing {name}")

    def test_find_duplicate_ids_reports_repeats(self):
        audit = load_audit_module()
        rows = [{"id": 1}, {"id": 2}, {"id": 2}, {"id": 3}, {"id": 3}]
        self.assertEqual(audit.find_duplicate_ids(rows, "id"), [2, 3])
        self.assertEqual(audit.find_duplicate_ids([{"id": 5}], "id"), [])


class SyntheticCatalogueTests(unittest.TestCase):
    """Negative controls: the checker must fail on broken catalogues."""

    def test_check_catalogue_flags_duplicate_quest_ids(self):
        audit = load_audit_module()
        catalogue = {
            "schema_version": audit.SCHEMA_VERSION,
            "sources": [],
            "corpus_totals": {"supplied-journal": 2},
            "quests": [
                {"source": "supplied-journal", "id": 7, "objectives": []},
                {"source": "supplied-journal", "id": 7, "objectives": []},
            ],
        }
        errors, _ = audit.check_catalogue(catalogue)
        self.assertTrue(any("duplicate" in e for e in errors), errors)

    def test_check_catalogue_flags_dangling_locale_reference(self):
        audit = load_audit_module()
        catalogue = {
            "schema_version": audit.SCHEMA_VERSION,
            "sources": [],
            "corpus_totals": {"supplied-journal": 1},
            "quests": [
                {
                    "source": "supplied-journal",
                    "id": 7,
                    "title_locale_id": 999999,
                    "title_locale": None,
                    "objectives": [],
                }
            ],
        }
        errors, _ = audit.check_catalogue(catalogue)
        self.assertTrue(any("locale" in e for e in errors), errors)

    def test_check_catalogue_flags_dangling_actor_reference(self):
        audit = load_audit_module()
        catalogue = {
            "schema_version": audit.SCHEMA_VERSION,
            "sources": [],
            "corpus_totals": {"local-opening": 1},
            "quests": [
                {
                    "source": "local-opening",
                    "id": 910001,
                    "actor_refs": [
                        {"field": "giver_guid", "guid": 42, "resolved": False},
                    ],
                    "objectives": [],
                }
            ],
        }
        errors, _ = audit.check_catalogue(catalogue)
        self.assertTrue(any("actor" in e for e in errors), errors)


class RealCorpusTests(unittest.TestCase):
    """Totals and integrity of the pinned corpora actually on disk."""

    @classmethod
    def setUpClass(cls):
        cls.audit = load_audit_module()
        cls.catalogue = cls.audit.build_catalogue(PROJECT_ROOT, LAB_ROOT)
        cls.by_source = {}
        for quest in cls.catalogue["quests"]:
            cls.by_source.setdefault(quest["source"], []).append(quest)

    def test_real_corpus_totals(self):
        totals = self.catalogue["corpus_totals"]
        self.assertEqual(totals["supplied-journal"], 25)
        self.assertEqual(totals["supplied-journal-objectives"], 42)
        self.assertEqual(totals["community-journal-stickers"], 16)
        self.assertEqual(totals["community-journal-opening-hub-quests"], 13)
        self.assertEqual(totals["community-journal-hubs"], 10)
        self.assertEqual(totals["wiki-quest-pages"], 52)
        self.assertEqual(totals["wiki-opening-quest-pages"], 2)
        self.assertEqual(totals["local-opening-stages"], 17)

    def test_real_catalogue_integrity_is_clean(self):
        errors, warnings = self.audit.check_catalogue(self.catalogue)
        self.assertEqual(errors, [], errors)
        self.assertIsInstance(warnings, list)

    def test_local_stages_have_no_original_bindings(self):
        stages = self.by_source["local-opening"]
        self.assertEqual(len(stages), 17)
        for stage in stages:
            self.assertIsNone(
                stage["original_quest_id"],
                f"stage {stage['id']} claims an original binding",
            )
        self.assertEqual(self.catalogue["coverage"]["local_binding"]["bound"], 0)
        self.assertEqual(self.catalogue["coverage"]["local_binding"]["total"], 17)

    def test_petty_yeti_1617_claims_disabled(self):
        blocking = {
            entry["quest_id"]: entry for entry in self.catalogue["blocked_claims"]
        }
        self.assertIn(1617, blocking, "quest 1617 must carry an explicit block")
        claim = blocking[1617]
        self.assertEqual(claim["status"], "disabled")
        record = next(q for q in self.by_source["supplied-journal"] if q["id"] == 1617)
        objective = record["objectives"][0]
        self.assertIn("Petty Yeti", objective["name_locale"])
        self.assertEqual(objective["required"], 20)
        engine = next(
            d for d in self.catalogue["engine_definitions"] if d["id"] == 1617
        )
        self.assertEqual(engine["enemy_definitions"], [0])
        self.assertTrue(
            all(
                stage["original_quest_id"] != 1617
                for stage in self.by_source["local-opening"]
            )
        )
        defeat_targets = {
            objective["trigger"].get("definition_id")
            for stage in self.by_source["local-opening"]
            for objective in stage["objectives"]
            if objective.get("trigger", {}).get("kind") == "defeat"
        }
        self.assertEqual(defeat_targets, {3156, 920001, 920002})

    def test_source_hashes_are_recorded(self):
        sources = self.catalogue["sources"]
        self.assertTrue(sources)
        for source in sources:
            self.assertEqual(len(source["sha256"]), 64, source)
            self.assertTrue(source["path"])


if __name__ == "__main__":
    unittest.main()
