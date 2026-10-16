# 027 Housing fixtures and limits

Status: closed for review, 2026-10-09 03:05 UTC. This piece recovers **snap, rotation, build bounds
and instance limits** and reports collision as unrecovered. It is evidence plus a checker plus a small
typed module; it is not a playable claim.

Owned paths: `prototype/osfr-tools/src/restoration/piece_027.rs`,
`prototype/osfr-tools/tests/piece_027.rs`, `scripts/pieces/piece_027.py`,
`tests/pieces/test_piece_027.py`, this note and `lab/reports/piece-027-20261009T025107Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Snap. The client Lua `Housing` object defines exactly two spacings, `GRID_SIZE_ON = 25` and
`GRID_SIZE_OFF = 5`, with `gridSpacing = 25` as the local default
(`lab/decompiled/client-ui/ScriptsBase.lua:33266`, `:33277`). `Housing:ToggleGrid()` swaps the pair,
persists it through `Ui.SetUserOptionsValue("Housing", "gridSpacing", …)`, and `Housing:SetGrid()`
pushes it to the engine with `House.SetGridSpacingInCentimeters(gridSpacing)` (`:33640`, `:33649`).

The engine side is `SetGridSpacingInCentimeters` = `FUN_00c0ba80` (client Lua binding table, entry
`0x182aeac` in the report's `housing-snap-export.txt`). It rejects a non-positive value, takes
`100.0 / centimetres` (`DAT_017ec67c` = 100.0, `housing-constants.txt`) and stores it at housing
manager `+0x4188`. `FUN_00aab720` then snaps only x and z with
`(int)(value * step) * (1.0 / step)` (`_DAT_0175b420` = 1.0, `housing-snap-constant.txt`); the C cast
truncates toward zero, so a negative coordinate rounds up toward zero. Y is never quantised in that
function.

Rotation. The radial menu calls a different function per direction — `House.RotateFixtureClockwise`
(`FUN_00c0c0f0`, action id **7**) and `House.RotateFixtureCounterClockwise` (`FUN_00c0c0a0`, action
id **6**) (`ScriptsBase.lua:33736`; decompile in `housing-rotate-decompile.txt`). The shared transform
function `FUN_00ab1e30` builds the preview rotation from `DAT_017f1914` = 0.2617994 rad = **15
degrees** and negates it for action id 7 (`DAT_017ea5a4` = -0.0). The same function carries
`DAT_017ed818` = 1.5707964 rad = 90 degrees, selected when a per-record flag at `+0x128` is set and the
action is not 6 or 7. Both constants are in `housing-angle-constants.txt`; the flag's meaning is not
recovered, so nothing applies the 90 degree value automatically.

Housing options expose both toggles as user state: `S##snapToGrid` / `S##SnapToGrid_ToolTip`
(`ToggleGrid()`) and `S##AutoRotation` / `S##AutoRotate_ToolTip` (`ToggleRotation()`), the latter
backed by `House.GetUseAutoOrientation()` / `SetUseAutoOrientation()` (`ScriptsBase.lua:56830`,
`:56869`). Verified in the decompiled SWFs `housingEditPanel`, `HousingOptions` under
`lab/reports/piece-027-20261009T025107Z/swf/`.

Limits. `HousingInstance_DS` carries `MaxFixtureCount`, `CurFixtureCount`, `MaxLandmarkCount` and
`CurLandmarkCount` (`ScriptsBase.lua:2594`), and `Housing:SetData()` forwards all four to the client
panel through `SetCounts(curFixtureCount, maxFixtureCount, curLandMarkCount, maxLandMarkCount)`
(`:33624`). Those values arrive from the server in `PlayerHousingInstanceData`
(`lab/references/Sanctuary/src/Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs`), so no
client-side numeric maximum exists and none was invented. `HousingMaxFixtures` and
`FixturePlacementDenied` are present as strings in the original `FreeRealms.exe`, confirmed with
`rg -a -o 'HousingMaxFixtures|FixturePlacementDenied'` against the working copy;
`HousingMaxFixtures` is the only recovered denial string, so the landmark denial text stays unknown.

