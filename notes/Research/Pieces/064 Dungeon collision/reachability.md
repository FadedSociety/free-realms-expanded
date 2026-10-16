# 064 Dungeon collision/reachability

The starting cave ships real collision assets. This piece recovers their container prefix, shows
where that prefix agrees with the community reader's render mesh, and shows why *render bounds are
not collision proof*: all 456 recorded native cave positions sit on a render surface, and none of
them sits on a surface from the parsed collision prefix. Read that together with
[[Starting Dungeon Audit]] - the render-floor numbers there are render mesh samples, not a
collision or physics result. Nothing here is a playable claim.

## Original recovered facts

- **Actor-to-collision binding is first-party.** `sg_npx_cavern_01_room_01.adr`
  (`pack_19/001776.adr`, sha256 `69376ab0…76a8`) contains the literal ASCII string
  `sg_npx_cavern_01_room_01.cdt`. The client asset manifest carries the matching pair
  `sg_npx_cavern_01_room_01.adr.z` / `sg_npx_cavern_01_room_01.cdt.z`. The same pairing appears for
  generic actors (`visible_cube.adr` + `visible_cube.cdt`, `invisible_cube_collision.cdt`), so `*.cdt`
  is the per-actor collision companion rather than cave-only art.
- **The cave has 11 preserved collision assets**, all beginning with the ASCII magic `CDTA`:

| Asset | Pack entry | Bytes | SHA-256 (prefix) |
| --- | --- | --- | --- |
| `sg_npx_cavern_01_room_01.cdt` | `pack_01/001632.cdt` | 394506 | `cc9a0d9e…246e` |
| `sg_npx_cavern_01_room_01_02.cdt` | `pack_01/001653.cdt` | 233734 | `f2e5f18d…e3b7` |
| `sg_npx_cavern_01_room_02.cdt` | `pack_01/001660.cdt` | 1401166 | `4bff8346…488d` |
| `sg_npx_cavern_01_room_02_02.cdt` | `pack_01/001637.cdt` | 713668 | `b41cb184…2105` |
| `sg_npx_cavern_01_room_03.cdt` | `pack_01/001661.cdt` | 1697090 | `4aee09e9…dd3e` |
| `sg_npx_cavern_01_room_03_02.cdt` | `pack_01/001663.cdt` | 987668 | `e2cf3de7…284f` |
| `sg_npx_cavern_01_door_01.cdt` | `pack_01/001641.cdt` | 52288 | `850b4884…13df` |
| `sg_npx_cavern_01_door_02.cdt` | `pack_01/001645.cdt` | 23008 | `8b1ae6c3…5b3c` |
| `sg_npx_cavern_01_rockbarrier_01.cdt` | `pack_01/001776.cdt` | 59300 | `e140f65a…e5a` |
| `sg_npx_cavern_01_room_01_water.cdt` | `pack_13/001736.cdt` | 364 | `836a6e9c…bcd1` |
| `sg_npx_cavern_sunblock_01.cdt` | `pack_17/001738.cdt` | 984 | `82a3f416…0e72` |

- **Recovered container prefix.** `CDTA | u32 version = 1 | u32 a | u32 b | u32 c |
  u32 vertex_count | f32[3 * vertex_count] | u32 index_count | u16[index_count]`. `(a, b, c)` is
  `(1, 1, 0)` on the rooms and doors and `(8, 1, 0)` on the water collider, so a reader must not
  hard-code those fields. Every parsed index is `< vertex_count`; index counts are not always a
  multiple of three (`door_01` has 464), so the trailing values are kept and not interpreted.
- **The prefix is real, if partial, mesh data.** On `room_01` the 2280 prefix vertices include 2108
  that appear in the community reader's `sg_npx_cavern_01_room_01_lod0.dme` vertex buffer; of the
  1007 distinct prefix triangles (1016 index triangles) 994 are exactly render triangles from that
  same export at 1e-3 precision; 1794 prefix edges, 1254 shared by exactly two triangles, zero
  degenerate and zero duplicated triangles. The remaining payload of `001632.cdt` is 361022 bytes
  (**91%** of the file) and stays undecoded, so the prefix is not claimed as the whole collider.
- **Terrain chunks carry their own collision payload.** All 16 `sg_npx_cavern_01_*.gcnk` chunks that
  the community reader accepts contain a zlib section that inflates to exactly 5632 bytes. The pinned
  toolkit skips it: `Editor/FileTypes/GcnkFile.cs` (`ForgeLightToolkit` `9c06714c…`, MIT, file
  sha256 `c96bf81d…8f3f`) calls `LoadCollision` and its body is `// TODO: Bullet Physics` (line 179-181).
- **No community reader handles the standalone format.** A case-insensitive search for `CDTA`, `.cdt`
  and `collision` across the pinned `ForgeLightToolkit`, `forgelight-toolbox`, `fr-adr-toolkit` trees
  and the `FreeRealms-Legacy`/`Sanctuary` sources finds asset-name listings only (`Models.txt`
  catalogue rows), no reader. A web pass over the published ForgeLight tools found the same gap:
  `pydmod` lists ADR/CNK0-1/DMA/DME/GFX/MRN/zone and `PS2Sanctuary/Mandible` reads `.pack` containers;
  neither reads `*.cdt`. Query strings, tool coverage and the date of the check are recorded in
  `../lab/reports/piece-064-20261009T0438Z/community-reader-search.md`. The CDTA prefix was therefore
  implemented here for a demonstrated gap.

## Collision bounds versus render bounds

