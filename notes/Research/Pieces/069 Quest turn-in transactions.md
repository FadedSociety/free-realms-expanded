# 069 Quest turn-in transactions

Status: scoped turn-in transaction implemented and locally verified, 2026-10-09 05:30 UTC.
Evidence label: **RECONSTRUCTED server transaction over ORIGINAL packet and native turn-in
observations**. This is not a native acceptance claim and no client has accepted anything this
module emits.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_069.rs` | durable turn-in ledger, staged/commit/recover boundary |
| `prototype/osfr-tools/tests/piece_069.rs` | 9 owned vectors, `#[path]` include |
| `scripts/pieces/piece_069.py` | read-only checker over the native turn-in proofs |
| `tests/pieces/test_piece_069.py` | 11 stdlib checker tests |
| `lab/reports/piece-069-20261009T052427Z/` | RED/GREEN logs, checker report |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Why this item exists

Both quest engines in this crate flip the quest's completion flags and hand back a reward bundle in
the same step that they apply it. `gameplay::apply_quest` and `gameplay::apply_opening` grant the
coins first and then each item one at a time, so a failure or a crash in the middle leaves a quest
that says `rewarded = true` with a partial or missing reward, and a repeated native turn-in request
has no recorded answer to replay. The observable once-only reward is already native-accepted
(`Notes/Reviews/Native Cooking Finish and Mae 171251.md`), but nothing durable records *which*
request produced it.

This module adds that record. It never grants and never writes the gameplay snapshot; it stages one
bundle per `(quest_id, attempt)`, answers an exact repeat from the record, and resolves a crash by
reading the persisted quest state.

## RED evidence

The owned test was written first and failed against a missing module:

```
error: couldn't find file `tests\..\src\restoration\piece_069.rs`
  --> tests\piece_069.rs:29:1
```

Writing the vectors also settled one contract question before implementation: the caller resolves
`progress` by `request.quest_id`, so an unresolvable quest is `None` and is refused as
`UnknownQuest`. The first draft passed another quest's progress for an unknown id and expected a
rejection; that expectation was wrong, not the module, and the vector now asserts the real contract.

## Original facts used

Everything below is read from the supplied server's decompiled enum or from preserved native
captures. None of it is invented.

- Quest packet family: the supplied server's `BaseQuestPackets` in
  `lab/decompiled/server/LibSOE/SOE/BaseQuestPackets.cs` defines `QuestInfoPacket = 1`,
  `QuestReplyPacket = 2`, `QuestAddPacket = 3`, `QuestCompletePacket = 4`, `QuestFailedPacket = 5`,
  `QuestAbandonedPacket = 6`, `QuestObjectiveAddedPacket = 7`, `QuestObjectiveActivatedPacket = 8`,
  `QuestObjectiveUpdatePacket = 9`, `QuestObjectiveCompletePacket = 10`,
  `QuestObjectiveFailedPacket = 11`, `CompletedQuestCountUpdatePacket = 12`,
  `QuestEndPacket = 13`, `QuestEndReplyPacket = 14`, `QuestStartBreadcrumbPacket = 15`,
  `QuestEndBlockedPacket = 16`.
- Cooking turn-in (native, `client-baseline-20261008T171251Z`,
  `analysis/native-cooking-finish-turnin-proof.json`): four quest packets only, all family
  `0x0031` — objective update (opcode 9) and objective complete (opcode 10) for quest `910004` at
  frames 3876/3877, then exactly one `QuestCompletePacket` (opcode 4) at frame 4071 and exactly one
  `CompletedQuestCountUpdatePacket` (opcode 12, count 4) at frame 4072. One wallet observation pair
  `110 -> 120`, one `ChefXP` pair `0 -> 25`, `stage4.rewarded = true`, `stage5_unaccepted = true`,
  and `repeat_Mae_no_reward_progress_packet = true` for the second Mae selection.
- Bartle turn-in (native, `client-baseline-20261008T193318Z`,
  `analysis-live/native-bartle-turnin-nextquest-proof.json`): one wallet value (194), one XP value,
  one quest removal (`910009`), one next-quest add (`910010`), `reward_emitted_once = true`, and
  `next_Bartle_interaction_accepts_artifact_quest_without_repeating_reward = true`.

