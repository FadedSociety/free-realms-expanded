# 049 Character appearance coverage

## Scope

Recovered the creator's supported appearance combinations and the invalid-asset exclusions: the
four character types, the nine customization categories, all 253 creator option rows with their
definition resolution, the 49 mapping rows the creator never offers, and the 12 rows that select
definitions from the class-1 `CATEGORY_ID 141` family. This is a recovery and validation
deliverable. It grants nothing, sends nothing and rejects nothing at runtime; wiring the
exclusions into creator/login validation is the lead's integration step.

Two pure lookups carry real behaviour, written test-first:

- `skin_tone_exclusion(id)` returns `Some(SkinTextureAbsent)` for the two skin tones whose head
  textures are absent.
- `head_exclusion(id)` returns `Some(NotOfferedByPublicTables)` for the four China head ids.

## Evidence

| Source | Path | Note |
| --- | --- | --- |
| Supplied creator tables | `lab/working/OSFR.Client/OSFR Client/Resources/CharacterCreateCustomization*.txt`, `CharacterCreateCategories.txt`, `CharacterCreateCharacterTypes.txt`, `GenderDefinitions.txt`, `RaceDefinitions.txt`, `Models.txt` | original client data |
| Supplied mapping/definition tables | `.../Resources/CharacterCreate/CharacterCreateItemMappings.txt`, `CharacterCreateItemClothingSets.txt`, `ItemDefinitions.txt` | original client data |
| Preserved locale export | `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` | resolves every quoted name id |
| Prior appearance evidence | `lab/reports/creator-appearance-availability.json` | absent head textures 7/8, public-table removals |
| Cached public creator tables | `lab/references/asset-cache/public-CharacterCreateCustomizationSkinTone.txt`, `...FaceType.txt` | community launcher tables |
| Recovered coverage document | `lab/reports/piece-049-20261009T042512Z/creator-appearance-coverage.json` | every supported combination, 253 option rows |

`scripts/pieces/piece_049.py` recomputes every fact below from those inputs on each run and
records one sha256 per input in the coverage document.

## Original facts

- Four creator character types (`CharacterCreateCharacterTypes.txt`): 1 Pixie Female, 2 Human
  Female, 3 Human Male, 4 Pixie Male (locale 430560/430558/430557/430559). They bind one-to-one
  to the four `Models.txt` rows with `IS_VALID_FOR_PC = 1`: 1 `human_m.adr`, 2 `fairy_m.adr`,
  60 `human_f.adr`, 61 `fairy_f.adr`.
- Race naming splits inside the original data: `RACE_ID 1` is `Human` and `RACE_ID 2` is `Pixie`
  (locale 2230/2231) while every race 2 model file is named `fairy_*`. Both facts are recovered;
  nothing renames either.
- Nine categories (`CharacterCreateCategories.txt`) each reference exactly one
  `CharacterCreateItemMappings.txt` TYPE: FaceType 8, SkinTone 0, HairStyle 1, HairColor 2,
  EyeColor 3, WingsPixie 6, FaceHair 4, Clothes 7, FacePaint 5. `FaceHair` is scoped Human/Male,
  `WingsPixie` Pixie/All, everything else All/All.
- 253 creator option rows exist: 67 Pixie Female, 61 Human Female, 63 Human Male, 62 Pixie Male.
  Per category the supplied tables offer FaceType 4, SkinTone 8, HairStyle 8, HairColor 10,
  EyeColor 10, WingsPixie 6 (Pixie only), FaceHair 7 (Human male only), Clothes 9 and FacePaint
  12 (female) / 7 (male) rows per character type.
- Resolution chain: creator row -> mapping row -> type-12 appearance definition, whose `PARAM1`
  is the appearance slot and `PARAM2` the appearance index. Clothing rows instead resolve through
  `CharacterCreateItemClothingSets.txt` to three item definitions (feet/legs/chest slots 5/4/3).
- `CharacterCreateItemMappings.txt` holds 272 rows (68 per race/gender); 223 are reachable from
  the creator tables and 49 are not: type 0 sixteen, type 1 thirty, type 3 one, type 4 one,
  type 5 one. The 30 unreachable type-1 rows resolve to the `FRC` hair definitions 78037-78066,
  which no creator row offers.