For each collision asset with a community-reader render twin, the prefix vertex AABB is compared with
the render AABB from `lab/reports/cave-geometry/community-reader-summary.json`:

| Asset | Relation | Note |
| --- | --- | --- |
| `room_01`, `room_01_02`, `room_03`, `room_03_02`, `room_01_water` | identical | collision AABB equals the render AABB within 1e-4 |
| `room_02` | collision subset | collision Z max `68.313` vs render `78.104` |
| `room_02_02` | collision subset | collision Z min `-18.909` vs render `-23.962` |

Identical bounds are *not* collision proof of that geometry; the relation only says the collision
asset covers the same volume. The two subset rooms show the assets are not copies of the render mesh.

## Native traversal compared with both surfaces

The preserved native cave entry run `lab/runs/client-baseline-20261008T073831Z` (gateway
`server-stdout.log`) records **456** client positions with `zone_id = 411`, spread over
X `276.87…296.96`, Z `279.42…293.99`, Y `27.993…28.000`.

| Check | Result |
| --- | --- |
| Samples with a community-render floor under them | **456 / 456**; max \|native Y - render floor\| **0.046**, median **0.040** |
| Samples with a surface in the parsed CDTA prefix | **0 / 456** |
| Reconstructed route probes with a parsed-prefix collision floor | **2 / 8** (`enemy3` `27.9622`, `boss_inward` `27.9912`, both equal to the render floor within 1e-4) |
| Route probes with render-only evidence | `spawn`, `guide`, `enemy1`, `enemy2`, `boss_current`, `door01` |

So the native client walked the cave on a surface the render mesh explains and the parsed collision
prefix does not. The parsed prefix is therefore incomplete (or the walkable floor comes from the
unparsed CDTA payload, from the GCNK collision section, or from another asset class) - that is the
piece's central negative result and it stays a gap, not a reconstructed fact.

## Inferred meaning and reconstructed policy

- Inferred: `*.cdt` holds per-actor collision authored from (a subset of) the render mesh, because the
  room prefix triangles and vertices are literally render triangles/vertices while 91% of the file
  uses an unrelated record structure (records such as `u32 278`, `u32 7`, `u32 1`, zero-padded
  blocks, and repeated `0xFFFFFFFD`-style values). The tail therefore looks like an acceleration or
  additional-shape structure, but its layout is **not** decoded and is not claimed.
- Reconstructed (analysis policy only, never game physics): `MAX_SLOPE_DEGREES = 50`, a `0.5`-unit
  floor grid, a `0.6`-unit step limit for walkability, and the room mapping
  `local = (worldZ - 370, worldY - 5, 380 - worldX)` reused from the Starting Dungeon Audit. The
  mapping is cross-checked by two prefix surfaces (`enemy3`, `boss_inward`) landing on the audit's
  render floors within 1e-4.
- Not claimed: any original collision material, friction, step height, capsule radius, or the
  original engine's collider choice. No money, items or state are granted anywhere in this piece.

## Artifacts and reproduction

- `prototype/osfr-tools/src/restoration/piece_064.rs` - `CDTA` prefix reader, bounds classification
  and a slope-limited floor query (`#[path]`-included by the test until the lead registers it).
- `prototype/osfr-tools/tests/piece_064.rs` - 8 tests; RED was the missing-module compile error,
  GREEN is 8 passed / 0 failed.
- `scripts/pieces/piece_064.py` and `tests/pieces/test_piece_064.py` - evidence checker and 15 tests
  (real-asset hashes, ADR binding, GCNK payload inflation, render/native replay, synthetic rejection
  paths). The checker also accepts `--native-log` so the lead can replay a new recorded trace.
- `../lab/reports/piece-064-20261009T0438Z/` - `collision-reachability-report.json`,
  `community-reader-search.md`, RED/GREEN logs (`red-cargo-test-piece_064.log`,
  `green-cargo-test-piece_064.log`, `red-python-test-piece_064.log`,
  `green-python-test-piece_064.log`, `run-piece_064.log`, `replay-native-log.log`) and the read-only
  probe `probe-collision.py`.

```powershell
.\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_064
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p 'test_piece_064.py' -v
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_064.py' --out '..\lab\reports\piece-064-20261009T0438Z'
```

## Skills applied

- **reverse-engineering**: recovered the CDTA container prefix from preserved asset bytes, validated
  it against the community DME export (vertices, triangles, bounds) and read the room ADR string
  table to prove the actor-to-collision binding.
- **game-research-websearch** (pinned-corpus form): searched the pinned community readers and wiki
  corpus already on disk for an existing `.cdt`/CDTA reader before implementing one; none exists, and
  no new network fetches were made.
- **ponytail**: one strict prefix parser (Rust + Python), one bounds relation, one floor query; no
  speculative collision solver, no new dependency, no fabricated physics constants.
- **caveman**: concise reporting only; the persisted note stays complete.

## Remaining gaps and gates

- **Unparsed evidence**: the CDTA payload after the prefix (91% of `room_01`) and the 5632-byte GCNK
  collision payloads. Until one is decoded, the walkable floor's collision source is unknown.
- **Native gate**: real client traversal must be shown against decoded collision data, not only the
  render mesh. The checker's `--native-log` replay is the tool for that; this piece ran it on the
  preserved run only.
- **Integration for the lead**: register `piece_064.rs` in `src/restoration`/`lib.rs` when the
  ownership window opens and use `classify_bounds`/`horizontal_floor` for future placement or content
  validation. Do not treat `route_probes` as encounter authoring data.
