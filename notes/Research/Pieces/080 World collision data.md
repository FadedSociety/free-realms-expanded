# 080 World collision data

Work item 080: recover the native/community collision representation and compare it against actual
movement samples. **The GCNK collision section is recovered, parsed and compared against 281 recorded
movement/floor samples.** Native acceptance of the down-ray hit remains open; no gameplay behaviour is
claimed.

## What was already known

The pinned community reader `ForgeLightToolkit` revision `9c06714c5d79d9152c322814bc904abad944e06e`
decompresses the second zlib section of a `.gcnk` chunk and then stops:

```csharp
private bool LoadCollision(Stream collisionStream)
{
    // TODO: Bullet Physics
    return true;
}
```

[[Farnum Stage7 Route Geometry]] and [[Anvil Placement Repair Proposal]] therefore used render-terrain
slopes and mesh footprints as obstacle proxies. No community tool that decodes this section was found
(checked: ForgeLightToolkit, `psemu/forgelight-chunk` `cnk.js`, `ryanjsims/pydmod`, fr-adr-toolkit,
OpenSourceFreeRealms, Sanctuary). The format work below is therefore a genuine gap fill, not a duplicate.

## Original client evidence

The preserved original executable
`SHA256 26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` carries the Bullet
serialisation type names and the physics asset classes, so the section is Bullet-shaped data:

`btQuantizedBvhFloatData`, `btQuantizedBvhNodeData`, `btBvhSubtreeInfoData`,
`btOptimizedBvhNodeData`, `btTriangleMeshShapeData`, `btStridingMeshInterfaceData`,
`btShortIntIndexTripletData`, `btMeshPartData`, `btOptimizedBvh::build`, `btQuantizedBvh::vftable`,
`AppPhysics::CollisionData`, `AppPhysics::CollisionDataAsset`, `AppPhysics::MeshData`,
`GameRuntimeTerrain::DecompressChunkDataJob`. Log strings: `"Collision data %s for actor %s failed to
load!"` (`00775e20` → `00fdfab0`), `"Chunk (GCNK) %s:  Bad chunk version"` and
`"Chunk %s:  Unknown chunk type (Expected \"GCNK\" or \"GCK2\")"` (`00d4e6b0`).

Grounding path recovered earlier by [[Stage7 NPC Grounding Semantics]] and reused here: `010ba6e0`
casts vertically from `Y+2` down to `Y-2000`, uses collision query `00fd4290` with filter `0x143`, and
replaces only the movement target Y with the collision-hit Y. A leaf column containing the sample X/Z is
therefore the surface that query can hit.

## Recovered representation (original facts, asserted by tests)

`GCNK` container: magic, `int32` version 1..6, then two zlib sections, each preceded by
`(uncompressed, compressed)` lengths. Section 1 is the render terrain the community reader decodes;
section 2 is collision.

Section 2 is 16 per-tile payloads (the same 4x4 grid of 66-unit cells as the terrain tiles, one unit of
bleed on each side). Every payload is followed by a 16-byte zero-padded trailer block. Within a payload:

| Offset | Contents |
|---:|---|
| `+0` | `u32 payloadSize` (equals the payload length) |
| `+4 .. +31` | 28 zero bytes |
| `+32` | AABB min as a 4-float `btVector3FloatData` |
| `+48` | AABB max as a 4-float `btVector3FloatData` |
| `+64` | per-axis quantisation scale, `scale[i] == 65535 / (max[i] - min[i])` |
| `+80 .. +207` | 128-byte prefix (mostly zeros, a few unnamed small ints) |
| `+208` | `btQuantizedBvhNode` array: 6 x `u16` quantised AABB + `i32` escape-or-triangle, 16 bytes per node |
| `+208+16N` | `btBvhSubtreeInfo`-shaped headers: 6 x `u16` + `u32 rootNodeIndex` + `u32 subtreeSize` + 12 zero bytes, 32 bytes each |
| end - 16 | 16-byte trailer |

Node count `N` is the first node's escape index (`-nodes[0].escape_or_triangle`); the record at
`+208+16N` is the first subtree header, which is what pins the array boundary. World coordinates come
from `world = aabbMin + q / scale` (Bullet `btQuantizedBvh::unQuantize`); the stored `float32` scale is
not exactly `65535/span`, so a full-range corner lands up to ~1.5e-3 units outside the float AABB.

Verified on 64 tiles across four preserved starting-area chunks (payloads 15408/24016/19248/16240/
18608/42352/22448/13328/14864/25968/20752/21872/10064/22064/33232/38704 in
`FabledRealms_4_-32`): every payload's node count equals its root escape, `payloadSize - 224 - 16N` is
always a multiple of 32, and every tile's X/Z span is 66.0 units.

## Movement-sample comparison

`scripts/pieces/piece_080.py` resolves the preserved chunk behind each recorded sample's render mesh,
finds the collision tile containing the sample X/Z, and reports every leaf whose dequantised X/Z box
contains that column. Samples: `darkthorne-journal-floor.json`, `route-floor-samples.json`,
`actor-floor-points.json`, `artifact-floor-points.json` (281 rows over 3 collision chunks).

**Result:** 281/281 columns contain collision leaves; the render floor lies inside the collision band for
281/281 (maximum offset outside a band `0.0`); median leaf hits per column `2`; median band width
`0.567` units, maximum `6.006`; `171/281` recorded sample heights fall inside the band, with a maximum
`|native - render floor|` of `24.813` (the known farm plateau/cliff rows where the recorded position is
not standing on the render floor). The collision surface therefore tracks the render terrain floor
closely while staying a separate Bullet tree, and the recorded-position outliers are the same
native/render discrepancies already documented for the farm.

## Reproduction

```powershell
# Checker + private bundle (schema, comparison, provenance, findings)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_080.py' `
    --out '..\lab\reports\piece-080-20261009T044143Z\bundle2'

# Python acceptance (10 tests, RED first then GREEN)
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces `
    -p "test_piece_080.py" -v

# Rust acceptance (4 tests, includes the preserved original chunk)
& '.\scripts\cargo.ps1' test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_080
```

Evidence: `../lab/reports/piece-080-20261009T044143Z/` (`bundle2/schema.json`, `bundle2/comparison.json`,
`bundle2/findings.json`, `bundle2/provenance.json`, `python-piece-080-tests.log`,
`rust-piece-080-tests.log`, plus the `_scratch-*.py` probes that recovered the schema).
`bundle/` and `run1/` are the earlier runs kept for comparison; they predate the band-width metrics and
the final `--out` naming.

## Boundaries and gaps

* **Original facts:** container layout, 16-tile payload grid, AABB + `65535/span` scale, node/subtree
  record sizes, the client's Bullet type names and grounding call chain.
* **Inferred:** that leaves index the chunk mesh's triangles and that a leaf AABB is the conservative
  box around its triangle. The triangle/vertex payload itself was not decoded.
* **Reconstructed:** nothing grants rewards or changes gameplay; only the reader and the dequantisation
  rule are reconstructed from Bullet's documented behaviour.
* **Gaps:** unnamed small integers in the 128-byte payload prefix; the exact semantics of the 16-byte
  trailer blocks (values 103, 59, 121, ... and 67 for `FabledRealms_4_-32`); the vertex/triangle arrays a
  leaf index refers to; exact triangle hit heights; static prop (`.dme`) and dynamic actor collision.
* **Native gate:** no live client down-ray has been captured against these surfaces. A native check would
  stand a character in a known column, capture the reported Y and compare it with the leaf band; that is
  lead-owned.
