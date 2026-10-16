# 014 Forging recipes/content

Status: bounded recovery plus checker returned. Item 014 acceptance is "recover
anvil patterns/equipment recipes; correct profile and slot restrictions". This
piece recovers the pattern-item catalogue, the equipment records the opening
forging path uses, the class-owner and slot tables, and the profile class
allowlists, and it ships a validator that enforces those restrictions. Native
equip/forge acceptance remains a separate gate.

## Recovered original facts

Pattern items are the recoverable `Class 58` records of the pinned Legacy
`ClientItemDefinitions.json` (revision `9e082025391177d5a33ee18c714ddd06b457a695`,
AGPL-3.0, inspected as facts; no upstream source copied into this repository).
There are 28 such records; 27 carry `CategoryId 17` and `Type 5`, single stack,
`NoSale` and a `Recipe` name. `Param1` is each record's own recipe index.

- `2506` "Spunky Scrapper Hammer Recipe" (`Param1 11`, `NameId 1565`,
  `DescriptionId 1587`). Recovered locale text for 1587 reads "This recipe
  teaches you how to create a Spunky Scrapper Hammer.", and locale 102173 is
  "Spunky Scrapper Hammer". The pattern to product link is therefore recovered
  text, not a name guess.
- The adjacent Blacksmith patterns `2505`, `2507`-`2510`, `2512`-`2517`,
  `2519`-`2526` match the recipe names listed on the community Blacksmith page
  (revision 46062). `869` is the Chef mousse pattern; `838` is the one class58
  record outside category 17.

Equipment records for this slice (same source):

- `30280`-`30289` "Spunky Scrapper Hammer": `Class 71`, `Slot 7`,
  `CategoryId 207`, model `hammer_ar_ag_weapon_logjammer.adr`, one genuine
  definition with ten tint variants (`255,237,243,234,229,235,231,259,248,242`).
- `75030` "Brawler's Mallet of Sweeps": the substitute the current opening
  campaign still compares against at slot 7.
- `6459` "Student Blacksmith Hammer": `Class 34`, `Slot 7`, `CategoryId 107`,
  model `tool_ar_ag_weapon_hammer.adr`.

Class ownership comes from the pinned `ItemClasses.txt`
`ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID`:

| class | locale name | owner profile name id | owner profile | wield type |
|---|---|---|---|---|
| 25 | Brawler Gear | 2873 | 43 Brawler | 0 |
| 34 | Blacksmith Tools | 2614 | 16 Blacksmith | 0 |
| 58 | Blacksmith Shards | 2614 | 16 Blacksmith | 0 |
| 71 | Brawler 1-Handed Hammers | 2873 | 43 Brawler | 13 |

Slot 7 is `Weapon` with `IS_WEAPON 1` in the supplied client's
`Resources/EquipmentSlotDefinitions.txt`. Class allowlists come from the pinned
`Profiles.json`: profile 16 admits `14,34,48,49,58,60,67,69,...` and profile 43
admits `5,25,42,50,52,69,71,...`; 16 does **not** admit 71 and 43 does **not**
admit 58. The recovered character's own profile spools carry slots
`[1,2,3,4,5,7,13]` for Blacksmith and `[1,2,3,4,5,7,8,12,6]` for Brawler.

Recovered restriction outcome: the Blacksmith holds and crafts the class58
pattern and carries its class34 tool at slot 7, while the class71 hammer is a
Brawler weapon at slot 7. A Blacksmith equipping the forged hammer is invalid;
a Brawler equipping the Blacksmith tool is invalid.

## Source disagreement, recorded not resolved

The supplied server's own customize definitions
(`lab/working/OSFR.Server/OSFR Server/Customize/ClientItemDefinitions.json`,
listed in `lab/manifests/OSFR.Server-intake.json`) give `75030` class 71, while
the pinned Legacy table gives class 25 ("Brawler Gear"); slot 7 and category 207
agree. Profile 43 admits both 25 and 71, so equip legality is unaffected. The
shipped packet value is the one the restored client loads; the Legacy class
value is reported as a warning rather than silently preferred.

## Reconstruction boundary

- Recovered: pattern records, product description text, equipment definitions,
  class owners, slot definition, profile class allowlists, profile slot spools.
- Reconstructed policy: `check_equip` (class allowlist plus slot spool). The
  original server-side equip enforcement code was not recovered.
- Reconstructed, unchanged here: per-attempt consumption
  `10 Copper Bars/5 Turquoise/5 Wooden Planks/5 Thorny Fasteners`, derived from
  the documented tutorial grant, not from a recovered recipe row.
- Unknown: which tint variant a historical forge produced; the seven
  MatchThree server tables remain missing. No native client input was sent.

## Artifacts

- `prototype/osfr-tools/src/restoration/piece_014.rs` - catalogue, class/slot
  tables and `check_equip`. Not yet registered in `lib.rs`; the owned test
  `#[path]`-includes it until the lead registers it.
- `prototype/osfr-tools/tests/piece_014.rs` - seven focused tests.
- `scripts/pieces/piece_014.py` - read-only recovery and drift checker; writes
  only its report directory.
- `tests/pieces/test_piece_014.py` - four stdlib tests, including one over the
  real preserved corpus.
- `../lab/reports/piece-014-20261009T030806Z/forge-pattern-catalogue.json` and
  `summary.md` - authoritative evidence run (13 checks: 12 pass, 1 warn).

Pinned source hashes of that run: Legacy `ClientItemDefinitions.json`
`40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`,
`ItemClasses.txt`
`73c2a9377477ff88e6f4966614930c3d375cd9d5904fddaa7544ec24f0c2f567`,
`Profiles.json`
`334feb47ca36f4f429fc1646e6d75e776de24175ecf1ac880f217accecba8ea0`,
client `EquipmentSlotDefinitions.txt`
`5e55b7b9e465414371c52be09a0082ed015b1d68148c9525a5db65180fa2795a`,
prepared `inventory.json`
`cb6b30648bdd77885cdcaae3cd562f51ad92673742a5d13c3c961fd9327022fb`.

## Commands and results

```powershell
cd prototype\osfr-tools; & '..\..\scripts\cargo.ps1' test --test piece_014
# 7 passed; 0 failed

cd ..\..; & '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_014.py" -v
# 4 passed

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/piece_014.py
# status=pass; 12 pass, 1 warn (75030 class source disagreement), 0 fail
```

## Remaining gates

- Lead: register `piece_014` in `lib.rs` only when a consumer exists, or use the
  checker as the content regression gate. Shared gateway/forging integration is
  lead-owned and was not modified here.
- Native: actual Brawler equip of the forged hammer, Blacksmith rejection of
  class71 and the original anvil forge flow remain unexercised by this piece.
- Original: the seven MatchThree tables, per-attempt forge consumption and the
  historical tint selection stay explicit gaps.
