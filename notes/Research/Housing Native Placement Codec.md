# Housing native placement codec

Lab-only candidate codec for the whole inner opcode-127 housing placement flow, 2026-10-09 (UTC).
The production write hold is active: **no `prototype/osfr-tools/src/` or `tests/` file was created
or modified**. Reviewer owns this note and
`lab/reports/housing-native-codec-20261009T2100Z/` only. Skills: RTK (normal commands), Ponytail
(one small std-only codec, no new dependencies), Caveman (terse report), reverse-engineering (the
reader anchors are decompile claims).

## Candidate files (to be copied by the lead after the hold)

| Candidate | Copy target later | SHA-256 |
| --- | --- | --- |
| `lab/reports/housing-native-codec-20261009T2100Z/candidate/housing_native.rs` | `prototype/osfr-tools/src/housing_native.rs` | see `final-hashes.txt` |
| `lab/reports/housing-native-codec-20261009T2100Z/candidate/tests/codec.rs` | `prototype/osfr-tools/tests/housing_native.rs` | see `final-hashes.txt` |

The test uses `#[path = "../housing_native.rs"] mod housing_native;` while the module is
unregistered; the lead owns `lib.rs` registration.

## Scope and units

Entry points start at the **subtype `u16`** and exclude the base opcode 127 (the analyzer/tunnel
unit): `PlaceFixture` (2) = 42 bytes; `FixtureUpdate` (40) = 229 for the captured empty paths
(2 + 133 fixture record + 94 tail); `FixtureRemove` (41) = 10; `HouseInfo` (44) = 22;
`PlacementDenied` (53) = 2 + 4 + message bytes. `HousingPacket::decode/encode` dispatch on the
subtype; unknown subtypes and empty payloads are refused.

## Layouts and provenance

* **127/2 `PlaceFixture`** - `u32 first_dword` (the placement object's `+0x40`; native anchors
  pseudocode 1407923-1407936, 1410183-1410190, 1410225-1410229), `f32[4] position`,
  `f32[4] rotation`, `f32 scale`. The first dword is typed as an uninterpreted `u32`; its identity
  is not established (all eight captured values resolve in `ClientItemDefinitions`, but the
  resolved entries are unrelated gear).
* **127/40 `FixtureUpdate`** - the 133-byte community `FixtureInstance` record, the 28-byte
  `FixtureInstanceInfo` (+ `CouplingDisplayData`), the variable community `FixtureDefinition`, then
  the native reader's positional `u64` list (`unknown_guids`) and three `u32` words. The
  `CharacterGuid` name for the list is plausible from nearby `SoeUtil` type text but **not**
  anchor-proven, so it stays positional. The action dictionary is key-then-value with
  `map_key` and `id` kept separate (`key == id` is not assumed; all captured action lists are
  empty).
* **127/41 `FixtureRemove`** - one `u64` fixture guid.
* **127/44 `HouseInfo`** - fixed 20-byte body (InEditMode, Unused, IsLocked, IsFloraAllowed,
  PetAutospawn, CurFixtureCount, CurLandmarkCount, FurnitureScore); only values vary across the 19
  captured records.
* **127/53 `PlacementDenied`** - one `i32`-length UTF-8 string, bounded by the native reader
  `FUN_00ab3030`; **trailing bytes are refused** because the handler never reads them and no
  captured 53 exists.

## Policies (fail closed, documented)

Finite floats only on encode and decode; strings `i32` length + UTF-8, `0..=4096`, invalid UTF-8
refused; array counts `0..=4096` and must fit; booleans decode only from `0`/`1` (the native
client treats any nonzero as true; this codec is stricter); every decoder must consume its payload
exactly; unknown positional fields keep `unknownN` names and no meaning is invented.

## Evidence

* RED first: `red.log` - `error: couldn't find file .../candidate/housing_native.rs`.
* `green.log` - **11 passed; 0 failed**, including a populated update whose second action uses
  `map_key 11` / `id 12` to prove the key and value id are separate fields.