Those captures are evidence that the *observable* once-only turn-in exists in the running
reconstruction. They do not exercise this module and do not prove an original server transaction.

Two pinned community cross-checks were read before writing anything custom, and neither supplies a
turn-in implementation to reuse:

- `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
  independently maps family `49` to the same opcode names (`4 QuestCompletePacket`,
  `12 CompletedQuestCountUpdatePacket`, `13 QuestEndPacket`, `14 QuestEndReplyPacket`,
  `16 QuestEndBlockedPacket`) and shows family `50` as the reward family
  (`RewardBundlePacket`/`RewardPacketNonMemberRewardGranted`). The two native turn-ins above carry
  no family-50 packet, so the original reward-bundle path is untouched and remains a gap; this
  reconstruction grants through the existing local inventory instead.
- The same checkout's handler tree has no quest packet handler, and its only quest persistence row
  (`Sanctuary.Database/Entities/DbCharacterQuest.cs`) stores `QuestId`, `Completed`, `GoalProgress`
  and `GoalCount` with no reward or transaction field. That absence is why the ledger below is
  labelled reconstructed rather than recovered.

## Reconstructed policy

- Ledger: one append-only JSONL document beside the snapshot. Records are
  `{schema, kind: intent|applied, quest_id, attempt, request_id, reward?}`; an `applied` record
  without its `intent`, a duplicate intent, a reward carried on an `applied` record, an unknown
  schema and an oversized or malformed document are all refused rather than repaired.
- `consider` order: invalid identity -> `UnknownQuest` -> `NotAccepted` -> `ObjectivesIncomplete`
  -> recorded receipt (`Replay`, or `StaleAttempt` when neither the exact request nor a rewarded
  quest matches) -> `AlreadyRewarded` when the persisted state shows a reward this ledger never
  recorded -> dangling intent with a different request (`StaleAttempt`) or the same one (`Apply`
  again, no second record) -> `StaleAttempt` for a reused attempt -> record the intent and
  `Apply`.
- Commit point: the caller applies the staged bundle and persists the whole gameplay snapshot with
  the existing atomic replace (`game_store::save`), so quest flags, items, currency and XP land
  together. `commit` then records that the snapshot landed.
- Recovery: an unsettled intent whose persisted quest already shows `rewarded` is settled without a
  grant; otherwise it replays its bundle exactly once. `commit` is idempotent for the same
  `(quest, attempt)`.
- Bounds: 256 KiB document, 4096 records, 16 item rows per bundle, `i32::MAX` coins, 10,000,000 XP.

## Verification

```
scripts/cargo.ps1 test --test piece_069      # 9 passed; 0 failed; 0 ignored
scripts/cargo.ps1 test --lib                 # 24 passed; 0 failed; 0 ignored
lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_069.py
                                             # status=pass, 26 checks
lab\tools\wiki-env\Scripts\python.exe -m unittest discover -s tests/pieces -p "test_piece_069.py" -v
                                             # 11 tests, OK
```

`rustfmt --check` is clean on both owned Rust files. The checker binds the module's opcode constants
to the decompiled enum, re-decodes all four Cooking quest packets, and re-asserts the single wallet
delta, single experience delta, single complete/count-update pair and the repeat flag. The checker
report is in the report directory above.

## Limits and remaining gates

- The ledger document, its schema, ordering, rejections and bounds are reconstructed. No original
  server transaction source was recovered.
- Nothing here is wired into `gameplay.rs`; the lead owns that integration, and until it lands a
  crash between the quest flag change and the reward grant is still possible in the running server.
- The per-quest reward numbers stay in the content owned by the other items; this module carries
  whatever bundle the caller stages and never mints value.
- Native acceptance (a real client turn-in replaying through this ledger, including a kill/power
  loss between the intent and the snapshot) remains an explicit gate.
- Repeatable quests are deliberately refused: a recorded quest answers only its exact request or a
  rewarded replay, and a later attempt on the same quest id is `StaleAttempt`. Neither native proof
  shows one quest id turned in twice, so that rule waits for evidence instead of guessing.
- Wiring is not drop-in for the legacy quest engine: `quests.rs::enemy_defeated` sets `completed`
  and `rewarded` in the same step as the reward, so the lead must pass the pre-turn-in progress
  (or split "ready" from "rewarded") for `consider` to see a first request.
