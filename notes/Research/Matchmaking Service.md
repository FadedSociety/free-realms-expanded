---
type: research
updated: 2026-10-13
scope: W52 transport-free matchmaking and lobby service
---

# Matchmaking Service

## Scope

`src/matchmaking.rs` is a transport-free queue state machine and a codec for the matchmaking/lobby packet layouts whose widths are recovered. The integration test imports it with `#[path = "../src/matchmaking.rs"]`; the library wiring is one line in `src/lib.rs` after review:

```rust
pub mod matchmaking;
```

Queue definitions are supplied by the caller. This module does not copy queue catalogue data, select an activity from an undocumented queue id, own sockets, or alter a save. Each state transition returns `SessionOutbound` values addressed by `players::SessionId`; `join` returns an empty outbound list because the native 141/4 request record cannot yet be constructed without guessing its member fields.

## Recovered client facts

These are client layouts, not evidence of the original server's decisions.

| Fact | Evidence and confidence |
|---|---|
| Family 141 starts with two 16-bit words: family and sub-op. The list request 141/1 body is the two-word connection/session key. | Queue guide `00a7fb90` confirmed; body reader `00a9d4f0` likely. The preserved frame is 19 bytes: 7-byte tunnel wrapper, 4-byte inner header and 8-byte key. `racing-24987-flow1-frame42928-op141-sub1.bin`, SHA256 `B01CD62DD0C7BA457CB1DCCA6C951ADD4EF391BE116882AA3A00275501CAF986`. |
| A 141/2 queue list starts with the same 8-byte key, then a u32 row count. Each row is nineteen dwords and two bytes, 78 bytes total. The captured kart list has five rows. | `00aa0460` likely list parser; `00a9d9c0` likely row reader; columns in `00a81e90` and `00a7f2f0` are confirmed. Captured frame `racing-23865-flow0-frame44522-op141-sub2.bin`, raw length 413, inner length 406, SHA256 `1F743E8A0FD12A6965C4E38EE78ED1481674E8AF379C00D26DBD3A252AE550CF`. Five 78-byte rows account for the full inner length after the 12-byte prefix. |
| The first queue-row dwords include queue id, display name id, match type, min/max players, min/max teams, max start delay, seven parameters, encounter description/icon ids and two unread dwords; byte one is the member-only flag and byte two is unused. | `00a9d9c0` likely reader; field-use crosswalk from confirmed `00a81e90`/`00a7f2f0`, as recorded in [Matchmaking and Lobby](../Build%20Guides/Gameplay/Matchmaking%20and%20Lobby.md). Unresolved dwords and the unused byte are kept raw. |
| 141/6 cancellation carries a 64-bit request id. 141/14 reads a session-key pair and two count-prefixed u32 arrays. 141/15 reads one boolean byte. | `00a7fc60` confirmed constructor; `00a9d590` reads two `ReaderReadArrayOfU32` arrays through `008dab40`; `00a9eec0` checks one byte after the header. The two stats arrays are called `values_a`/`values_b`; UI usage associates them with pending counts and average wait, while array alignment is inferred. |
| Family 175 uses a 14-byte header: u16 family, then three i32 values. 175/5 reads a u32 count followed by 8-byte GUID entries. 175/9 reads a u32 member id and u8 state; 175/17 reads a u8 decision and u32 reason. | Shared header reader `009c1e00` likely; 175/5 list reader `00c4a810` likely, byte-checked here; 175/9 `00c4b9e0` likely; 175/17 `00c4acd0`/`00c4adb0` cover related server results and `00c4d160` is the client invite response reader (likely). Header-only senders 175/14, /15 and /16 are confirmed at `00c49ba0`, `00c49c60`, `00c49d20`. |
| 175/2 and /3 carry a GUID, length-prefixed bytes, a boolean and a u32 result code. | `00c4acd0` and `00c4adb0` likely. The meaning of the result code is unresolved; the codec preserves it without mapping it to text. |
| 39/16 is the shared minigame offer packet; a run ledger allocates its id and `offer` returns its bytes. | [Minigame Framework](../Build%20Guides/Gameplay/Minigame%20Framework.md) and `src/minigame.rs` (`MinigameRuns::offer`). This verifies client packet shape, not a historical matchmaking handoff. |

The capture-backed byte gate uses only the two preserved kart/racing 141 frames above. The lab also contains derby 141/1 and 141/2 frames with the same shape. Soccer captures found in the packet directory are family 76 gameplay frames, not matchmaking/lobby family 141 or 175, so this codec does not claim soccer lobby capture coverage. No raw frame or player text is committed.

An earlier [Kart Finish Line Binding](Kart%20Finish%20Line%20Binding.md) note describes an extra u32 after 141/1. The actual 19-byte capture and `00a7fb90` layout show only the 8-byte key after the four-byte header. The extra dword belongs to the captured 141/2 response prefix/first row boundary, not 141/1; the codec follows the bytes.

## Reconstructed policy (R)

The client does not document queue formation, group fill order, ready timeout, requeue, countdown, or the queue-to-match transition. The service labels these rules as R in code and uses deterministic caller-supplied clock and RNG.

