"""Catalogue the world objects placed in the client's terrain chunks (.gcnk).

Each GCNK (version 6) holds a zlib-compressed chunk body: tiles, each listing
runtime objects (model file, position, rotation, scale, object id), lights,
areas and groups. Layout follows the MIT-licensed ForgeLightToolkit
(lab/references/ForgeLightToolkit/Editor/FileTypes/Gcnk); this is our own parser.
Only the tile section is read; meshes and collision are skipped.

Usage: world_objects.py <dir with .gcnk files, searched recursively> <out.json>
"""
import hashlib
import json
import struct
import sys
import zlib
from pathlib import Path


class Reader:
    def __init__(self, data):
        self.d = data
        self.p = 0

    def i32(self):
        v = struct.unpack_from('<i', self.d, self.p)[0]; self.p += 4; return v

    def f32(self):
        v = struct.unpack_from('<f', self.d, self.p)[0]; self.p += 4; return v

    def vec(self, n):
        v = struct.unpack_from('<%df' % n, self.d, self.p); self.p += 4 * n; return list(v)

    def byte(self):
        v = self.d[self.p]; self.p += 1; return v

    def skip(self, n):
        self.p += n

    def cstr(self):
        end = self.d.index(b'\0', self.p)
        s = self.d[self.p:end].decode('latin1'); self.p = end + 1; return s


def parse_tiles(body, version):
    r = Reader(body)
    tiles = []
    for _ in range(r.i32()):
        coords = (r.i32(), r.i32())
        r.vec(4)  # tile position
        if r.i32() > 0:
            r.skip(16)
        r.f32()
        r.skip(4 * r.i32())  # eco data ids
        objects = []
        for _ in range(r.i32()):
            r.i32()
            name = r.cstr()
            extra = r.cstr()
            pos = r.vec(4)
            rot = r.vec(4)
            scale = r.f32()
            material = tint = ''
            if version >= 6:
                material = r.cstr()
                tint = r.cstr()
                if tint:
                    r.vec(4)
            r.skip(4)
            object_id = r.i32()
            r.i32()
            if version > 2:
                for _ in range(r.i32()):
                    r.skip(4 * r.i32())
            objects.append({"file": name, "extra": extra, "position": pos[:3], "rotation": rot,
                            "scale": scale, "material": material, "tint": tint, "object_id": object_id})
        lights = []
        for _ in range(r.i32()):
            lname = r.cstr(); r.cstr(); r.byte(); lpos = r.vec(4); r.f32(); r.f32(); r.skip(4)
            lights.append({"name": lname, "position": lpos[:3]})
        areas = []
        for _ in range(r.i32()):
            aname = r.cstr(); r.i32(); akind = r.cstr(); a4 = r.vec(4); a5 = r.vec(4); r.i32(); r.vec(3)
            areas.append({"name": aname, "kind": akind, "a": a4[:3], "b": a5[:3]})
        groups = []
        for _ in range(r.i32()):
            gname = r.cstr(); g2 = r.vec(4); r.vec(4); r.i32()
            groups.append({"name": gname, "position": g2[:3]})
        r.i32(); r.skip(4)
        tiles.append({"coords": coords, "objects": objects, "lights": lights,
                      "areas": areas, "groups": groups})
    return tiles


def parse_gcnk(data):
    if data[:4] != b'GCNK':
        raise ValueError('not a GCNK file')
    version, raw_len, comp_len = struct.unpack_from('<iii', data, 4)
    if not 0 <= version <= 6:
        raise ValueError(f'unsupported GCNK version {version}')
    body = zlib.decompress(data[16:16 + comp_len])
    if len(body) != raw_len:
        raise ValueError('chunk length mismatch')
    return version, parse_tiles(body, version)


def main():
    root, out = Path(sys.argv[1]), Path(sys.argv[2])
    seen = set()
    chunks, errors = [], []
    for path in sorted(root.rglob('*.gcnk')):
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        if digest in seen:
            continue
        seen.add(digest)
        try:
            version, tiles = parse_gcnk(data)
        except Exception as error:  # report and continue: one bad chunk must not hide the rest
            errors.append({"file": str(path), "error": str(error)})
            continue
        chunks.append({"file": path.name, "path": str(path), "sha256": digest,
                       "version": version, "tiles": tiles})
    objects = sum(len(t['objects']) for c in chunks for t in c['tiles'])
    out.write_text(json.dumps({"chunks": chunks, "errors": errors}), encoding='utf-8')
    print(json.dumps({"chunks": len(chunks), "objects": objects, "errors": len(errors)}))


if __name__ == '__main__':
    main()
