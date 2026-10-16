# 060 Dungeon instance lifecycle

Candidate implementation plus evidence bundle. Membership, spawn/reset, completion, exit and
restart/disconnect invariants are covered by 16 focused Rust tests and a 14-test evidence checker.
Independent review and the native run are still open; nothing here is a playable claim.

## Original recovered facts

Wire, from the preserved retail captures and the pinned community serializers:

- Dungeon entry uses the encounter family **41**: `u16 41 | u16 sub | i32 encounter id | i32
  instance id`. Three retail frames in `lab/reports/original-minigame-packets/` re-encode byte for
  byte through the new encoders, and the Rust tests hash them:
  `demoderby-23087-flow0-frame5078-op41-sub2.bin` (28 B, sha256 `ecf5eda4…e7bf`, header + `u64`
  player guid + one trailing byte), `demoderby-22789-flow0-frame119-op41-sub106.bin` (23 B, sha256
  `625e25ea…4b91`, header + `i32` state), `demoderby-22822-flow0-frame153-op41-sub107.bin` (19 B,
  sha256 `95c4e623…c3c3`, the bare header). These three frames come from minigame activities, so they
  prove the family header and the three bodies, not dungeon-specific field values.
- The community serializer for unmerged Sanctuary PR #76 (`9236220c44086ba9a31393cc30b5ebd44d05787f`)
  documents the same family and the derived sub-opcodes: `EncounterPacketPlayerEnter` 2,
  `EncounterStatePacket` 106, `EncounterZoneIsReadyPacket` 107, `EncounterDetailsResponsePacket`
  114, `EncounterPacketIsFighting` 133, with file sha256s recorded in the evidence bundle.
- Encounter state walk: `41/107` flips the offer popup into the GO button, then `41/106` walks
  4 ready → 5 after GO → 6 running. The shipped client carries the matching handlers
  `MiniGame:SetMiniGameReady` (line 1195728), `MiniGame:RequestStartGame` (1206499),
  `MiniGame:LeaveMiniGame` (1205447, 1206939), `MiniGame:RequestRestartGame` (1205032) and
  `MiniGame:OnGameOver` (1192885) in `lab/decompiled/client/FreeRealms/pseudocode.c`
  (sha256 `f84a5867…bef3`).
- Settlement reuses minigame family **39** (`u16 39 | u8 sub | i32 state | i32 group | i32 game`),
  recovered by work item 016 from nine preserved retail frames and native constructor
  `FUN_008a4420`: `39/18` game over is `u8 Won` plus the `0, 0, 1` triple, `39/19` is the bare
  header. Piece 016 records that neither `39/18` nor `39/19` appears in any preserved capture; the
  checker counts 7 captured `39/23` knockouts and 2 `39/7` leave requests as the family evidence.
- The preserved corpus contains 59 family 41/39 frames across 11 family/sub pairs, all decoded by
  the checker (`lab/reports/piece-060-20261009T041903Z/evidence.json`).

Corpus, from the pinned wiki `wiki.sqlite` (page revisions and content hashes in the bundle):

- "Dungeons" (rev 49285): dungeons are combat minigames with static entrances, ranked 1–5;
  difficulty 3/4/5 require a combat job of at least level 5/9/13, difficulties 1 and 2 "can be
  entered by anyone" (2 recommends level 3). The Rust table and the checker's corpus parse are
  independent implementations of this rule and both are asserted in the tests.
- "Groups" (rev 45235): groups are two or more players who fight in dungeons through an invite.
- "Bixie Hive" (rev 49119): bonus rewards grant "all players in the instance" — instance-scoped
  membership, and per-instance reward scope.
- "Achievements" (rev 48170): 36 "complete the … encounter with a full group" rows, so a full-group
  completion is a real concept.

## Inferred meaning

