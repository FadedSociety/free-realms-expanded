# TCG Card Data

**Research pass:** 2026-10-13. Scope: extract and inventory the separate Qt4 `tcg/FreeRealmsTCG.dll` client data and art. This is not the in-exe Treasure War card engine. The client guide describes them as separate games and leaves the DLL's own network link unresolved.

## Results

The selected immutable client copy is `lab/runs/client-baseline-20261008T015138Z/client/tcg/`. `lab/content/tcg/inventory.json` records every file in that tree with type, byte length, SHA-256 and CRC-32, plus pack-entry, RCC-resource, extracted-art and discovered collection-copy inventories. Collection filenames other than `default.cln` are replaced by stable redacted aliases.

| Inventory layer | Count and validation |
| --- | --- |
| Client `tcg/` tree | 63 files: 19 DAT, 10 DLL, 9 QM, 8 DIR, 6 RCC, 4 CLN, and 7 other files; each has size and hashes in `inventory.json`. |
| `AssetsTcgW_000.pack` | 113,356,959 bytes; SHA-256 `60cd84e43ee76654f95d248b9919c7f76a4aa01740ea9fea6fc801f526ed3b4f`; 57 entries across 9 file types. All extracted entries match catalog sizes and CRC-32 values. |
| `cards.rcc` | 51,406,873 bytes; SHA-256 `4383c893334073960629516e0024ba1546e6a7fd2d154cdcaa31c59fd3706de0`. The Qt RCC v1 tree contains 418 nodes and 414 resources: 403 PNG resources and 11 fonts. |
| Art exports | 403 canonical PNG exports are under `lab/content/tcg/art/canonical/`. Inventory also records 665 PNGs from an earlier extraction under `art/images/`; those are not used by `cards.json`. |
| Collections | Four client `.cln` files are inventoried; the three non-default names are redacted. The separately extracted `lab/extracted/client-assets/pack_23/000037.cln` is also inventoried. |

The baseline `tcg/data/collections/default.cln`, embedded `AssetsTcgW_000.pack` entry 37, and `lab/extracted/client-assets/pack_23/000037.cln` are byte-identical: 356,892 bytes, SHA-256 `7cc6b4c8e70a1a31863f2921379d1a79aca8bfe58836ed5434daa9d67bb61595`, CRC-32 `89a39fc0`.

`cards.json` currently records **201 art-linked archetype candidates**, not 201 confirmed playable cards. There are 202 numeric IDs under `images/card/<id>.png`; 201 match `data/archetypes/index.dat`, while `100011272` does not. One decompressed archetype sample (`2900001`) contains product-label strings for booster products, demonstrating that an art/index match alone does not classify an entry as a playable card. Each record links its ID, generated text-key names, selected art path and art hash to the validated compressed-store row and decoded-record hash. Fields whose values are not reliably decoded remain `null`.

## Formats and evidence

### Asset pack and Qt resources

The existing Rust `osfr-tools` reader cataloged and extracted the pack into `lab/content/tcg/pack-catalog.tsv` and `lab/content/tcg/raw/pack_00/`. The repository README documents its `catalog` and `extract` commands. The native pack notes describe manifest blocks, big-endian record fields and CRC-32 checks ([S0603 findings](../../../lab/native-map/run/shards/S0603/findings.md), especially lines 94–107).

