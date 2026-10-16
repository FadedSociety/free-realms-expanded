# 038 Adventurer progression

Status: closed for review, 2026-10-09 04:20 UTC. This piece recovers the **exploration/collection
unlock rules** and the **profile-scoped collection progress** model. It is evidence plus a checker
plus a small typed module; it is not a playable claim and it grants nothing.

Owned paths: `prototype/osfr-tools/src/restoration/piece_038.rs`,
`prototype/osfr-tools/tests/piece_038.rs`, `scripts/pieces/piece_038.py`,
`tests/pieces/test_piece_038.py`, this note and `lab/reports/piece-038-20261009T0405Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Client update packets. The original client binary's `ClientUpdatePacketCollection*` constructors pass
their opcode to `FUN_0089d180`: **8** `ClientUpdatePacketCollectionStart`, **9**
`ClientUpdatePacketCollectionRemove`, **10** `ClientUpdatePacketCollectionAddEntry`, **11**
`ClientUpdatePacketCollectionRemoveEntry` (`pseudocode.c:973451`, `:973510`, `:973569`, `:973618`;
the next constructor, 12, is `ClientUpdatePacketUpdateLocation`, so the family is exactly four wide).
The pinned community project declares the same 8 and 10 for the two classes it ships and has no
class for 9 and 11 — partial coverage, recorded rather than smoothed over.

Progress on the wire. Collection progress is not only a flag on a list; it is also a reward-bundle
entry. `RewardBundleEntryCollectionAdd` returns type id **10** and formats a single `CollectionID`;
`RewardBundleEntryCollectionEntryAdd` returns type id **11** and formats `CollectionEntryType` plus
three parameters (`pseudocode.c:2433757`, `:2433819`, `:2434802`, `:2434834`). The type-11 variant
does not appear in the accepted initialization payload at all, and the type-10 variant appears 126
times there.

Client UI and data sources. The same decompile holds the action/event pair
`StartCollection` / `CollectionStarted` (`:1370479`, `:1370526`), `UpdateCollection` (`:1370693`) and
`CompleteCollection` / `CollectionCompleted` (`:1370772`, `:1370819`), the `CollectionCategories.txt`
data file (`:1253992`; Ghidra exports the file name as the symbol `CollectionCategories_txt`), its
`Collection Category ID` / `NAME` / `ICON_ID` columns (`:1253751`, `:1253757`, `:1253759`), the
`BaseClient.CollectionData` table with a `Reward` column (`:1171225`, `:1687247`) and the two
counters `CollectionsCompleted` (about-me field 5, `:1653347`) and `CollectionCount` (profile column
10, `:1799728`) together with the `AboutMeCollectionCount` panel element (`:1653732`). The collection
count is therefore a profile-level number surfaced in the about-me/profile fields, not a per-session
one.

Accepted initialization payload. `PacketSendSelfToClient.json` is the supplied emulator's
initialization payload, and the original client logs in and enters the world with it
([[Rust Gateway Milestone]]). It carries **253 collection rows, 1815 entries, 841 collected entries
and 55 complete collections** for one profile. The profile identity in the supplied data is a
placeholder (`PlayerGUID` 1, name "Open Source Free Realms", Coins 0) while the collection progress
inside it is partial, so the row shape is what this piece relies on, not the identity. Every entry
carries `id`, `index`, `collection_id`, `name_id`, `icon`, `tint`, one
unnamed int and a collected flag; the collection id is written twice per entry (list key and first
value field). Two findings the lead should not smooth over:

* **19 rows arrive out of ascending index order** (244-260 descend; 277 and 278 are arbitrary
  permutations). The index *set* of every row is still exactly `1..n`, so this is a delivery order
  fact, not corrupt data. The module normalises on definition order and the checker reports the list
  verbatim.
* the reward bundle attached to a collection row is not uniformly a completion reward: 126 rows carry
  a `RewardBundleEntryCollectionAdd`, alongside 151 experience, 187 item, 7 quest and 40 token
  entries. All 62 distinct parameters fall inside the captured collection id range (48 of them are
  collections this profile already owns) and 14 are not. That is *consistent* with a cascading
  collection grant but the meaning is not recovered, so no behaviour depends on it.

## Community facts used as the unlock model

From the read-only wiki corpus (`wiki.sqlite`; revisions are in the evidence document):

* the **Adventurer** is the default job, "experience is mainly gained by completing collections",
  could be unlocked by speaking to **Michael Tallstrider** after finishing the first quest in the
  tutorial, and was unlocked automatically when the tutorial was skipped. The page also carries a
  sunset restriction ("this job is no longer going to be unlocking");
* `Collections` names six types — exploration, rare exploration, explorer, item, meta and job
  collections — and describes how each one starts: exploration entries activate by **approaching**
  a location, rare exploration entries are sparkling **coins that must be clicked**, item entries
  come from sparkling spawns, and explorer collections are collections **of the other collections**;
* `List of Collections` enumerates **19 exploration**, **12 rare exploration** and **4 explorer**
  collections. `Exploration (Collections)` lists the individual locations and repeats the reward
  text ("Adventurer Exp." for the common sets, Pathfinder/Seeker/Traveler clothing for the rare
  ones). No numeric experience value appears anywhere in that text.

Community cross-check. The pinned Sanctuary project's `Collections.json` carries a single row
(collection 10, eight entries, item definitions 11081-11088) whose entry ids match the captured row
10 exactly, and its `ClientCollectionEntry` writer emits nine values — the same count as the
accepted payload once the list key is included. The field counts agree and the duplicated
definition id matches; the byte-for-byte mapping remains an inference, not something this piece
verified.

## Reconstructed boundary

The module implements reconciliation rules only, and reports outcomes instead of granting anything:

* an entry is collected at most once; a replayed `AddEntry` returns `DuplicateIgnored`;
* a collection reports `Completed` exactly on the transition that fills the last entry, so a reward
  emitter cannot double-fire, and `completed_count` is the derived `CollectionsCompleted` value;
* updates and snapshots carry the owning profile GUID and are refused with `ForeignProfile` when they
  name a different one, which is what keeps two characters' collection progress apart;
* `Remove` clears progress but keeps the definition, so a collection can be re-earned; `RemoveEntry`
  reverts completion;
* exploration entries unlock by proximity, rare-exploration and item entries return
  `RequiresInteraction` for proximity alone, and explorer entries return `CompletedByChildren` —
  the child collections are resolved by `resolve_explorer_unlocks`, once per transition.

Nothing numeric was invented: the proximity range is a caller argument (validated finite and
non-negative), the reward values are not modelled, and the module never touches coins, items or
experience.

## Explicit gaps

* the proximity radius that activates an exploration entry is unrecovered; only the `Reward` column
  of the client's collection data table is named, not its values;
* no collection reward amount, item or currency value is recovered — only the reward-bundle entry
  kinds and the captured per-collection reward rows;
* item definition ids for item collections are work item **074**'s scope, and the discoveries of
  item **073**; neither is duplicated here;
* the captured payload's unnamed fields keep their positions only; their meaning is unverified, and
  the `CollectionAdd` parameter interpretation is an open question;
* the four client update packets have recovered opcodes and field names from the client classes, but
  no payload writer exists yet on the server side;
* the wiki's collection lists are community transcriptions; the original collection definition table
  is not present in the extracted client assets.

## Verification

```
& ..\..\scripts\cargo.ps1 test --test piece_038
  RED  before implementation: couldn't find file `tests\..\src\restoration\piece_038.rs`
  GREEN: 15 passed; 0 failed; 0 ignored