The op41 identity pair is an encounter id plus a run/instance id, and `41/2` names the player guid, so
the client is told *who* entered *which* run. Combined with the corpus wording ("players in the
instance", groups of two or more), membership is instance-scoped rather than global. Dungeons being
"combat minigames" is the bridge between the encounter family (entry, HUD, objectives) and the
minigame family (start, leave, restart, game over, state removal).

The three re-encoded frames are activity traffic, so they are evidence for the *encoding*, and the
unmerged PR is evidence for the *field order*; neither says how an original dungeon server ordered
entry, spawn and settlement. That ordering is where the reconstructed policy lives.

## Reconstructed policy (no original instance manager recovered)

`prototype/osfr-tools/src/restoration/piece_060.rs` implements one instance lifecycle plus a
registry:

- **Membership**: entry is gated by the recovered difficulty table and a caller-supplied capacity;
  the roster freezes when the GO packet is emitted; duplicate entry is rejected; one player may be
  in only one live instance.
- **Spawn/reset**: actor ids are registered per generation, never reused; a spawn requires a
  connected member and cannot repeat inside a generation; `reset` clears the generation, retires its
  actor ids and resumes at state 6. The owning engine decides *when* a wipe happens — this module
  validates ordering only.
- **Completion**: requires the active phase, a member, a connected member and every spawned actor of
  the current generation defeated; it is once-only and emits the recovered `39/18` + `39/19` pair.
  The result carries ids and the roster, never an amount.
- **Exit**: a member leaves; the last member closes the instance and receives `39/19` unless the
  completion already sent it.
- **Restart/disconnect**: a dropped connection keeps its slot until a caller-supplied grace deadline;
  a reconnect before the deadline restores the member *without* respawning or resetting the run; an
  expired membership is rejected and reaped, and reaping the last member closes the instance.

Capacity and grace are caller-supplied on purpose: the corpus has no party-size rule and no
disconnect rule, and nothing was invented to fill either.

## Gaps

- No captured dungeon instance traffic exists; the op41 bodies rest on an unmerged community PR plus
  activity frames. The original server-side instance manager, its roster rules and its spawn/reset
  ordering remain unrecovered.
- Party capacity, disconnect grace and any reset/lockout period are unrecovered (the "Dungeons" page
  mentions none). They stay configuration values.
- No wipe/defeat condition, enemy health, wave or gate rule (work item 061) and no reward amount
  (work item 063) is modelled. The module never grants money or items.
- No roster-removal, invite, kick or leader packet is written; only the four bodies with recovered
  byte layouts are emitted. Membership changes outside the terminal packet set are silent.
- `39/18` and `39/19` have no captured frame (same gap work item 016 records).

## Artifacts and reproduction

- `prototype/osfr-tools/src/restoration/piece_060.rs` (module, `#[path]`-included by the test until
  the lead registers it)
- `prototype/osfr-tools/tests/piece_060.rs` (16 tests; RED was a missing-module compile error,
  GREEN is 16 passed / 0 failed; a second in-review RED proved that leaving a live instance must
  free the player in the registry, and `Registry::exit` was fixed against it)
- `scripts/pieces/piece_060.py` and `tests/pieces/test_piece_060.py` (evidence checker, 14 tests)
- `../lab/reports/piece-060-20261009T041903Z/` (RED/GREEN logs, `evidence.json`, `findings.json`,
  `provenance.json`, `run.log`, README)

```powershell
.\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_060
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p 'test_piece_060.py' -v
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_060.py' --reports-dir '..\lab\reports\piece-060-20261009T041903Z'
```

## Skills applied

- **reverse-engineering**: read the decompiled client handler strings and the pinned community
  serializers to recover the family/sub-opcode layouts and field order.
- **game-research-websearch** (pinned-corpus form): queried the preserved wiki database and the
  preserved community reference trees already pinned under `lab/references`; no new network fetches
  were needed because a community serializer for this family already exists.
- **ponytail**: one member map, one spawn record, one membership index; no packet writer for
  unrecovered bodies and no tick loop.
- **caveman**: concise reporting only.

## Native and integration gates

Native acceptance stays with the lead: a real client run must show entry (`41/2`), zone ready
(`41/107`), the state walk (`41/106` 5→6), spawn, settlement (`39/18` + `39/19`), exit and a
reconnect. Integration for the lead is registering the module, de-duplicating the terminal encoders
against `src/encounter.rs` and work item 016, and wiring capacity/grace from real evidence when it
is recovered.
