import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from asset_delivery import AssetDelivery, baseline_ini, manifest_crc_body, manifest_wire


class AssetDeliveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        extracted = self.root / 'extracted/client-assets'
        extracted.mkdir(parents=True)
        (extracted / 'catalog.tsv').write_text('name\tcrc32\textracted_relative_path\tbytes\n')
        self.cache = self.root / 'references/asset-cache/http-raw'
        self.cache.mkdir(parents=True)
        self.raw = {'track.gcnk': b'synthetic geometry', 'kart.cdt': b'<synthetic/>', 'wall1&2.dds': b'synthetic texture', 'body_decal^_goldcup.dds': b'synthetic decal'}
        rows = []
        lines = []
        for name, raw in self.raw.items():
            (self.cache / name).write_bytes(raw)
            rows.append(dict(name=name, file=name, bytes=len(raw), crc32=zlib.crc32(raw), sha256=hashlib.sha256(raw).hexdigest()))
            key = name if name.endswith('.gcnk') else name + '.z'
            wire = raw if name.endswith('.gcnk') else struct.pack('>II', 0xA1B2C3D4, len(raw)) + zlib.compress(raw)
            lines.append(f'{key},{zlib.crc32(raw)},{len(wire)}')
        (self.cache / 'index.json').write_text(json.dumps(dict(assets=rows)))
        self.manifest = '\n'.join(lines).encode()
        self.delivery = AssetDelivery(self.root, self.manifest)

    def tearDown(self):
        self.temp.cleanup()

    def request(self, name):
        raw = self.raw[name]
        key = name if name.endswith('.gcnk') else name + '.z'
        return f'/562/{key}?{zlib.crc32(raw)}'

    def test_raw_geometry_is_verbatim_and_compressed_assets_keep_their_envelope(self):
        body, proof = self.delivery.get(self.request('track.gcnk'))
        self.assertEqual(body, self.raw['track.gcnk'])
        self.assertEqual(proof['name'], 'track.gcnk')
        self.assertEqual(proof['encoding'], 'raw')
        body, proof = self.delivery.get(self.request('kart.cdt'))
        self.assertEqual(struct.unpack('>II', body[:8]), (0xA1B2C3D4, len(self.raw['kart.cdt'])))
        self.assertEqual(zlib.decompress(body[8:]), self.raw['kart.cdt'])
        self.assertEqual(proof['name'], 'kart.cdt')
        body, _ = self.delivery.get(self.request('wall1&2.dds'))
        self.assertEqual(zlib.decompress(body[8:]), self.raw['wall1&2.dds'])

    def test_wrong_crc_unknown_asset_and_path_escape_are_refused(self):
        for request in ['/562/track.gcnk?0', '/562/missing.gcnk?1', '/562/../track.gcnk?1', '/562/%2e%2e%2ftrack.gcnk?1', '/x/track.gcnk?1']:
            self.assertIsNone(self.delivery.get(request))

    def test_native_caret_decal_names_are_delivered_with_the_same_crc_guard(self):
        request = self.request('body_decal^_goldcup.dds')
        for path in [request, request.replace('^', '%5E')]:
            body, proof = self.delivery.get(path)
            self.assertEqual(zlib.decompress(body[8:]), self.raw['body_decal^_goldcup.dds'])
            self.assertEqual(proof['name'], 'body_decal^_goldcup.dds')
        self.assertIsNone(self.delivery.get(request.rsplit('?', 1)[0] + '?0'))

    def test_corrupted_geometry_is_never_delivered(self):
        (self.cache / 'track.gcnk').write_bytes(b'X' * len(self.raw['track.gcnk']))
        with self.assertRaises(ValueError):
            self.delivery.get(self.request('track.gcnk'))

    def test_plain_geometry_must_match_the_manifest_length(self):
        manifest = self.manifest.replace(str(len(self.raw['track.gcnk'])).encode(), b'999', 1)
        delivery = AssetDelivery(self.root, manifest)
        with self.assertRaises(ValueError):
            delivery.get(self.request('track.gcnk'))

    def test_recovered_source_cannot_escape_the_cache(self):
        doc = json.loads((self.cache / 'index.json').read_text())
        doc['assets'][0]['file'] = '../outside.gcnk'
        (self.cache / 'index.json').write_text(json.dumps(doc))
        with self.assertRaises(ValueError):
            AssetDelivery(self.root, self.manifest)


class CardAssetSetTests(unittest.TestCase):
    """The card window's own asset set (`TcgServerAddress`) is answered by the harness server."""

    CONFIG = (
        '[Network]\nServer=old.example:1\n[AssetDelivery]\n'
        'IndirectServerAddress=http://assets.example/assets\n'
        'TcgServerAddress=http://patch.example/freerealmstcg/live/client\n'
        '[Portrait]\nUploadUrl=http://portrait.example/upload\n[CrashReporter]\nAddress=lvsadp-x.example\n'
    )

    def test_card_assets_point_at_the_harness_server_and_every_other_url_stays_offline(self):
        lines = baseline_ini(self.CONFIG, '127.0.0.1:4000', 'http://127.0.0.1:5000/').splitlines()
        self.assertIn('Server=127.0.0.1:4000', lines)
        self.assertIn('IndirectServerAddress=http://127.0.0.1:5000/', lines)
        self.assertIn('TcgServerAddress=http://127.0.0.1:5000/tcg', lines)
        self.assertIn('UploadUrl=http://127.0.0.1:9/', lines)
        self.assertIn('Address=127.0.0.1', lines)

    def test_manifest_crc_is_the_raw_manifest_crc_and_length_the_client_compares_locally(self):
        manifest = b'a.dat.z,1,2\r\nb.dat.z,3,4\r\n'
        crc, length = manifest_crc_body(manifest).decode('ascii').strip().split(',')
        self.assertEqual((int(crc), int(length)), (zlib.crc32(manifest), len(manifest)))
        magic, raw_length = struct.unpack('>II', manifest_wire(manifest)[:8])
        self.assertEqual((magic, raw_length), (0xA1B2C3D4, len(manifest)))
        self.assertEqual(zlib.decompress(manifest_wire(manifest)[8:]), manifest)


if __name__ == '__main__':
    unittest.main()
