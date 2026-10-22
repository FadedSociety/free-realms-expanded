"""The client overlay holds exactly the pinned creator files at the paths the client opens, or nothing."""

from __future__ import annotations

import shutil
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "bundle"))

import bundle_common  # noqa: E402
import client_overlay  # noqa: E402

SKIN_TONE = "public-CharacterCreateCustomizationSkinTone.txt"


class ClientOverlayTest(unittest.TestCase):
    def setUp(self):
        self.temp = Path(tempfile.mkdtemp(prefix="osfr-client-overlay-"))
        self.out = self.temp / "overlay"
        self.client = self.temp / "client"

    def tearDown(self):
        shutil.rmtree(self.temp)

    def test_overlay_is_what_the_harness_installs(self):
        manifest = client_overlay.build(self.out)
        bundle_common.verify_manifest(self.out)
        self.assertEqual(
            [entry["path"] for entry in manifest["files"]],
            sorted(path for _, path, _ in client_overlay.OVERLAY),
        )
        client_overlay.install(self.client)  # the harness's call
        for name, path, _ in client_overlay.OVERLAY:
            bundle_common.safe_manifest_path(path)  # the launcher accepts it
            data = (client_overlay.CACHE / name).read_bytes()
            self.assertEqual((self.out / path).read_bytes(), data, path)
            self.assertEqual((self.client / path).read_bytes(), data, path)
            if path.startswith("Resources/"):  # replaces a table the shipped client opens
                self.assertTrue((bundle_common.CLIENT_RESOURCES.parent / path).is_file(), path)

    def test_tampered_table_is_refused_and_nothing_written(self):
        cache = self.temp / "cache"
        cache.mkdir()
        for name, _, _ in client_overlay.OVERLAY:
            shutil.copyfile(client_overlay.CACHE / name, cache / name)
        table = cache / SKIN_TONE
        table.write_bytes(table.read_bytes() + b"1000001^6562^551^272^Pixie^Female^429660^Ivory^\r\n")
        with self.assertRaisesRegex(ValueError, "SkinTone"):
            client_overlay.build(self.out, cache)
        self.assertFalse(self.out.exists())
        self.client.mkdir()
        with self.assertRaisesRegex(ValueError, "SkinTone"):
            client_overlay.install(self.client, cache)
        self.assertEqual(list(self.client.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