& ..\..\scripts\cargo.ps1 build --lib
  Finished `dev` profile
& '..\lab\tools\wiki-env\Scripts\python.exe' 'tests\pieces\test_piece_038.py' -v
  RED  9 tests failed while the checker was absent
  GREEN: Ran 9 tests, OK
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_038.py' --output <report>\piece-038-collections.json
  piece 038 evidence ok: 253 collection rows/1815 entries/841 collected, 19 exploration names
```

The checker fails closed: it rejects a changed opcode, a duplicated collection row, a non-permutation
entry index set, a drifted community table and a drifted wiki unlock rule, each with a named check.
Hashes for the client decompile, the accepted payload, the community table and packets and the wiki
database are in `lab/reports/piece-038-20261009T0405Z/source-provenance.txt` with the anchor line
numbers.

## Next gates

1. Lead integration: register `pub mod piece_038;` under `src/restoration/` and drop the `#[path]`
   include from the test. Keep item 074 as the owner of item/collectible definitions and 073 as the
   owner of discovery triggers.
2. Wire integration: the four recovered update packets need writers (collection id, entry id, index,
   name/description/icon/tint, reward bundle) before a restored server can push live progress; the
   accepted payload already gives the row shape the client parses.
3. Economy: completion must emit the completion transition exactly once into the lead-owned atomic
   economy before any reward is granted; this module deliberately grants nothing.
4. Native acceptance: the lead must walk a real exploration location and click a real coin in the
   client and confirm the journal/collection panel count before any playable claim.
5. Evidence review: re-check the 19 non-ascending rows and the 126 `CollectionAdd` reward rows
   against a fresh capture before treating either as a protocol rule.

Cross-links: [[Wiki Research Database]], [[../Design/MMO Worker Contract]], [[../../MMO Work Queue]].