Build bounds. The instance packet carries a list of min/max boxes (`BuildAreas`, `BoundingBox` with
two `Vector4` values). The community property table
`lab/references/Sanctuary/src/Resources/Houses.json` (sha256 `854b2d3f…d3c`, identical in the
pinned Sanctuary and FreeRealms-Legacy copies) holds 31 rows with `Id`, `NameId`, `ZoneId`, `Icon`,
`SpawnPosition`, `SpawnRotation` and one or more `BuildAreas`. It is **community content**: the
original client `Resources` folder has no house/lot table at all — only `HouseNames.txt`,
`HousingNames.txt`, `HousingPlacementData.txt`, `HousingPlacementAssociations.txt` and
`FactoryPlots.txt`. `HousingPlacementData.txt` contains two `test.adr` rows; `FactoryPlots.txt` is the
farm/ranch foundation grid and is a different system.

## Community-table findings the lead must not smooth over

`scripts/pieces/piece_027.py` writes these into `housing-fixture-limits.json` verbatim:

* ids 11, 12 and 13 are missing; the table spans 1-34 with 31 rows, so complete house coverage is not
  established;
* five rows have an inverted axis and were **not** normalised: 30 (y), 31 (y), 32 (y and z), 33 (z),
  34 (z);
* 19 of 31 spawn points are outside their own build box, including row 1 (spawn x 444.3 against a box
  edge at 443.3) and rows 33/34, which repeat the identical box;
* row 7 is a 1024 x 1024 x 1024 catch-all box spanning zones, unlike the ~120-200 unit lot boxes.

Consequence: the typed Rust contract `BuildBounds::validate` rejects inverted boxes, so rows 30-34
cannot be loaded silently; they need review against a native capture or server data before use.

## Reconstructed boundary

Nothing numeric was invented. The module implements the recovered arithmetic, the two-value toggle, a
signed 15 degree step and a closed-box containment test. No ceiling, radius, tolerance, fixture
maximum or collision shape is fabricated. `snap_position` leaves y untouched because the recovered
snap function only quantised x and z.

## Explicit gaps

* collision and floor/surface semantics: unrecovered. The box test is a proxy, not native collision.
* vertical placement and `MoveFixtureVertically` geometry: unrecovered.
* meaning of the `+0x128` per-record flag that selects the 90 degree constant: unrecovered.
* landmark denial string and any landmark semantics beyond the displayed maximum: unrecovered.
* numeric fixture/landmark maxima: server-supplied per instance, not recovered.
* whether the client preview clamps at all, or relies on server denial: unrecovered.

## Verification

```
& ..\..\scripts\cargo.ps1 test --test piece_027
  RED  before implementation: couldn't find file `tests\..\src\restoration\piece_027.rs`
  GREEN: 12 passed; 0 failed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'tests\pieces\test_piece_027.py' -v
  RED  5 tests: checker script absent
  GREEN: 7 passed (Ran 7 tests, OK)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_027.py' --output <report>\housing-fixture-limits.json
  piece 027 evidence ok: 31 properties, grid 25/5 cm, rotation 15 degrees
```

The client exe, `ScriptsBase.bin`, the decompiled Lua, `Houses.json` and the seven housing SWFs are
hashed in `lab/reports/piece-027-20261009T025107Z/source-provenance.txt`. The full-suite
`cargo test` currently fails for other workers' targets (`tests/kart_racing.rs` missing
`src/kart_racing.rs`, `tests/soccer_review.rs` arity errors); that is unrelated to these paths and
`cargo test --test piece_027` plus `cargo build --lib` are green.

## Next gates

1. Lead integration: register `pub mod piece_027;` under `src/restoration/` and drop the `#[path]`
   include; keep `housing.rs` (006) as the owner of placement state.
2. Wire integration (025): send the recovered grid spacing and receive the instance maxima so a
   restored server and the native client agree on `Housing.gridSpacing` and `SetCounts`.
3. Native acceptance (030): place with snap on and off, rotate both ways, and confirm the visible step
   and the count display in an actual housing instance.
4. Evidence review: re-check the five inverted rows and the 19 spawn mismatches against a live
   capture before any build-area validation is trusted.

Cross-links: [[Housing Restoration]], [[../Design/MMO Worker Contract]], [[../../MMO Work Queue]].
