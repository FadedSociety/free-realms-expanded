import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from asset_delivery import (AssetDelivery, _pack_path, baseline_ini, manifest_crc_body,
                            manifest_wire)

CATALOG_HEADER = 'pack\tentry\tname\toffset\tbytes\tcrc32\textracted_relative_path\n'


def pack_layout(entries):
    """One big-endian directory block (next=0, count, entries) with the raw bytes after
    it, plus the `osfr-tools catalog` rows for those entries."""
    head = struct.pack('>II', 0, len(entries))
    offset = len(head) + sum(4 + len(name.encode('utf-8')) + 12 for name in entries)
    directory, payload, rows = bytearray(head), bytearray(), []
    for name, raw in entries.items():
        encoded = name.encode('utf-8')
        directory += struct.pack('>I', len(encoded)) + encoded
        directory += struct.pack('>III', offset + len(payload), len(raw), zlib.crc32(raw))
        rows.append((name, offset + len(payload), len(raw), zlib.crc32(raw)))
        payload += raw
    return bytes(directory + payload), rows


class AssetDeliveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        extracted = self.root / 'extracted/client-assets'
        extracted.mkdir(parents=True)
        (extracted / 'catalog.tsv').write_text(CATALOG_HEADER)
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

class PackNameRuleTests(unittest.TestCase):
    """`_pack_path` is the launcher patch's `patch::valid_path` shape rule."""

    def test_pack_name_allowlist_matches_the_launcher_valid_path_rule(self):
        allowed = ['images/card/1.jpg', 'flat.dds', 'a/b c.dds', 'x' * 220]
        refused = ['', 'a//b', 'a/.hidden', 'a/b.', 'a/b ', 'CON', 'con.txt', 'COM1.dds',
                   'LPT9.dds', 'part.osfr-part', 'C:/x.dds', 'a\\b.dds', 'x' * 221]
        for name in allowed:
            self.assertTrue(_pack_path(name), name)
        for name in refused:
            self.assertFalse(_pack_path(name), name)


