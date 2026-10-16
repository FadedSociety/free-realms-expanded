r"""Focused stdlib tests for scripts/audit-minigame-coverage.py.

Run from the project directory:

    & '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests -p "test_minigame_coverage.py" -v

The real-source tests are read-only against the preserved lab corpus. They are
skipped when the lab is absent (for example on another machine).
"""

import hashlib
import importlib.util
import json
import pathlib
import tempfile
import unittest

PROJECT = pathlib.Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"
SCRIPT = PROJECT / "scripts" / "audit-minigame-coverage.py"
RESOURCES = LAB / "working" / "OSFR.Client" / "OSFR Client" / "Resources"
LOCALE = LAB / "reports" / "locale-id-to-text.json"
CATALOG = LAB / "manifests" / "rust-asset-catalog.tsv"
INVENTORY = LAB / "reports" / "minigame-content-inventory.json"

EXPECTED_SOURCE_SHA256 = {
    "MiniGameData.txt": "8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252",
    "MiniGameTypeData.txt": "2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0",
    "MiniGameGroupData.txt": "3b288fa6e665c5ea41122195273ae846251835c8fc323183407e0d48ca969048",
    "MiniGameGroupLinks.txt": "bff472fb91e88ebf6077f541012701cc774dcfb1d7b89261b2ad92d95ffcb623",
}


