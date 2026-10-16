# Definition-Driven Quest Engine (W21)

Status: focused candidate module. `quest_engine.rs` is not registered in `lib.rs`, is not routed through the gateway, and has no native acceptance of its own.

## Evidence boundary

`notes/Build Guides/Build Guides.md` says the guides were generated from `lab/native-map/run/map.sqlite`; the feature specification is `notes/Build Guides/Gameplay/Quests and Objectives.md`. The quest guide distinguishes recovered client requirements from server policy. There is no original server implementation, so eligibility, event bindings, payout and persistence rules below remain reconstructed `(R)` unless stated otherwise.

Recovered client facts used by this module:

- Quest records and their objective states arrive from the server; client has no quest/objective definition table. Offer is family 49/1 (reader `00c7bb60`, UI handler `00a94be0`). Accept and decline are client family 49/2 with `i32 quest id, u8 accept` (senders `00a97d70`, `00a97e10`). QuestAdd is 49/3 (`00c7cc80`, `00c7ccd0`, apply `00c7bd10`).
- Objective activation, update and completion are 49/8, 49/9 and 49/10. Their server writers/readers are respectively `00c7b570`/`00c7a910`, `00c7b5e0`/`00c7a990`, and `00c7b650`/`00c7aa10`. Runtime objectives are keyed by definition id, not list position (`00baf458`, `00bab990`; also `Native Journal Live Objective Keys`).
- Turn-in screen is 49/13 (`00c7bbd0`, `00c7c030`, consumer `00a95420`); client reply is 49/14 with quest id (`00a97ef0`). Abandon is client command 26/0x17 (`00c09f60`, `00c0a500`) and the server removes it with 49/6 (`00c7b500`). Completed count is absolute in 49/12 (`00c7b730`).
- Quest-limit notice is family 47/subtype 15, byte 1 for member and 2 for non-member (`00a91bf0`). The client reads member/non-member limits from self-data; their historical values are unknown. An observed `30` is reconstructed local service data, not a recovered retail limit.
- Journal definitions 209/1 have keyed tables of key plus 7, 10, 3 and 9 dwords (`00a45d80`, `00a45d10`). Journal set 209/2 has a count and key/value dword pairs (`00a45bf0`, `00a45680`). The Rust startup builders are `startup_world::build_journal_definitions` and `startup_messages::build_journal_set`.
- Quest updates and rewards travel in tunneled messages. `QuestMessage::encode_exact` returns inner bytes only for the decoded bodies; the gateway owns tunnel framing.

## Model and policies

`QuestState` is character-scoped and serializable. Each present quest row is offered, active with one counter per objective, completed, or abandoned. A single pending offer records the offered id that can be accepted or declined; a single pending turn-in records the 49/13 to 49/14 handshake. State validation binds a reload to the same character and definition ids, checks counter bounds and requires completed objectives before a saved completion/turn-in. It also persists selected quest and auto-switch lock after the caller decodes 47/12 and 47/13; the caller owns inbound decoding.

`QuestEngine::from_content_db` converts current `content::Quest` and `ContentGoal` rows. `CharacterEligibility` and `QuestRequirements` are policy inputs because current `ContentDb::Quest` does not store player/job level or membership requirements. The engine checks prerequisite completion, level, job, membership and active/offered count before offering. The configured member limits are service policy `(R)`; the client bytes do not recover historical values.

The objective event API accepts talk-to-NPC, kill-actor-kind, collect-item, enter-area, finish-minigame and use-item events. It also supports the extra local opening event kinds: POI reach, NPC interaction, gathering, crafting, job selection, equipment, dungeon completion, ability use and a position/radius visit. Event authenticity, actor range, current job, inventory ownership/consumption and minigame victory must be verified by the upstream authoritative subsystem `(R)`; this module does not trust a raw client report.

Current `ContentGoal` can encode only defeat/model id, NPC talk, item collection, POI reach and zone visit. Finish-minigame and use-item definitions cannot yet be expressed in ContentDb JSON; the new event API supports those kinds, but a later shared content-schema change is needed to author them as data. ContentDb also lacks quest description/helper locale ids, turn-in NPC, level/job/membership fields and reward-display item rows. The factory uses safe placeholders for absent display strings and defaults absent policy requirements to none. It does not claim complete quest records or retail content.

`OpeningEngine` adapts into the new state machine through `QuestEngine::from_opening`. It links the 17 local opening rows sequentially as prerequisites and preserves their objective order, event text, guide ids and rewards. This is a compatibility adapter for the parity test; the native accepted 17-stage behavior remains owned by `opening.rs` until lead integration. The old `quests.rs` is a separate local command/counter and checkpoint-activity service, not the 17-stage event oracle; it remains unchanged.