class ClientPackTests(unittest.TestCase):
    """The client's own downloaded TCG pack (`assets/assetpack000*`) is a second allowlisted source."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.extracted = self.root / 'extracted/client-assets'
        self.extracted.mkdir(parents=True)
        self.catalog = [CATALOG_HEADER]
        self.client = self.root / 'client'
        (self.client / 'assets').mkdir(parents=True)
        self.card = b'\xff\xd8\xff\xe0 synthetic card'
        self.card2 = b'\xff\xd8\xff\xe0 synthetic card two'
        self.card3 = b'\xff\xd8\xff\xe0 synthetic card three'
        self.stale = b'\xff\xd8\xff\xe0 an older card'
        self.foil = b'synthetic foil'
        self.other = b'unrelated bytes'
        self.pack_flat = b'pack flat bytes'
        self.base_flat = b'synthetic base flat'
        self.base_filled = b'synthetic base filled'
        self.packed_raw = b'base compressed-looking row'
        # The first pack holds `images/card/3.jpg` at an older CRC; the second holds the row's.
        self.write_pack('assetpack000W_000.pack', {
            'images/card/1.jpg': self.card,
            'images/card/2.jpg': self.card2,
            'images/card/3.jpg': self.stale,
            'images/card/wrong-size.jpg': self.card3,
            'images/foils/1.dds': self.foil,
            'images/secret/1.png': self.other,
            'flat.dds': self.pack_flat,
            'packed.dds': self.packed_raw,
        })
        self.write_pack('assetpack000W_001.pack', {
            'images/card/3.jpg': self.card3,
            'filled.dds': self.base_filled,
        })
        # A lab source for a base name; the pack's `flat.dds` must not shadow it.
        (self.extracted / 'flat.dds').write_bytes(self.base_flat)
        self.catalog.append(f'AssetsW_000.pack\t0\tflat.dds\t0\t{len(self.base_flat)}'
                            f'\t{zlib.crc32(self.base_flat):08x}\tflat.dds')
        # Catalog rows that name bytes outside the pack and a row over the size bound.
        self.catalog.append(f'assets/assetpack000W_000.pack\t99\timages/huge.bin\t0'
                            f'\t{16 * 1024 * 1024 + 1}\t{zlib.crc32(b"huge"):08x}\tpack_00/000099.bin')
        self.catalog.append(f'assets/assetpack000W_000.pack\t98\timages/outside.bin\t1000000'
                            f'\t7\t{zlib.crc32(b"outside"):08x}\tpack_00/000098.bin')
        (self.extracted / 'catalog.tsv').write_text('\n'.join(self.catalog) + '\n', encoding='utf-8')
        rows = [
            f'images/card/1.jpg,{zlib.crc32(self.card)},{len(self.card)}',
            # A `.z` row is announced raw: the pack holds the raw bytes.
            f'images/card/2.jpg.z,{zlib.crc32(self.card2)},65535',
            f'images/card/3.jpg,{zlib.crc32(self.card3)},{len(self.card3)}',
            f'images/card/wrong-size.jpg,{zlib.crc32(self.card3)},{len(self.card3) + 1}',
            f'images/foils/1.dds,{zlib.crc32(self.foil) + 1},{len(self.foil)}',
            '../escape.jpg,1,2',
            'images/secret/1.png,not-a-crc,3',
            f'flat.dds,{zlib.crc32(self.pack_flat)},{len(self.pack_flat)}',
            f'images/huge.bin,{zlib.crc32(b"huge")},4',
            f'images/outside.bin,{zlib.crc32(b"outside")},7',
        ]
        (self.client / 'assets/assetpack000_manifest.txt').write_text(
            '\r\n'.join(rows) + '\r\n', encoding='utf-8')
        self.base = (f'flat.dds,{zlib.crc32(self.base_flat)},{len(self.base_flat)}\r\n'
                     f'filled.dds,{zlib.crc32(self.base_filled)},{len(self.base_filled)}\r\n'
                     f'packed.dds.z,{zlib.crc32(self.packed_raw)},{len(self.packed_raw)}\r\n').encode()
        self.delivery = AssetDelivery(self.root, self.base, self.client)

    def write_pack(self, filename, entries):
        data, rows = pack_layout(entries)
        (self.client / 'assets' / filename).write_bytes(data)
        for entry, (name, offset, size, crc) in enumerate(rows):
            self.catalog.append(f'assets/{filename}\t{entry}\t{name}\t{offset}\t{size}'
                                f'\t{crc:08x}\tpack_00/{entry:06}.bin')

    def tearDown(self):
        self.temp.cleanup()

    def request(self, name, raw):
        return f'/562/{name}?{zlib.crc32(raw)}'

    def test_only_rows_the_pack_holds_with_matching_crc_are_offered(self):
        self.assertEqual(self.delivery.client_rows, 4)
        served = self.delivery.served_manifest.decode('ascii')
        self.assertIn(f'images/card/1.jpg,{zlib.crc32(self.card)},{len(self.card)}\r\n', served)
        self.assertIn(f'images/card/2.jpg,{zlib.crc32(self.card2)},{len(self.card2)}\r\n', served)
        self.assertIn(f'images/card/3.jpg,{zlib.crc32(self.card3)},{len(self.card3)}\r\n', served)
        self.assertNotIn('images/card/2.jpg.z', served)
        for refused in ('images/card/wrong-size.jpg', 'images/foils/1.dds', 'escape',
                        'images/secret/1.png', 'images/huge.bin', 'images/outside.bin'):
            self.assertNotIn(refused, served)
        self.assertEqual(served.count('flat.dds,'), 1)
        self.assertIn(f'flat.dds,{zlib.crc32(self.base_flat)},{len(self.base_flat)}\r\n', served)
        self.assertEqual(served.count('filled.dds,'), 1)
        self.assertIn(f'filled.dds,{zlib.crc32(self.base_filled)},{len(self.base_filled)}\r\n', served)
        self.assertTrue(served.startswith(self.base.decode('ascii')))
        self.assertNotIn('\r\n\r\n', served)
        crc, length = manifest_crc_body(self.delivery.served_manifest).decode('ascii').strip().split(',')
        self.assertEqual((int(crc), int(length)),
                         (zlib.crc32(self.delivery.served_manifest), len(self.delivery.served_manifest)))
        magic, raw_length = struct.unpack('>II', manifest_wire(self.delivery.served_manifest)[:8])
        self.assertEqual((magic, raw_length), (0xA1B2C3D4, len(self.delivery.served_manifest)))
        self.assertEqual(zlib.decompress(manifest_wire(self.delivery.served_manifest)[8:]),
                         self.delivery.served_manifest)

    def test_manifest_routes_describe_the_same_merged_manifest(self):
        routes = self.delivery.manifest_routes()
        self.assertEqual(routes['/manifest.crc'],
                         manifest_crc_body(self.delivery.served_manifest))
        self.assertEqual(routes['/Assets_manifest.txt'], self.delivery.served_manifest)
        self.assertEqual(zlib.decompress(routes['/manifest.txt.z'][8:]),
                         self.delivery.served_manifest)

    def test_pack_row_resolves_from_the_pack_holding_the_announced_crc(self):
        body, proof = self.delivery.get(self.request('images/card/3.jpg', self.card3))
        self.assertEqual(body, self.card3)
        self.assertEqual((proof['source'], proof['raw_crc32']),
                         ('client-pack', zlib.crc32(self.card3)))
        # The first pack's older version of the same name is not answered.
        self.assertIsNone(self.delivery.get(self.request('images/card/3.jpg', self.stale)))

    def test_base_row_without_a_lab_source_is_filled_from_the_pack(self):
        body, proof = self.delivery.get(self.request('filled.dds', self.base_filled))
        self.assertEqual(body, self.base_filled)
        self.assertEqual((proof['source'], proof['encoding']), ('client-pack', 'raw'))

    def test_a_compressed_base_row_is_not_filled_from_a_raw_pack_entry(self):
        self.assertEqual(self.delivery.client_rows, 4)
        crc = zlib.crc32(self.packed_raw)
        self.assertIsNone(self.delivery.get(f'/562/packed.dds.z?{crc}'))
        self.assertIsNone(self.delivery.get(f'/562/packed.dds?{crc}'))

    def test_base_row_with_a_lab_source_keeps_it(self):
        body, proof = self.delivery.get(self.request('flat.dds', self.base_flat))
        self.assertEqual(body, self.base_flat)
        self.assertEqual(proof['source'], 'lab')
        self.assertIsNone(self.delivery.get(self.request('flat.dds', self.pack_flat)))

    def test_allowlisted_pack_names_are_served_raw_and_compressed(self):
        body, proof = self.delivery.get(self.request('images/card/1.jpg', self.card))
        self.assertEqual(body, self.card)
        self.assertEqual((proof['name'], proof['source'], proof['encoding']),
                         ('images/card/1.jpg', 'client-pack', 'raw'))
        body, proof = self.delivery.get(self.request('images/card/2.jpg', self.card2))
        self.assertEqual(body, self.card2)
        body, proof = self.delivery.get(self.request('images/card/2.jpg.z', self.card2))
        magic, raw_length = struct.unpack('>II', body[:8])
        self.assertEqual((magic, raw_length), (0xA1B2C3D4, len(self.card2)))
        self.assertEqual(zlib.decompress(body[8:]), self.card2)
        self.assertEqual(proof['encoding'], 'native-zlib')

    def test_pack_requests_outside_the_allowlist_are_refused_and_change_nothing(self):
        before = self.delivery.served_manifest
        refused = [
            self.request('images/card/3.jpg', self.stale),  # another pack entry's CRC
            self.request('images/foils/1.dds', self.foil),  # CRC mismatch row
            self.request('images/card/wrong-size.jpg', self.card3),  # row length disagrees
            self.request('images/secret/1.png', self.other),  # unsafe client row
            '/562/images/card/1.jpg?0',  # wrong query CRC
            f'/562/../card/1.jpg?{zlib.crc32(self.card)}',
            f'/562/images/%2e%2e/card/1.jpg?{zlib.crc32(self.card)}',
            f'/562/images/card/1.jpg%5c..?{zlib.crc32(self.card)}',
            f'/x/images/card/1.jpg?{zlib.crc32(self.card)}',
            f'/562/images/huge.bin?{zlib.crc32(b"huge")}',
            f'/562/images/outside.bin?{zlib.crc32(b"outside")}',
        ]
        for request in refused:
            self.assertIsNone(self.delivery.get(request), request)
        self.assertEqual(self.delivery.served_manifest, before)
        self.assertEqual(self.delivery.client_rows, 4)
        self.assertEqual(self.delivery.get(self.request('images/card/1.jpg', self.card))[0], self.card)

    def test_pack_bytes_lost_after_startup_are_never_trusted(self):
        (self.client / 'assets/assetpack000W_000.pack').write_bytes(b'')
        before = self.delivery.served_manifest
        with self.assertRaises(ValueError):
            self.delivery.get(self.request('images/card/1.jpg', self.card))
        self.assertEqual(self.delivery.served_manifest, before)
        self.assertEqual(self.delivery.get(self.request('flat.dds', self.base_flat))[0], self.base_flat)

    def test_without_a_client_pack_the_manifest_is_served_unchanged(self):
        delivery = AssetDelivery(self.root, self.base)
        self.assertEqual(delivery.served_manifest, self.base)
        self.assertEqual(delivery.client_rows, 0)
        self.assertIsNone(delivery.get(f'/562/images/card/1.jpg?{zlib.crc32(self.card)}'))


if __name__ == '__main__':
    unittest.main()