* `capture-roundtrip.log` - **44 pass, 0 fail** over the private capture vectors extracted from
  `p2.pcap` (`65AC0801…`): 9× 127/40 (229 B), 8× 127/2 (42 B), 8× 127/41 (10 B), 19× 127/44
  (22 B), each decoded and re-encoded byte-identical (`capture-vectors/manifest.json`).
* `oracle-compare-three.log` - the candidate tail bytes equal, byte for byte:
  (a) the unchanged community C# serializers on a 152-byte synthetic fixture
  (`oracle-synthetic.log`), (b) the independent reviewer's compiled `non_empty_all` fixture
  (162 B), and (c) the independent `actions_only` fixture (127 B). This is what fixed the earlier
  152-byte mismatch: the community writer (and native `FUN_00aba9e0`) emit each action as
  dictionary key then value record.
* Correction receipt for the earlier tail note (reader-chain typo `1402340` -> `1392340`, the
  CharacterGuid qualification and the action key/value correction):
  `lab/reports/housing-fixture-tail-compatibility-20261009T2000Z/correction-receipt-20261009T2140Z.json`.
  The original tail report/note are untouched.

## Read-cursor limits and boundaries

The captured paths exercise empty coupling/strings/actions; the non-empty paths are pinned by the
unchanged community writer and the independent compiled fixtures, not by a capture. `127/53` has
no captured vector and any trailing field is unconfirmed, so the codec refuses it. Nothing here is
native acceptance or a gameplay claim (no physics, no authority, no inventory). The capture
transport decode inherits the pinned piece-022 model (`crc_seed_recovered=false`).

## Commands

```
rustc --edition 2024 --test candidate/tests/codec.rs -o codec.exe && codec.exe
rustc --edition 2024 -A dead_code probe/capture-roundtrip.rs -o capture-roundtrip.exe
capture-roundtrip.exe capture-vectors
lab/tools/dotnet/dotnet.exe run --project oracle-synthetic -- oracle-synthetic.bin
rustc --edition 2024 -A dead_code probe/oracle-compare-three.rs -o oracle-compare-three.exe
oracle-compare-three.exe oracle-synthetic.bin independent-nonempty-all.bin independent-actions-only.bin
```


## Lead production integration (2026-10-09T17:31 UTC)

Independent review accepted the frozen candidate after16 executed checks,44 independently re-extracted capture roundtrips, every-cut truncation checks and three nonempty compiled community-writer vectors. See [[Reviews/Housing Native Placement Codec Review]]. The passing candidate log is `green.log`; `green-compile.log` is an earlier failed intermediate receipt and is preserved as history.

The lead copied the codec into `prototype/osfr-tools/src/housing_native.rs` and registered it in `lib.rs`. Both misleading whole-inner comments now say subtype-level body. The `rotation` label is explicitly conventional: captured second float4 values are all zero, so layout is established but semantic naming is not. Typed encode/decode methods still exclude family127. Only `HousingPacket::encode_inner/decode_inner` include one127 prefix; the gateway transport wrapper remains outside the codec.

Three added synthetic tests check exact remove framing, all5 subtype boundaries, wrong family, double prefix, every truncation and trailing-byte refusal. They failed before the new API and pass after it. The registered module now passes14 ordinary tests and one explicit private gate validating all44 manifest hashes, subtype counts and exact body/inner roundtrips. Evidence is `../lab/reports/housing-codec-lead-integration-20261009T173110Z/receipt.json`. Raw vectors remain in the lab. Root combined gate/release is running; this is protocol acceptance only. There is no house entry, fixture placement authority, inventory consumption, house persistence or native building acceptance yet.


## Completed combined source gate

The fresh root174628 gate passes1536 ordinary tests and12 explicit private checks including the44-vector Housing body/inner oracle. Production housing9091118f is unchanged. Frozen21233dc7 release remains undeployed/native-untested; protocol acceptance does not establish playable housing.