## Message sequences

`QuestOutcome::outbound` preserves order. Offer emits intent 49/1. Accept emits 49/3 then 49/8 for currently active objectives. Progress emits 49/9, then 49/10 when the count reaches its requirement, then 49/8 for the next ordered objective. Talk at a completed turn-in NPC emits 49/13; `finish_turnin` is the server-side 49/14 handler and emits `rewards::apply_grant` packets, then 49/4 and the updated absolute 49/12 count. Abandon emits 49/6 and no grant. Limit denial emits 47/15.

Server-side policy `(R)` in these sequences: the engine derives objective events; clamps counters to required counts; sends visible notifications; averages normalized objective counters for quest progress; treats unknown trailing message fields as zero; and uses grant key `quest:<character_id>:<quest_id>`. `rewards::apply_grant` returns exact transaction packet bytes and a new reward snapshot. The caller must persist quest state, inventory, profiles and applied keys in one save transaction before sending the returned sequence. A committed key makes a retry safe even if the caller restores a pending turn-in snapshot.

`QuestMessage::encode_exact` builds only protocol bodies supported by the guide: 49/8, 49/9, 49/10, 49/4, 49/6, 49/12 and 47/15. Visible-flag values and unknown trailing dwords are reconstructed policy `(R)`. 49/1 and 49/13 body order is unresolved; 49/3 is a full quest record but this engine does not yet own every required field or verified bundle row. These are typed opcode intents with no encoded body. Do not send an intent as a packet. The first login journal sequence is 209/1 then 209/2, built by the existing startup modules; journal key meanings and set values must come from the authored journal/content service.

## Fixture and verification

`prototype/osfr-tools/tests/fixtures/quests/opening.json` contains the 17-stage local opening fixture. It keeps the legacy loader shape so `OpeningEngine` can remain the reference oracle. Explicit source tags live in its `provenance` index, keyed to actor, quest and objective ids; each is `(R)` and does not upgrade a local quest binding into a recovered retail fact. The fixture began from `lab/references/opening-anvil-placement-v1/content/opening.json` (SHA-256 `4e6ac15cd19b5785a04ac7eb944bf0235f4a0592f4c42df755f89c47896b0ad1`), then removed the discovery rows and replaced external provenance with local reconstruction tags. The separate native 17-stage acceptance is recorded in `notes/Current Progress.md` and `notes/Reviews/Native Opening 17 103605.md`.

Test command, from `prototype/osfr-tools`:

```powershell
$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W21'
& '..\..\scripts\cargo.ps1' test --test quest_engine
```

Final result: **8 passed, 0 failed, 0 ignored**. The opening parity test runs both `OpeningEngine` and the new engine through each fixture objective and turn-in, then compares the exact outcome text sequence and all 17 quest reward summaries. Other tests cover ContentDb conversion, all six requested objective kinds, offer/accept/decline, prerequisite/level/job/member/limit gates, abandon, reload including selection/lock state, duplicate turn-in retry, known packet bytes and 209/1-before-209/2 login order. First compile/test attempts exposed two test ownership errors, fixture tags rejected by the legacy strict schema, a wrong limit context and zone 0 rejection; these were corrected before the final green run.

`rustfmt` was run only on `src/quest_engine.rs` and `tests/quest_engine.rs` using the installed private toolchain executable under `lab/tools/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin`.

## Lead integration steps

1. Register the module in `src/lib.rs`; preserve the current `quests.rs` and `opening.rs` routes during staging.
2. Extend the shared quest content schema with source-tagged level/job/member requirements, description/helper ids, turn-in NPC, ordered activation metadata, finish-minigame/use-item goals and exact display bundle rows. Keep unresolved values tagged `(R)`.
3. Route authoritative NPC interactions to `offer`/`begin_turnin`; decode 49/2 accept/decline and 49/14 reply into `accept`/`decline`/`finish_turnin`. Verify caller session, range, actor identity, job and consumed item before events.
4. Add verified builders/readers for complete 49/1 offer, 49/3 quest record and 49/13 end-screen bodies before sending those intents. Keep the documented 49/9 then 49/10, grant, 49/4, 49/12 ordering.
5. Persist `QuestState` and the returned `RewardState` together. Compose self-data quest records and membership activation with login; send 209/1 before 209/2. Reconcile selected quest, auto-switch and hub/sticker state with the journal tables.
6. Migrate the 17 local opening rows and native save fields into `QuestState`; rerun old/new parity and native acceptance before retiring hard-coded opening bindings. Do not treat this candidate test as native or release acceptance.
