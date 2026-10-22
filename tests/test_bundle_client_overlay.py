"""T575: a bundle carries the pinned creator overlay as its client patch folder, or is not built."""

from __future__ import annotations

import hashlib
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "bundle"))

import bundle_common  # noqa: E402
import client_overlay  # noqa: E402
import make_bundle  # noqa: E402


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


@unittest.skipUnless(client_overlay.CACHE.is_dir(), "needs the lab asset cache")
class BundleClientOverlayTest(unittest.TestCase):
    def setUp(self):
        self.bundle = Path(tempfile.mkdtemp(prefix="osfr-bundle-overlay-"))

    def tearDown(self):
        shutil.rmtree(self.bundle)

    def build(self) -> str:
        """make_bundle.main's two steps, in its order."""
        make_bundle.copy_fixed_inputs(self.bundle)
        return make_bundle.copy_client_overlay(self.bundle)

    def test_bundle_serves_the_22_pinned_files_and_records_the_patch_pin(self):
        pin = self.build()
        overlay = self.bundle / "client-overlay"
        manifest, count = bundle_common.verify_manifest(overlay)  # what W121 accepts
        self.assertEqual(count, 22)
        self.assertEqual(
            {entry["path"]: entry["sha256"] for entry in manifest["files"]},
            {path: file_pin for _, path, file_pin in client_overlay.OVERLAY},
        )
        for _, path, file_pin in client_overlay.OVERLAY:
            self.assertEqual(sha256(overlay / path), file_pin, path)
        self.assertEqual(pin, sha256(overlay / "manifest.json"))
        bundle_manifest = bundle_common.make_manifest(self.bundle)
        self.assertEqual(set(bundle_manifest), {"version", "files"})  # W121 rejects extra keys
        recorded = {entry["path"]: entry["sha256"] for entry in bundle_manifest["files"]}
        self.assertEqual(recorded["client-overlay/manifest.json"], pin)

    def test_login_resources_carry_the_allow_list_inputs(self):
        self.build()
        resources = self.bundle / "login-resources"
        mappings = "CharacterCreate/CharacterCreateItemMappings.txt"
        self.assertEqual(
            (resources / mappings).read_bytes(),
            (bundle_common.CLIENT_RESOURCES / mappings).read_bytes(),
        )
        tables = [
            (path.removeprefix("Resources/"), file_pin)
            for _, path, file_pin in client_overlay.OVERLAY
            if path.startswith("Resources/CharacterCreateCustomization")
        ]
        self.assertEqual(len(tables), 2)  # SkinTone, FaceType
        for name, file_pin in tables:
            self.assertEqual(sha256(resources / name), file_pin, name)
            # The trimmed community table, not the stock client's.
            self.assertNotEqual(sha256(bundle_common.CLIENT_RESOURCES / name), file_pin, name)

    def test_the_security_scan_ignores_creator_tables_in_harness_runs(self):
        # Each native run's client tree holds the overlay (the harness installs it) and the
        # stock item mappings, byte for byte what the bundle ships; a save beside them counts.
        lab = self.bundle / "lab"
        run = lab / "runs" / "client-baseline-x"
        client_overlay.install(run / "client")
        mappings = "CharacterCreate/CharacterCreateItemMappings.txt"
        shutil.copyfile(bundle_common.CLIENT_RESOURCES / mappings, run / "client" / "Resources" / mappings)
        (run / "character.json").write_text("{}", encoding="utf-8")
        with mock.patch.object(bundle_common, "LAB", lab), mock.patch.object(
            bundle_common, "ACTIVE_PACKAGE", lab / "package"
        ):
            self.assertEqual(list(bundle_common.protected_source_files()), [run / "character.json"])

    def test_a_cached_file_off_its_pin_writes_no_overlay(self):
        make_bundle.copy_fixed_inputs(self.bundle)
        before = sorted(p.relative_to(self.bundle) for p in self.bundle.rglob("*"))
        name, path, _ = client_overlay.OVERLAY[-1]
        broken = (*client_overlay.OVERLAY[:-1], (name, path, "0" * 64))
        with mock.patch.object(client_overlay, "OVERLAY", broken):
            with self.assertRaisesRegex(ValueError, "does not match its pin"):
                make_bundle.copy_client_overlay(self.bundle)
        self.assertEqual(sorted(p.relative_to(self.bundle) for p in self.bundle.rglob("*")), before)


if __name__ == "__main__":
    unittest.main()