- Entries reserve every member atomically. Parties are not split to fit a match; a configured group must fit at least one team. A session already reserved in any queue, ready check or match cannot join another queue. Repeated joins by a currently reserved same-queue entry are inert.
- Formation consumes a FIFO prefix of whole parties. A later party is not skipped when it does not fit the remaining player/team capacity. Team capacities are optional queue configuration and whole parties go to the first team with enough room. Empty capacities count each party as one team.
- When the configured player/queue delay threshold is met, the service opens one ready check for that queue and sends 175/14 to each selected session. Each changed ready bit emits 175/9 to all selected sessions; the service uses formation-order indices as the 175 member ids (R), because the matchmaking-to-launch member-id mapping is not recovered. If the timeout expires, only ready, still-connected players are requeued, at the front as solo entries; unready players are released. A leave/disconnect cancels the active ready check, requeues its ready survivors, and releases unready survivors.
- When all selected players are ready, the injected countdown elapses. The service makes one `MinigameRuns` ledger for the match, calls `offer` once, and sends 175/15, 175/5 with the selected GUID roster, and the same 39/16 bytes to every active session. A disconnect after launch removes that session; the remaining match stays alive until all members leave or the integration calls `close_match` after the activity's terminal processing.
- On a queued-party leave/disconnect, only that member is removed; remaining group members stay together. This is a policy choice, not recovered behavior.
- The 175 header's last two words are preserved as generic integers. The service currently uses its launch id and queue id in those positions; this association is R and must be checked with the native client before relying on it.

## Codec boundary and open evidence

`Packet141` and `GroupLaunchPacket` preserve unrecognized bodies byte-for-byte. Typed 141 codecs cover 141/1, /2, /6, /14 and /15. Typed 175 codecs cover the exact known shapes for /2, /3, /5, /9, /14-/17 and header-only operations. Queue records, invite/result payloads and roster ids do not receive invented meanings.

Still unverified or unsupported as typed fields:

- 141/3 join-request writer, 141/4 and /5 request-record responses, 141/9 invitation, 141/10 reply writer, 141/12 queue selection, 141/13 stats request, and 141/18 system-removal meanings. `00aa16f0` shows request-record scalar fields/list/info blocks, but `00aa0550` and `00a24750` show a nested participant record whose flags and strings need further byte-level recovery. The service therefore does not emit 141/4 on join or fabricate a 141/5 body on leave.
- 175/1 invite details contains an optional roster and extra record; it is not emitted by this queue implementation. 175/2-/3 text and result-code semantics are not mapped. 175/6 cancel-body reason is not emitted here; timeout/abort uses the exact header-only 175/16 event.
- No native capture demonstrates a successful queue-to-match flow, ready policy, numerical result codes, or how the original service chose between 141 invitations and family-175 launch events. The service's 175/14,/15,/5 + 39/16 path is a reproducible R proposal, not original server behavior.
- One `MinigameRuns` object currently belongs to each match, while existing save/gateway ownership is per character. The integration must define how the match ledger and identical instance id are represented in each player's persistence and how each client's lifecycle controls feed shared activity state. Do not persist only one player's ledger for a multiplayer match.

## Gateway wiring needed

The integration builder owns this work; no shared module was changed for W52.

1. Register the module with `pub mod matchmaking;` in `src/lib.rs` after lead review.
2. Decode tunneled family 141 packets with `Packet141`; route 141/1, /3, /6, /10 and /13 through the exact request codecs or opaque captured bodies. Resolve the sending `SessionId` to its live character GUID through `PlayerRegistry`/runner session state.
3. For group entry, consult `SocialService::group_of`, `group_leader` and `group_members`; resolve every member GUID to a live session, verify the requester is the leader and that the full group is present, then pass the complete `QueueEntry`. The service intentionally does not own or clone `SocialService`.
4. Route each `SessionOutbound.recipient` through that session's `ReliableSender` at the W35 runner synchronization boundary. Persist/lifecycle-stage any per-character snapshots before sending; the match ledger's multiplayer persistence contract remains open.
5. Add the captured 141/4 join acknowledgement/queue record only after the unknown member record writer is recovered. Do not interpret 141/4 empty outbound as a complete native queue UI integration.

## Native two-client test plan

Lead-owned; not run by this worker. Use two distinct authenticated accounts and an activity with a known 141/2 queue row. Capture both clients' 141/1, 141/2, 141/3, 141/4, 141/6 and any 141/9-/10 or 175 messages; compare exact tunnel-inner bytes and connection keys. Verify whole-group join, duplicate join, leave, disconnect during ready, timeout requeue, launch roster and 39/16 instance id independently for both recipients. Check each client receives the same roster/instance, then verify both enter the activity and that the chosen persistence model survives reconnect and terminal close. This gate has not been run.

## Checks run

From `prototype/osfr-tools`, with `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W52`:

- `rustfmt.exe --edition 2024 src/matchmaking.rs tests/matchmaking.rs` — passed; only W52-owned Rust files were formatted.
- `..\..\scripts\cargo.ps1 'test' '--test' 'matchmaking'` — passed: 11 passed, 0 failed, 1 ignored. The ordinary run reports the pre-existing `housing::Lot::instance_index` dead-code warning.
- The opt-in capture command used `OSFR_W52_KART_141_1` and `OSFR_W52_KART_141_2` set to the raw files under `lab/reports/original-minigame-packets`, then `..\..\scripts\cargo.ps1 'test' '--test' 'matchmaking' '--' '--ignored'` — passed: 1 passed. It decoded and re-encoded both complete tunnel frames byte-for-byte. An initial unquoted wrapper invocation passed `--ignored` to Cargo in the wrong position and was rejected; the quoted command above is the successful form.
- No full crate suite, native client, or two-client session was run. The test imports `matchmaking.rs` with `#[path]`; `src/lib.rs` remains unchanged.