- `ItemDefinitions.txt` holds 42 rows with `CATEGORY_ID 141`: the six standard skin tones
  77484-77489 (class 0) and 36 class-1 rows - 30 `FRC` hair definitions 78037-78066, the four
  China heads 78086-78089 and the two extra skin tones 78090/78091.
- Exactly twelve creator rows resolve into that class-1 family, two per character type in the
  SkinTone table and one in FaceType:
  - SkinTone: mapping 272 -> definition 78090 (`SkinTone 7`, index 7) and mapping 1 -> definition
    78091 (`SkinTone 8`, index 8).
  - FaceType: mappings 268-271 -> definitions 78086-78089 (`fairy_f`/`fairy_m`/`human_m`/
    `human_f_head_china.adr`, indexes 5-8).
- After the twelve exclusions the creator offers 241 rows: 64 Pixie Female, 58 Human Female,
  60 Human Male, 59 Pixie Male.

## Community corroboration

- The cached public launcher tables remove exactly those twelve rows and add or change none:
  SkinTone loses the eight extra rows, FaceType loses the four China rows. The prior
  `creator-appearance-availability.json` records the same removals independently.
- Human male Caucasian head textures 1-6 match the original manifest CRC; textures 7/8 are
  absent from the supplied client and the 18 scanned public packs, and native run
  `client-baseline-20261008T065152Z` failed on `human_m_head_caucasian_08.dds` (404). Skin tone 8
  is what the extra "Pearl" row selects, so the missing texture and the removed row are the same
  defect seen from two sides.

## Original versus reconstruction

- Original: every row, id, index, model name and scope above, plus the mapping type per category
  and the row counts. All are re-derived from the supplied tables on each checker run.
- Community evidence: the public-table removals. The checker treats them as corroboration and
  fails if they stop matching the supplied data, but it does not call them original.
- Reconstructed policy: the two exclusion reasons. `skin_texture_absent` is supported by absent
  assets and a native 404; `not_offered_by_public_tables` is a community-table fact only. No
  behaviour is changed here.

## Gaps

- The four China head rows carry community-table evidence only; no missing
  `*_head_china.adr` model or texture was proven.
- `HeadMappings.txt` id 4 (`<race>_<gender>_head_china.adr`) exists but no creator row in either
  table references it; its support status stays unresolved.
- EyeColor guid 1000013 (Human Female) resolves to definition 77605 whose `GENDER_USAGE` is 1
  (male). The definition carries no model name, so this is recorded as a data conflict, not as an
  exclusion. The two China head definitions 78086/78089 carry the same contradictory tag.
- Skin tones 7/8 are proven absent only for the Human male Caucasian textures that were scanned;
  the other three character types inherit the public removal without an individual proof.
- The 30 unreachable `FRC` hair definitions are offered by no table; no supported-option claim is
  made for them in either direction.
- No native client run was made for this item.

## Commands

```
& ..\lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_049.py ^
    --output ..\lab\reports\piece-049-20261009T042512Z\creator-appearance-coverage.json
# piece 049 coverage ok: 4 character types, 9 categories, 241 supported rows,
# 12 excluded rows, 223 reachable mapping rows, 6 gaps

& ..\lab\tools\wiki-env\Scripts\python.exe tests\pieces\test_piece_049.py -v
# Ran 8 tests ... OK

& scripts\cargo.ps1 test --test piece_049        # from prototype/osfr-tools
# 6 passed; 0 failed
```

A module-drift copy of `piece_049.rs` with one changed row count makes the checker exit 2 with
`RustModuleDrift`, and a mutated supplied or public SkinTone table exits 2 with
`ExcludedRowSetDrift` / `PublicTableDrift`.

## Integration gate

`prototype/osfr-tools/src/restoration/piece_049.rs` is not registered in `src/lib.rs` (lead-owned).
The module is recovery metadata plus two pure lookups; the lead decides whether the creator/login
path rejects skin tones 7/8 and head ids 5-8, and no native acceptance is claimed here.
