# 059 Dungeon definitions

Acceptance: inventory recovered maps/encounters/entrances/difficulty with historical coverage. Work
item [[../../WorkItems/059]] under the [[../../Design/MMO Worker Contract]] classification rules.

Artifacts

- `prototype/osfr-tools/src/restoration/piece_059.rs` - the recovered roster plus validation and
  lookup helpers (lead owns registration in `src/lib.rs`).
- `prototype/osfr-tools/tests/piece_059.rs` - 11 acceptance tests, `#[path]` include until the lead
  registers the module.
- `scripts/pieces/piece_059.py` - evidence checker over the wiki, locale, pack catalogue and
  terrain table.
- `tests/pieces/test_piece_059.py` - 10 checker tests, including four hard-failure cases.
- `../lab/reports/piece-059-20261009T054317Z/` - `dungeons.json`, `provenance.json`,
  `red-evidence.txt`, `checker.log`, `python-tests.log`, `rust-tests.log`.

## Original facts

**Roster and difficulty.** The read-only wiki corpus page `Dungeons` (page id 15) lists 43 combat
dungeons in five ranked sections: difficulty 1 has 8 rows, then 8, 9, 9 and 9. Two rows are marked
"(removed)": Hewey's Escape and King of the Ring!. The page states the entry rule: difficulty 1 is
open to anyone, 2 recommends level 3, and 3/4/5 require an entry with a combat job of level 5/9/13.
This is community documentation, not first-party data.

**Achievement block.** The preserved locale export (`locale-id-to-text.json`, 85,932 rows, reader
output over the original `locale/en_us/data.dat`) carries one contiguous block, ids 30439-30475, of
"Complete the <name> encounter." rows. The block is contiguous and ascending in roster order and
covers 37 of the 43 dungeons - every live row except Bixie Hive, Cursed Graveyard!, Sandscale Oasis
and Precursor Ruins, and, as expected, neither removed dungeon. Two rows in the block are not
byte-identical to the roster: `Complete the The Bat Cave! encounter.` (doubled article) and
`Complete the Misty Mountain  encounter.` (doubled space). `Dark spore Depths` (id 30470) differs
only by case. All three stay recorded verbatim; the doubling is normalized only to bind the row.

**Entrances.** Each dungeon page opens with "The <name> is a difficulty N dungeon that is started by
clicking on the <trigger> in <place>." 41 of 43 pages yield a usable sentence; every page carries the
Atlas icon sentence. `place` is stored verbatim. `region` is this piece's inference, resolved
against the wiki's own location vocabulary (first mentioned token among Sanctuary, Snowhill, Wilds,
Seaside, Blackspore Swamp, Briarwood, Wugachug, Merry Vale, Shrouded Glade, Sunstone Valley,
Crossroads, Highroad Junction, Stillwater Crossing). Precursor Ruins is a lore page, not a dungeon
page, and Treeleaf's Retreat has no click sentence, so both entrance rows are explicit gaps.

**Maps.** The pinned pack catalogue (108,547 rows) holds a zone definition (`*.gzne`, header
`GZNE` v2) for 14 roster dungeons, plus map chunks (`*_<col>_<row>.gcnk`):

| Dungeon | Zone stem | \*.gzne bytes | chunks | chunk bytes |
| --- | --- | --- | --- | --- |
| Bixie Hive | `sg_bixie_hive` | 1,944 | 9 | 194,780 |
| Arachnia's Lair | `bw_spider_lair` | 2,431 | 4 | 2,697,568 |
| Highroad Hijinx | `sg_highroad_hijinx` | 3,805 | 25 | 8,083,144 |
| Sweetwater Climb | `wc_sweetwater_climb` | 5,438 | 30 | 6,251,162 |
| Bone Bog Cemetery | `bs_bone_bog_cemetery` | 7,793 | 30 | 14,619,138 |
| Floren Forest | `sg_floren_forest` | 2,027 | 0 | 0 |
| Misty Mountain | `gl_misty_mountain` | 1,940 | 16 | 15,959 |
| Bandit Hideout! | `sg_bandit_hideout` | 664 | 2 | 28,728 |
| Frostfang Caverns | `sh_frostfang_cavern` | 1,656 | 9 | 20,288 |
| Mugwort's Hollow | `sg_mugworts_hollow` | 1,552 | 7 | 7,049 |
| Treeleaf's Retreat | `bw_treeleaf_retreat` | 3,356 | 4 | 2,678,705 |
| Cray Caves | `ss_cray_caves` | 3,056 | 4 | 123,790 |
| Tanglewood Fort | `bw_tanglewood_fort` | 3,331 | 9 | 4,138,889 |
| Trail of Betrayal | `bw_trail_of_betrayal` | 4,078 | 35 | 6,629,059 |