`cards.rcc` begins with the Qt `qres` signature and version 1. The bounded Python reader follows the Qt 4 RCC tree and resource layout, decompresses flagged payloads, and validates exported PNG chunk CRCs. Several PNG payloads require byte-wise XOR with `0x73` before they pass PNG validation. The implementation was checked against the [Qt 4.8 resource reader](https://github.com/qt/qt/blob/4.8/src/corelib/io/qresource.cpp) and [RCC writer](https://github.com/qt/qt/blob/4.8/src/tools/rcc/rcc.cpp). Resource paths, locale fields, offsets, payload lengths and both stored/exported SHA-256 values are in `rcc-resources.json`.

### Archetypes and card text keys

The store reader decodes `data/archetypes/index.dat` into 1,391 four-integer rows. It checks each row's compressed range in `storage.dat`, zlib-decompresses it, and verifies the declared decoded length; all 1,391 records pass. It does **not** yet deserialize the inner class-specific archetype object. Raw string spans can therefore be inspected, but are not assigned to title, set, rarity, type, cost, statistics or rules fields without a field-level parser.

Native-map provenance in `lab/native-map/run/map.sqlite` (read through `python -I scripts/native_map/nmq.py`):

- `ArchetypeDB_LoadIndex` at `012b8530` reads index integers.
- `ArchetypeDB_GetArchetype` at `012b8ff0` uses row fields `+4` compressed length, `+8` decoded length and `+0x0c` storage offset.
- `ArchetypeDB_DecodeArchetype` at `012b87e0` uncompresses the record and passes it through the object factory/reader.
- `SerialAppendInteger` at `013c23f0` and the corresponding read path at `013c24d0` establish the compact integer encoding used by these records and the collection stream.
- `Archetype::GetTcgTextProperty` at `0137cd70` constructs `Tcg_title_%d`, `Tcg_subtitle_%d`, `Tcg_game_text_%d`, `Tcg_lore_%d` and `Tcg_traits_%d` keys from the archetype ID. For IDs above 100,000,000 it removes the encoded 100,000 digit group when nonzero. These are recovered key patterns; they do not provide the translations.
- `CardAttributeNameForId` at `0131cee0` and card serialization at `0131e290` expose fields such as title, card type, game text, traits, color, value, illustrator, flavor text and collectors number. These names describe the client object layout; this pass does not prove corresponding values for each store record.

`cards.json` preserves those generated key strings and art references, while `name`, `set`, `rarity`, `type`, `cost`, `stats` and rules-text values remain unresolved. The unmatched art ID is listed separately.

Nine Qt `.qm` files were parsed into `localization.json`; none contained a `Tcg_*` source key. The TCG's `locale/en_us_data.dat` is reported in the [Trading Card Game Build Guide](../Build%20Guides/Gameplay/Trading%20Card%20Game.md) as 5,092 rows (`ucdt` 5,013; `ugdt` 79), but its string-ID/hash mapping to the archetype keys is unresolved. The Qt translation parser follows the Qt 4 QM source-section format described in the [Qt 4.8 translator implementation](https://github.com/qt/qt/blob/4.8/src/corelib/kernel/qtranslator.cpp).

Related vault note [[Research/Pieces/010 Trading card game]] describes the in-exe Scaleform/native game and reports its power/cost/colour/rarity values arrive through card-data messages rather than a static card-number table. That finding is specific to that game path; it does not establish how the separate Qt DLL stores or receives its card data.

### Collection file

`collections.json` decodes the extracted `pack_23/000037.cln` sample. Observed header: marker `#`, version 3.3, a 7-byte label, four zero bytes whose meaning is unknown, then compact-integer payload at offset 15. The label bytes are not copied into the artifact; only their length and SHA-256 are recorded. The payload decodes fully as 195,063 integers with no trailing bytes. Every one of the 201 known art-linked candidate IDs occurs at least once.

The decoder counts integer-token occurrences only. The record boundaries and interpretation of those occurrences are unknown, so they are not per-card owned quantities. Three other client collection files are hashed but not decoded. No player names or chat are included in the artifacts or this note.

A Java static-string/xref scan of the shipped `FreeRealmsTCG.dll` finds `.cln` and `data/collections` references in a constructor at `10070b50`, and `data/archetypes` references in constructors. This locates the collection/archetype readers but does not reveal their inner schemas. Full Ghidra auto-analysis timed out at 120 and 300 seconds; the no-analysis script output is in `lab/content/tcg/decomp/tcg-data-functions.txt` with run logs beside it.

## Open gaps

- The inner archetype object fields are not decoded. Names, sets, rarity, card type, cost, statistics, and rules text values are therefore not yet established from these blobs.
- The TCG locale's key-to-string mapping is unknown; the main client `.qm` files do not supply the generated keys.
- The 201 art-linked archetype candidates need semantic classification; one numeric art ID is unmatched.
- `.cln` collection boundaries, ownership quantities, and the four-byte header prefix remain unknown. Other collection payloads were not decoded.
- Deck lists and the store/booster contents are not reconstructed from this extraction.
- The rules engine and the DLL's own network protocol/server are outside this extraction pass and remain unknown. Do not infer that the mapped Treasure War/game-connection protocol is the DLL protocol.

## What the card service needs next

Before a Rust TCG service can serve gameplay data, the DLL's own connection and login flow must be identified from `FreeRealmsTCG.dll`; the native map currently only establishes its launch boundary. The service also needs a field-level archetype decoder, the locale mapping for generated text keys, validated playable-card classifications, collection record semantics, deck/store data, and protocol evidence for data delivery and match actions. The broader [Trading Card Game Build Guide](../Build%20Guides/Gameplay/Trading%20Card%20Game.md) already separates the DLL service from Treasure War and records the current protocol boundary and open questions.

## Reproduction artifacts

- `lab/content/tcg/inventory.json`: source file, pack entry, RCC resource and art hashes/sizes, including the extracted client `.cln` copy.
- `lab/content/tcg/cards.json`: matched archetype/art candidates and unresolved fields.
- `lab/content/tcg/collections.json`: collection header and compact integer counts; no raw label text.
- `lab/content/tcg/rcc-resources.json`, `localization.json`, and `pack-catalog.tsv`: decoded resource/locale manifests and pack catalog.
- `lab/content/tcg/tools/`: bounded readers for Qt RCC, Qt QM, archetype index/storage, collection payload and inventory. Run Python readers with `python -I` and pass source/output paths as arguments.

Commands used (PowerShell from the project root; extraction outputs are under the owned lab path):

```powershell
& '.\prototype\osfr-tools\target\release\osfr-tools.exe' catalog '..\lab\runs\client-baseline-20261008T015138Z\client\tcg' '..\lab\content\tcg\pack-catalog.tsv'
& '.\prototype\osfr-tools\target\release\osfr-tools.exe' extract '..\lab\runs\client-baseline-20261008T015138Z\client\tcg' '..\lab\content\tcg\raw'
python -I '..\lab\content\tcg\tools\rcc_extract.py' '..\lab\content\tcg\raw\pack_00\000056.rcc' --manifest '..\lab\content\tcg\rcc-resources.json' --image-dir '..\lab\content\tcg\art\canonical'
$qmPaths = Get-ChildItem '..\lab\content\tcg\raw\pack_00' -Filter '*.qm' | ForEach-Object FullName
python -I '..\lab\content\tcg\tools\qt_qm_extract.py' $qmPaths --output '..\lab\content\tcg\localization.json'
python -I '..\lab\content\tcg\tools\decode_tcg_store.py' '..\lab\runs\client-baseline-20261008T015138Z\client\tcg\data\archetypes\index.dat' '..\lab\runs\client-baseline-20261008T015138Z\client\tcg\data\archetypes\storage.dat' '..\lab\content\tcg\rcc-resources.json' --art-root '..\lab\content\tcg\art' --output '..\lab\content\tcg\cards.json'
python -I '..\lab\content\tcg\tools\decode_tcg_collection.py' '..\lab\extracted\client-assets\pack_23\000037.cln' '..\lab\content\tcg\cards.json' --output '..\lab\content\tcg\collections.json'
python -I '..\lab\content\tcg\tools\inventory_tcg.py' '..\lab\runs\client-baseline-20261008T015138Z\client\tcg' '..\lab\content\tcg\pack-catalog.tsv' '..\lab\content\tcg\raw' '..\lab\content\tcg\rcc-resources.json' '..\lab\content\tcg\art' --extracted-client-assets '..\lab\extracted\client-assets' --output '..\lab\content\tcg\inventory.json'
```

The Python archetype, collection and inventory commands above were run in this pass: 1,391 zlib rows validated, 201/201 candidate IDs found in the sample collection, and 63 client files / 57 pack entries / 414 RCC resources inventoried. `osfr-tools` output and the RCC/QM extraction manifests were already present from the earlier completed extraction stage described above.
