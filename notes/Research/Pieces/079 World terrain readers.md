# 079 World terrain readers

## Scope

Ran the pinned community terrain-chunk readers over every preserved world chunk, measured what they
decode, and named the remaining format gaps. No custom reader ships from this item, nothing in the
client, the gateway, a save or a release changed, and no gameplay or native behaviour is claimed.

## Original facts

* The extraction holds **2,344 `*.gcnk` entries** across 22 packs. The checker re-reads every one
  against `../lab/manifests/rust-asset-catalog.tsv` (length, CRC32, SHA-256). 2,336 carry the `GCNK`
  magic with a little-endian version word: **v2 = 2, v3 = 51, v4 = 35, v5 = 1,122, v6 = 1,126**.
* The other **8 entries are five-byte ASCII stubs containing `hello`**: `sh_snowball_battle_-4_16`,
  `-4_20`, `-4_24`, `-4_28`, `-8_16`, `-8_20`, `-8_24`, `-8_28`, from `Assets_000/003/006/007/
  008/009.pack`, all with CRC32 `3610a686`. They are shipped placeholders, not truncated chunks.
* The applied reader is `ForgeLightToolkit.GcnkFile` revision
  `9c06714c5d79d9152c322814bc904abad944e06e` (MIT, `lab/references/ForgeLightToolkit`,
  `SOURCE.json` pins every file hash). It declares its window as `Version is < 0 or > 6`, so no
  preserved chunk is outside it.
* **Executed over all 2,344 chunks, the pinned reader decodes 2,277 and aborts on 67.** The eight
  stub failures are a magic mismatch. The other **59 aborts are `NotImplementedException` thrown at
  `Editor/FileTypes/Gcnk/RuntimeObject.cs:70`**, the version <= 4 terrain-object path: **32 x v3**
  (`sg_dm_arena2_*`, `test_designtrack_bbe_*` and neighbours) and **27 x v4** (22 of them in
  `AssetsW_010.pack`, plus the identical 132,221-byte chunk `005029`/`003339`/`003099`).
* A private lab variant reads the terrain-object field as the null-terminated string that two
  independent community sources document (`soir20/oxide` `gcnk.rs`: `Name` for `version < 5`;
  `soir20/oxide-tools` `gcnk_chunk.hexpat`: `else char terrain_object_name[];`). It records how many
  decompressed chunk bytes each parse consumed. Result: **47 of the 59 recover** (all 32 v3 and 15
  of the 27 v4). Every recovered chunk contains at least one runtime object, and **all 32 recovered
  v3 chunks consume exactly their declared chunk length** - the structural evidence for the field,
  since the names themselves read back empty in this corpus.
* The same accounting exposes what "ok" hides: **15 chunks parse without consuming the declared
  chunk length** - 13 of the recovered v4 chunks, and two chunks the pinned reader already reports
  `ok` (`pack_10/000820.gcnk` 2,324 of 135,952 bytes; `pack_21/003752.gcnk` 1,128 of 132,146).
  Trailing unparsed data remains unexplained; no PS3/extra-section reader was written.
* Second community tool, run not assumed: `psemu/forgelight-chunk` (`cnk.js`) at
  `9b3a389f3738e4d6ad99bde12aa67538e1f5535d` (MIT). It rejects a v6 FR chunk with
  `Unsupported CNK version: 6` and dies on the five-byte stub, because it implements the
  PlanetSide 2-era `CNK0`/v<=2 schema. It is **not applicable** to Free Realms GCNK.
* Still-unread preserved world formats, header-triaged here: **174 `.gzne`** (all `GZNE`, versions
  2 = 146 and 3 = 28) and **2,743 `.cdt`** (all `CDTA`). No community reader for either was found
  or run; the pinned toolkit has no GZNE reader.

## Inference

The v3 evidence (32/32 byte-exact consumption with an empty name) supports a variable-length
terrain-object string on the version <= 4 path. The v4 anomalies - 13 short parses, 12 overruns -
point at a different v4 layout or content family; the exact difference is **not recovered**.

## Gaps

| Gap | Evidence | Status |
| --- | --- | --- |
| Collision section not decoded | `GcnkFile.cs` `LoadCollision` is `// TODO: Bullet Physics` | open; `soir20/oxide` decodes it as bounding-volume hierarchies, source-only, AGPL-3.0, not executed here |
| Detail mask / height map not decoded | `GcnkFile.cs` `// TODO: Height Map` then `reader.Skip(...)` | open; layout documented by `oxide-tools` `gcnk_chunk.hexpat` (v>=4 uses 4-bit channels) |
| Per-tile `Unknown7..Unknown13` | `Gcnk/Tile.cs` reads them unlabelled; hexpat says `unknown1` is never > 0 in FR/CWA | open |
| PS3 eco-data path | `Gcnk/Tile.cs` `// TODO: FreeRealms PS3` | open |
| v3/v4 runtime objects | `RuntimeObject.cs:70` `throw new NotImplementedException();` | root-caused; v3 recovered by the documented string path, v4 half open |
| v4 layout | 12 chunks still overrun; 13 recovered chunks stop early | open; no custom v4 reader written |
| `ok` is not proof of a complete parse | two accepted chunks stop far short of the declared chunk length | reader defect, recorded |
| `LoadCollision` argument | `Load()` passes `chunkDecompressedStream`, not the collision stream | defect today invisible because the callee ignores its argument |
| GZNE (174) / CDT (2,743) containers | header triage only | unread; no community reader found |

## Not claimed

No native or gameplay acceptance, no collision or path-and-reachability proof, no rendering result,
and no historical statement about how the original client treated the stubs or the v4 family. The
variant lives only under `lab/reports` as evidence: shipping code copied from an AGPL-3.0 project
needs a licence decision first.

## Evidence

Private run directory `../lab/reports/piece-079-20261009T0455Z`:

| Artifact | sha256 |
| --- | --- |
| `piece-079.json` (checker output) | `a429cdd77b7ad65747a92da14c53a4ba631b0f093e4f72925479e9d862ffb1c8` |
| `survey/reader-survey.json` | `451a481954a4724658ceb303947b23e0d5e6b43127ac8daf9bf4eeca6cecfe5f` |
| `survey-variant/reader-survey-variant.json` | `c840ebdcbfc4ffe21384b34ac95cc2a426887d9c381685e7ca0c92f10fc1b524` |
| `cnk-probe.json`, `cnk-probe-output.txt` | `5b33c802...`, `f428a4b8...` |
| `survey/inputs.json` (2,344 chunks) | `73ca74f70b2049b5df7cca78551b62233c9e64014454ba7e2a3852f1ba6ed207` |

Committed-side code: `prototype/osfr-tools/src/restoration/piece_079.rs` (inventory, classifier and
declared bounds, awaiting lead registration in `src/lib.rs`), its `#[path]` test
`prototype/osfr-tools/tests/piece_079.rs`, the checker `scripts/pieces/piece_079.py` and its test
`tests/pieces/test_piece_079.py`.

## Reproduce

```powershell
& .\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_079
& ..\lab\tools\wiki-env\Scripts\python.exe tests\pieces\test_piece_079.py -v
```

The lab `README.md` in the run directory lists the exact survey, variant and checker commands. The
checker fails closed (exit 2) if the catalogue, the extraction, the reader source, either survey,
the probe or the Rust module drifts from the recorded evidence.