43,071 zone-definition bytes were read back with the `GZNE` magic; 53,092,522 chunk bytes are
counted, not re-parsed. `mv_den_of_secrets.gzne` is a 84-byte header-only stub while four
`mv_den_of_secrets_*` chunks exist, so Den of Secrets is a gap: a map without a recovered zone
definition. Same-name candidates with a header-only definition stay gaps too, each recorded with
its chunk count in `dungeons.json`: `sg_tavern_cellar` (84 bytes, 3 chunks), `sg_robgoblin_trove`
(84, 4), `sh_deep_mines` (84, 4), `bw_mushroom_cave` / `bw_mo_mushrooms` (84 each, 2 each),
`bw_briarheart_caverns` (84, 12) and `sg_haunted_mines` (56, 1). Candidates whose stem does not
carry the dungeon name - `sg_stillwater_crossing_combat`, `sg_sheep_watch`, `sg_bear_cave_combat`,
`bw_hedgemaze`, `bw_corrupted_vale` - are listed but counted in no coverage total.

**Cross-sources.** `Resources/ObjectTerrainData.xml` (34 rows) registers only one roster dungeon by
name: id 87 `DenOfSecrets`. The shipped `Resources/MiniGameData.txt` (783 rows,
sha256 `8ce3cc61...`) contains no dungeon at all - dungeons are combat encounters, not activity rows
- so it is recorded as a negative result rather than mined for difficulty. The `MiniGameData`
difficulty column must not be confused with dungeon difficulty ranks.

## Original vs reconstruction boundary

- Original: the wiki roster/difficulty/removal marks, the entrance sentences, the locale
  achievement block, the pack catalogue stems and byte counts, the terrain table row, and every one
  of the 43 names.
- Inferred (labelled in the checker): a zone stem matched to a dungeon name; the `region` token
  resolved from a verbatim place text; the `id` being the roster row number.
- Reconstructed: `entry_rule()` restates the wiki difficulty policy for the lead to enforce. No
  gameplay behaviour, spawn, wave, reward or balance value is reconstructed here.
- Not recovered: 29 of 43 zone definitions, encounter scripts, spawn tables, per-dungeon player
  counts, rewards, and all collision data.

## Verification

```
..\lab\tools\universal-modder-env\Scripts\python.exe scripts\pieces\piece_059.py --output ..\lab\reports\piece-059-20261009T054317Z\dungeons.json
  piece 059 evidence ok: 43 dungeons, 14 recovered zone maps, 29 zone gaps
..\lab\tools\universal-modder-env\Scripts\python.exe tests\pieces\test_piece_059.py -v
  Ran 10 tests in 10.248s / OK
.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_059
  test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The four hard-failure checker tests reject a duplicate roster row, an achievement name that does
not match a dungeon (`ObjectiveNameMismatch`), a declared zone stem that vanishes from the
catalogue (`DeclaredZoneMissing`), and an unreadable evidence file. The Rust suite additionally
rejects reordered ids, an out-of-range difficulty, a duplicate name, a duplicate zone stem, a
removed row carrying an achievement, and a row carrying both a zone and a stub. RED evidence is in
`red-evidence.txt`; it includes the mid-implementation failure that exposed the achievement block
being contiguous in roster order rather than offset from the row id.

## Limits and next gates

- Coverage: 14/43 maps by confirmed zone definition; 41/43 entrance sentences; 37/43 achievement
  rows (every row of the contiguous block binds to exactly one dungeon); 1/43 terrain
  registrations. The remaining 29 maps need either a recovered `*.gzne` or an explicit lead
  decision to accept a stub-plus-chunks row such as Den of Secrets.
- Candidate identities not counted as recovery: `sg_stillwater_crossing_combat` (2,403 bytes, 9
  chunks) for Cursed Graveyard!, `sg_sheep_watch` (865 bytes) for Sheep Watch, and
  `sg_bear_cave_combat` (560 bytes) for The Bat Cave!. Accepting them would move the count to 17;
  that needs evidence, not a name guess.
- Difficulty 1 is the weakest band: only Bixie Hive has a confirmed zone definition.
- Integration gates: register `piece_059` in `src/lib.rs`, decide whether the gateway uses
  `entry_rule()` to gate dungeon entry, and feed item 060 (instance lifecycle) and item 061
  (waves/gates) from this roster rather than re-deriving names.
- Native gate: none claimed. Nothing here was run in the client; no dungeon entry, encounter,
  reward or collision behaviour is verified.

## Skills applied

`game-research-websearch`/community-first practice for the wiki corpus and pinned catalogue before
any custom parsing, `reverse-engineering` for reading the locale export and GZNE catalogue shape,
RTK (normal commands first), Ponytail (reuse the existing locale export, catalogue and wiki rather
than new readers; one checker, one table, no speculative subsystem) and Caveman (concise
reporting). `asset-pipeline` and `game-automation` were not needed: no asset conversion and the
shared native window is lead-controlled.

Cross-links: [[../../MMO Work Queue]], [[../Starting Dungeon Audit]],
[[../Original Activity Coverage]], [[../Wiki Research Database]].