def load_module():
    spec = importlib.util.spec_from_file_location("audit_minigame_coverage", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


audit = load_module()


def write_fixture_resources(root: pathlib.Path, *, games: str, types: str,
                            groups: str = "#ID^NAME_ID^\n1^0^\n",
                            links: str = "#ID^GROUP_ID^PARENT_GAME_ID^LINK_GAME_ID^LINK_NAME^SHORT_NAME^ICON_ID^STAGE_NUMBER^LINK_GROUP_ID^\n") -> pathlib.Path:
    root.mkdir(parents=True, exist_ok=True)
    (root / "MiniGameData.txt").write_text(games, encoding="utf-8")
    (root / "MiniGameTypeData.txt").write_text(types, encoding="utf-8")
    (root / "MiniGameGroupData.txt").write_text(groups, encoding="utf-8")
    (root / "MiniGameGroupLinks.txt").write_text(links, encoding="utf-8")
    return root


GAME_HEADER = "#ID^TYPE^NAME_ID^DESCRIPTION_ID^ICON_ID^DIFFICULTY^MEMBERS_ONLY^SHORTCUT^MIN_PLAYERS^MAX_PLAYERS^START_SOUND_DEFINITION^PRELOAD_FILE^LEADERBOARD_ID^\n"
TYPE_HEADER = "#TYPE^NAME_ID^DESCRIPTION_ID^IS_FLASH^IS_MICRO^PARAM1^HOWTO_SWF_NAME^ICON_ID^MEMBERS_ONLY^BACKGROUND_ID^IS_POPULAR^IS_HIGHLIGHTED^CATEGORY_ID^IS_ACTIVE^\n"
LINK_HEADER = "#ID^GROUP_ID^PARENT_GAME_ID^LINK_GAME_ID^LINK_NAME^SHORT_NAME^ICON_ID^STAGE_NUMBER^LINK_GROUP_ID^\n"


class ParseTests(unittest.TestCase):
    def test_malformed_row_raises_with_source_and_line(self):
        text = GAME_HEADER + "1^2^3^4^5^1^0^^0^0^0^^0^\n" + "9^9^\n"
        with self.assertRaises(audit.CaretTableError) as ctx:
            audit.parse_caret_text(text, source_name="MiniGameData.txt")
        message = str(ctx.exception)
        self.assertIn("MiniGameData.txt", message)
        self.assertIn("line 3", message)

    def test_lenient_parse_collects_malformed_rows(self):
        text = GAME_HEADER + "9^9\n" + "2^2^0^0^2^1^0^^0^0^0^^0^\n"
        table = audit.parse_caret_text(text, source_name="MiniGameData.txt", strict=False)
        self.assertEqual(len(table.rows), 1)
        self.assertEqual(len(table.malformed), 1)
        self.assertEqual(table.malformed[0]["line"], 2)
        self.assertEqual(table.malformed[0]["fields"], 2)

    def test_trailing_delimiter_column_is_preserved(self):
        table = audit.parse_caret_text(GAME_HEADER + "1^1^0^0^0^1^0^^0^0^0^^0^\n")
        self.assertEqual(table.header[-1], "")
        self.assertEqual(len(table.row_dicts()[0]), len(table.header))


class DetectionTests(unittest.TestCase):
    def test_duplicate_ids_are_reported_sorted(self):
        rows = [{"ID": "7"}, {"ID": "3"}, {"ID": "7"}, {"ID": "3"}, {"ID": "9"}]
        self.assertEqual(audit.find_duplicate_ids(rows), ["3", "7"])

    def test_missing_locale_reference_is_a_finding_not_a_crash(self):
        resources = write_fixture_resources(
            pathlib.Path(tempfile.mkdtemp()) / "Resources",
            games=GAME_HEADER + "1^1^4242^0^0^1^0^^0^0^0^^0^\n",
            types=TYPE_HEADER + "1^0^0^0^0^0^^0^0^0^0^0^0^1^\n",
        )
        locale = resources.parent / "locale.json"
        locale.write_text(json.dumps([{"id": 1, "text": "Known"}]), encoding="utf-8")
        catalogue = audit.build_catalogue(resources, locale)
        missing = catalogue["findings"]["missing_locale_names"]
        self.assertEqual(missing, [{"id": "4242", "games": ["1"]}])
        self.assertEqual(catalogue["games"][0]["name"], "")

    def test_orphan_type_and_dangling_group_link_are_flagged(self):
        resources = write_fixture_resources(
            pathlib.Path(tempfile.mkdtemp()) / "Resources",
            games=GAME_HEADER + "1^9^0^0^0^1^0^^0^0^0^^0^\n",
            types=TYPE_HEADER + "2^0^0^0^0^0^^0^0^0^0^0^0^1^\n",
            links=LINK_HEADER + "1^1^0^77^^^0^0^0^\n",
        )
        locale = resources.parent / "locale.json"
        locale.write_text("[]", encoding="utf-8")
        catalogue = audit.build_catalogue(resources, locale)
        self.assertEqual(catalogue["findings"]["missing_type_definitions"], [{"type": "9", "games": ["1"]}])
        self.assertEqual(
            catalogue["findings"]["dangling_group_links"],
            [{"link_id": "1", "field": "LINK_GAME_ID", "value": "77"}],
        )
        self.assertEqual(catalogue["games"][0]["engine"], audit.ENGINE_UNKNOWN)

    def test_engine_classification_uses_declared_flag(self):
        resources = write_fixture_resources(
            pathlib.Path(tempfile.mkdtemp()) / "Resources",
            games=(GAME_HEADER
                    + "1^1^0^0^0^1^0^^0^0^0^^0^\n"
                    + "2^2^0^0^0^1^0^^0^0^0^^0^\n"
                    + "3^9^0^0^0^1^0^^0^0^0^^0^\n"),
            types=TYPE_HEADER
                  + "1^0^0^1^0^0^^0^0^0^0^0^0^1^\n"
                  + "2^0^0^0^0^0^^0^0^0^0^0^0^1^\n",
        )
        locale = resources.parent / "locale.json"
        locale.write_text("[]", encoding="utf-8")
        catalogue = audit.build_catalogue(resources, locale)
        engines = [g["engine"] for g in catalogue["games"]]
        self.assertEqual(engines, [audit.ENGINE_FLASH, audit.ENGINE_NATIVE, audit.ENGINE_UNKNOWN])

    def test_counts_and_status_totals_are_consistent(self):
        resources = write_fixture_resources(
            pathlib.Path(tempfile.mkdtemp()) / "Resources",
            games=(GAME_HEADER
                    + "1^1^0^0^0^1^0^^0^0^0^^0^\n"
                    + "2^1^0^0^0^1^0^^0^0^0^^0^\n"
                    + "3^2^0^0^0^1^0^^0^0^0^^0^\n"),
            types=TYPE_HEADER + "1^0^0^1^0^0^^0^0^0^0^0^0^1^\n2^0^0^0^0^0^^0^0^0^0^0^0^1^\n",
            links=LINK_HEADER + "1^1^0^1^^^0^0^0^\n2^1^0^3^^^0^0^0^\n",
        )
        locale = resources.parent / "locale.json"
        locale.write_text("[]", encoding="utf-8")
        catalogue = audit.build_catalogue(resources, locale)
        self.assertEqual(catalogue["counts"]["games"], 3)
        self.assertEqual(sum(t["games"] for t in catalogue["types"]), 3)
        self.assertEqual(sum(catalogue["engine_totals"].values()), 3)
        self.assertEqual(sum(catalogue["status_totals"].values()), 3)
        self.assertEqual(catalogue["counts"]["games_in_groups"], 2)

    def test_every_status_has_recorded_evidence(self):
        for type_id, record in audit.FAMILY_EVIDENCE.items():
            status = record["restoration_status"]
            self.assertIn(status, audit.STATUS_VOCABULARY, type_id)
            self.assertTrue(record["restoration_evidence"], type_id)


class OutputTests(unittest.TestCase):
    def test_cli_writes_catalogue_and_evidence_deterministically(self):
        resources = write_fixture_resources(
            pathlib.Path(tempfile.mkdtemp()) / "Resources",
            games=GAME_HEADER + "1^1^0^0^0^1^0^^0^0^0^^0^\n",
            types=TYPE_HEADER + "1^0^0^0^0^0^^0^0^0^0^0^0^1^\n",
        )
        locale = resources.parent / "locale.json"
        locale.write_text("[]", encoding="utf-8")
        root = resources.parent
        out = root / "catalogue.json"
        reports = root / "reports"
        catalog = root / "catalog.tsv"
        catalog.write_text(
            "pack\tentry\tname\toffset\tbytes\tcrc32\textracted_relative_path\n"
            "AssetsW_008.pack\t1399\tgame_mining.gfx\t99218999\t91300\tdf150264\tpack_08/001399.gfx\n",
            encoding="utf-8",
        )
        inventory = root / "inventory.json"
        inventory.write_text(json.dumps({"minigames": [], "source_sha256": {}}), encoding="utf-8")
        argv = ["--resources", str(resources), "--locale", str(locale),
                "--out", str(out), "--reports-dir", str(reports),
                "--catalog", str(catalog), "--inventory", str(inventory),
                "--stamp", "20260101T000000Z"]
        self.assertEqual(audit.main(argv), 0)
        first = out.read_bytes()
        self.assertEqual(audit.main(argv), 0)
        self.assertEqual(out.read_bytes(), first)
        catalogue = json.loads(first)
        self.assertEqual(catalogue["generated_utc"], "20260101T000000Z")
        evidence = reports / "activity-catalogue-20260101T000000Z"
        for name in ("catalogue.json", "findings.json", "provenance.json", "run.log"):
            self.assertTrue((evidence / name).is_file(), name)


class RealSourceTests(unittest.TestCase):
    def setUp(self):
        if not (RESOURCES / "MiniGameData.txt").is_file():
            self.skipTest("preserved client resources are not present")

    def test_real_source_hashes_match_recorded_provenance(self):
        for name, expected in EXPECTED_SOURCE_SHA256.items():
            actual = hashlib.sha256((RESOURCES / name).read_bytes()).hexdigest()
            self.assertEqual(actual, expected, name)

    def test_real_catalogue_counts_and_coverage(self):
        catalogue = audit.build_catalogue(RESOURCES, LOCALE)
        counts = catalogue["counts"]
        self.assertEqual(counts["games"], 783)
        self.assertEqual(counts["types_declared"], 22)
        self.assertEqual(counts["groups_declared"], 20)
        self.assertEqual(counts["group_links"], 610)
        self.assertEqual(counts["unnamed_games"], 44)
        self.assertEqual(sum(t["games"] for t in catalogue["types"]), 783)
        self.assertEqual(catalogue["findings"]["orphan_types"], ["0", "1009", "1011", "17"])
        self.assertEqual(catalogue["findings"]["duplicate_game_ids"], [])
        self.assertEqual(catalogue["findings"]["dangling_group_links"], [])
        self.assertEqual(catalogue["findings"]["source_hash_mismatches"], [])
        self.assertEqual(sum(catalogue["status_totals"].values()), 783)

    def test_real_catalogue_types_agree_with_existing_inventory(self):
        self.assertTrue(INVENTORY.is_file(), "inventory report missing")
        inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
        by_id = {row["#ID"]: row for row in inventory["minigames"]}
        catalogue = audit.build_catalogue(RESOURCES, LOCALE, inventory_path=INVENTORY)
        self.assertEqual(catalogue["findings"]["inventory_disagreements"], [])
        for game in catalogue["games"]:
            source = by_id[game["id"]]
            self.assertEqual(game["type"], source["TYPE"])
            self.assertEqual(game["name_id"], source["NAME_ID"])
            self.assertEqual(game["description_id"], source["DESCRIPTION_ID"])
            self.assertEqual(game["min_players"], source["MIN_PLAYERS"])
            self.assertEqual(game["max_players"], source["MAX_PLAYERS"])

    def test_real_asset_catalog_links_known_flash_movies(self):
        catalogue = audit.build_catalogue(RESOURCES, LOCALE, catalog_path=CATALOG)
        movies = catalogue["asset_references"]["game_movies"]
        self.assertIn("game_mining.gfx", movies)
        self.assertEqual(movies["game_mining.gfx"]["pack"], "AssetsW_008.pack")


class NativeReceiptTests(unittest.TestCase):
    """Regression: a row is native-accepted only with a captured receipt."""

    def _fixture(self, tmp):
        resources = write_fixture_resources(
            pathlib.Path(tmp) / "Resources",
            games=(GAME_HEADER
                    + "46^9^100^0^0^1^0^^0^0^0^^0^\n"
                    + "173^9^101^0^0^1^0^^0^0^0^^0^\n"),
            types=TYPE_HEADER + "9^0^0^1^0^0^^0^0^0^0^0^0^1^\n",
        )
        locale = resources.parent / "locale.json"
        locale.write_text(json.dumps([
            {"id": 100, "text": "Cooking: Strong Arm Stew"},
            {"id": 101, "text": "Cooking: Simone's Spiralmint Steak"},
        ]), encoding="utf-8")
        return audit.build_catalogue(resources, locale)

    def test_receipted_row_is_accepted_and_unreceipted_sibling_is_not(self):
        with tempfile.TemporaryDirectory() as tmp:
            catalogue = self._fixture(tmp)
        rows = {row["id"]: row for row in catalogue["games"]}
        self.assertEqual(rows["46"]["restoration_status"], audit.STATUS_NATIVE_ACCEPTED)
        self.assertEqual(rows["46"]["native_receipt"]["kinds"],
                         [audit.NATIVE_RECEIPT_ENTRY, audit.NATIVE_RECEIPT_COMPLETION, audit.NATIVE_RECEIPT_REWARD])
        self.assertEqual(rows["173"]["restoration_status"], audit.STATUS_CODE_TEST)
        self.assertIsNone(rows["173"]["native_receipt"])

    def test_receipts_are_scoped_to_source_activity_ids_with_existing_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            catalogue = self._fixture(tmp)
        receipt = catalogue["games"][0]["native_receipt"]
        self.assertTrue(receipt["captures"])
        self.assertTrue(receipt["evidence"])
        for path in receipt["evidence"]:
            self.assertTrue(audit.receipt_evidence_path(path).is_file(), path)
        self.assertEqual(catalogue["findings"]["native_receipt_evidence_missing"], [])


class RealNativeReceiptTests(unittest.TestCase):
    """The overclaim regression against the preserved 783-row corpus."""

    @classmethod
    def setUpClass(cls):
        if not (RESOURCES / "MiniGameData.txt").is_file():
            raise unittest.SkipTest("preserved client resources are not present")
        cls.catalogue = audit.build_catalogue(RESOURCES, LOCALE)

    def test_only_receipt_bound_rows_are_native_accepted(self):
        accepted = sorted(
            (row["id"] for row in self.catalogue["games"]
             if row["restoration_status"] == audit.STATUS_NATIVE_ACCEPTED),
            key=int,
        )
        self.assertEqual(accepted, sorted(audit.NATIVE_ROW_RECEIPTS, key=int))
        self.assertEqual(self.catalogue["status_totals"][audit.STATUS_NATIVE_ACCEPTED],
                         len(audit.NATIVE_ROW_RECEIPTS))
        for row in self.catalogue["games"]:
            if row["restoration_status"] == audit.STATUS_NATIVE_ACCEPTED:
                self.assertIn(audit.NATIVE_RECEIPT_ENTRY, row["native_receipt"]["kinds"], row["id"])
                self.assertIn(audit.NATIVE_RECEIPT_COMPLETION, row["native_receipt"]["kinds"], row["id"])
            else:
                self.assertIsNone(row["native_receipt"], row["id"])

    def test_family_movie_acceptance_stays_separate_from_row_acceptance(self):
        by_type = {record["type"]: record for record in self.catalogue["types"]}
        type_nine = by_type["9"]
        self.assertTrue(type_nine["family_gate"]["movie_accepted"])
        self.assertTrue(type_nine["family_gate"]["evidence"])
        self.assertEqual(type_nine["native_accepted_rows"], ["46", "77"])
        self.assertGreater(type_nine["row_status_counts"].get(audit.STATUS_CODE_TEST, 0), 0)
        cooking_row = next(row for row in self.catalogue["games"] if row["id"] == "173")
        self.assertEqual(cooking_row["restoration_status"], audit.STATUS_CODE_TEST)
        self.assertIsNone(cooking_row["native_receipt"])

    def test_type_id_math_separates_declared_empty_and_undeclared(self):
        math = self.catalogue["findings"]["type_id_math"]
        self.assertEqual(math["declared_count"], 22)
        self.assertEqual(sorted(math["declared_empty"], key=int), ["4", "18"])
        self.assertEqual(sorted(math["undeclared_referenced"], key=int), ["0", "17", "1009", "1011"])
        self.assertEqual(math["referenced_count"], 24)
        self.assertEqual(math["distinct_ids_across_tables"], 26)
        self.assertIn("20 with game rows", math["wording"])
        self.assertIn("4 undeclared", math["wording"])
        self.assertEqual(self.catalogue["counts"]["types_declared_with_games"], 20)
        self.assertEqual(self.catalogue["counts"]["types_undeclared_referenced"], 4)

    def test_forging_139_is_type8_and_not_described_as_mimic_type9(self):
        by_type = {record["type"]: record for record in self.catalogue["types"]}
        self.assertNotIn("forging", by_type["9"]["family"].lower())
        before = "\n".join(str(by_type["9"][key]) for key in ("family", "family_evidence", "restoration_evidence"))
        self.assertNotIn("anvil", before.lower())
        row = next(row for row in self.catalogue["games"] if row["id"] == "139")
        self.assertEqual(row["type"], "8")
        self.assertEqual(row["restoration_status"], audit.STATUS_NATIVE_ACCEPTED)
        binding = self.catalogue["findings"]["forging_row_139_binding"]
        self.assertEqual(binding["type"], "8")
        self.assertIn("local", binding["resolution"])
        self.assertTrue(binding["original_engine_class_unproven"])

    def test_smelting_77_receipt_records_the_captured_reward_and_turnin(self):
        receipt = next(
            entry for entry in self.catalogue["findings"]["native_acceptance"]["accepted_rows"]
            if entry["id"] == "77"
        )
        self.assertIn(audit.NATIVE_RECEIPT_REWARD, receipt["kinds"])
        self.assertEqual(receipt["kinds"],
                         [audit.NATIVE_RECEIPT_ENTRY, audit.NATIVE_RECEIPT_COMPLETION, audit.NATIVE_RECEIPT_REWARD])
        self.assertNotIn("no stage5 reward", receipt["caveat"])
        frames = "\n".join(receipt["frames"][capture] for capture in receipt["frames"])
        for frame in ("8850", "8852", "9090", "9093"):
            self.assertIn(frame, frames)
        self.assertIn("reconstructed", receipt["caveat"])
        self.assertTrue(any("reward-packets" in path for path in receipt["evidence"]))

    def test_no_catalogue_evidence_text_describes_forging_as_type9_mimic(self):
        """Every evidence string must avoid the stale TYPE 9 Mimic anvil claim."""
        off_markers = ("was wrong", "is removed", "not a TYPE 9", "no recovered binding", "unproven", "no TYPE 9")
        forge_tokens = ("forging", "forge", "anvil")
        mimic_tokens = ("mimic", "anvil", "type 9")
        offenders = []

        def walk(node, path):
            # The leftover-wording record intentionally quotes the stale piece
            # 018 text; it is a pointer for its owner, not a catalogue claim.
            if path == "catalogue.findings.leftover_wording_outside_ownership":
                return
            if isinstance(node, dict):
                for key, value in node.items():
                    walk(value, f"{path}.{key}")
            elif isinstance(node, list):
                for index, value in enumerate(node):
                    walk(value, f"{path}[{index}]")
            elif isinstance(node, str):
                lowered = node.lower()
                if (any(token in lowered for token in forge_tokens)
                        and any(token in lowered for token in mimic_tokens)
                        and not any(marker.lower() in lowered for marker in off_markers)):
                    offenders.append((path, node))

        walk(self.catalogue, "catalogue")
        self.assertEqual(offenders, [])

    def test_type_records_expose_strongest_row_status_not_family_acceptance(self):
        for record in self.catalogue["types"]:
            self.assertIn("strongest_row_status", record, record["type"])
            self.assertNotIn("restoration_status", record, record["type"])
            self.assertIn(record["strongest_row_status"], audit.STATUS_VOCABULARY)
        by_type = {record["type"]: record for record in self.catalogue["types"]}
        self.assertEqual(by_type["9"]["strongest_row_status"], audit.STATUS_NATIVE_ACCEPTED)
        self.assertEqual(by_type["9"]["native_accepted_rows"], ["46", "77"])
        self.assertEqual(by_type["9"]["family_gate"]["verified_rows"], ["46", "77"])
        self.assertEqual(
            sorted(
                (row["id"] for row in self.catalogue["games"]
                 if row["restoration_status"] == audit.STATUS_NATIVE_ACCEPTED),
                key=int,
            ),
            ["46", "77", "139"],
        )

    def test_leftover_forging_wording_outside_ownership_is_recorded(self):
        leftovers = self.catalogue["findings"]["leftover_wording_outside_ownership"]
        self.assertTrue(leftovers)
        entry = leftovers[0]
        self.assertEqual(entry["owner"], "piece 018")
        self.assertEqual(entry["path"], "prototype/osfr-tools/src/restoration/piece_018.rs")
        self.assertIn("TYPE 9 mimic anvil", entry["text"])
        self.assertIn("not edited", entry["action"])


if __name__ == "__main__":
    unittest.main()
