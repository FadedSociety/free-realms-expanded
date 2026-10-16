# Kart Entry Encounter Binding

Read-only research + executed probe, 2026-10-09 (UTC). Scope: the kart entry
encounter/scene handshake — the minimal client Ready map, the render-load
gate, the 41/114 + 41/2 contract and the 102/141 lobby selection pair — with
known offsets and guid rebinding for the local identity guid `1`. No Rust
source, gateway, content, save, UI or release change; no client launched; no
packet sent. Kart controls/finish and the vehicle-GUID review stay with their
owners.

Owned artifacts: this note and
`lab/reports/kart-encounter-binding-20261009/` (`probe-kart-entry-encounter.py`
`df679275…`, `proof.json` `e27395bb…`, `timeline-window.json`,
`op13-client-is-ready.bin`, `op10-client-finished-loading.bin`,
`op31-kart-scene.bin`, `rebound-41114-state4.bin`,
`rebound-412-playerenter.bin`, `run.log`, `final-hashes.txt`, `receipt.json`).

## Answer first (what the lead needs now)

* The native client's two readiness messages carry **no payload**:
  `op13 PacketClientIsReady` (inner `13 00`, 2 bytes) and `op10
  PacketClientFinishedLoading` (inner `10 00`, 2 bytes). Community enum:
  `PacketCodes.cs:8` = 10 FinishedLoading, `:10` = 13 IsReady.
* In the racing capture the client sends **op13 first** (frame 15232, 14.908 s
  after the op31 kart scene) and **op10 9.322 s later** (frame 21375) — after
  the encounter entry, the 33/9 register and the server's `op14
  PacketZoneDoneSendingInitialData`. A server that waits for op10 before doing
  anything will hang the entry for ~9 s; a server that treats op13 as "world
  initialized" will stream world actors into a kart scene still loading.
* The render-load gate is real: between op31 (frame 723) and op13 (15232) the
  server sends only 124 frames — `op150/27` x116, `op38/12` x2, `op38/13`,
  `op41/13`, `op45/7`, `op47/4`, `op52/15`, `op53/15`. None of
  `op49/39/33/143/62/14/36/57/129/147` appears before op13; they all start
  after it.
* The encounter bundle is `41/114` state 4 + `41/2` at frame 15449, i.e.
  **~0.8 s after op13**, and the client's `33/9` register follows ~0.07 s
  later (frame 15480), then `op14` at frame 15496. The early `41/114` state 2
  is frame 135, before the scene.
* Current `src/gateway.rs` replays the captured **world** Ready block
  (templates 8..46, first opcode 67) on op13 when `ready == false`
  (`gateway.rs:587-599`), requires op10 for `Event::Loaded`
  (`gateway.rs:600-616`), and skips application packets while `!loaded`
  (`gateway.rs:665`); a scene zoning reply resets both flags
  (`gateway.rs:697-706`, `746-755`). For a kart scene that baseline world
  block is the wrong reply to op13 — use the kart-appropriate stream
  (below) instead, and do not gate the kart entry on op10.

## Offsets

All offsets are body-relative (body = bytes after the 4-byte inner header
`u16 op | u16 subtype`; file offset = body + 11). Inner length includes the
4-byte header.

`41/114 EncounterDetailsResponsePacket` (`FUN_00aa3430` -> `FUN_00aa32d0` ->
`FUN_008d6690` reads `i16, i16, u32, u32`):

| Off | Type | Value | Note |
| --- | --- | --- | --- |
| 0 | u32 | 917566 | encounter (`0x000E003E`) |
| 4 | u32 | 1008 | instance |
| 8 | u32 | 917566 | encounter repeated |
| 12 | u32 | 2 or 4 | state (early/late) |
| 16, 20 | u32 | 1, 1 | unnamed |
| 24 | u32 | `0xde2106fb` | shared racing/derby/soccer constant |
| 28 | u64 | session | **rebind here** |
| 36..40 | bytes | 00 01 00 00 00 | unnamed |
| 41 | u8 | 1 | unnamed |
| 42 | u8 | 0 state 2 / 1 state 4 | participant flag |
| 47 | f32[4] | spawn = op31 position, w = 1.0 | 719.60999, 72.0, 421.5 |

Body lengths: 407 (state 2, inner 411) and 387 (state 4, inner 391); the
20-byte difference is trailing data after the spawn block. Soccer uses
510/490 with the same prefix.

`41/2 EncounterPacketPlayerEnter` (17-byte body, inner 21):

| Off | Type | Value |
| --- | --- | --- |
| 0 | u32 | encounter |
| 4 | u32 | instance |
| 8 | u64 | 0 on first entry; captured session on the later send |
| 16 | u8 | 0 |

Captured sends: first entry frame 15449 (zeros) and later frame 21639
(session) in racing; derby 5078/13827 and soccer 3797/6300 repeat the pattern.

`102/1` (c2s) is empty (inner length 4). `102/2` (s2c) body starts
`u32 2378 | u32 6 | row0 = (1, 1, 1, 37, 3030)` (Kart Race) and is already
decoded by the registered `src/kart_lobby_native.rs`. `141/1` (c2s, the lobby
selection request) is the session u64 alone (inner 12); `141/2` (s2c) begins
with the session u64, then `u32 5 | u32 5 | …` (413-byte frame).

## Ordered entry (racing capture, seconds after the capture's t0)

| t | Frame | Dir | Packet | Role |
| --- | --- | --- | --- | --- |
| 8.122 | 715 | C→S | 41/108 | participant request entrance (session u64) |
| 8.439 | 723 | S→C | op31 | scene `sg_kt_track`, spawn 719.61/72.0/421.5, sky.xml, id 0, geometry 101 |
| 8.735+ | 897+ | S→C | op150/27 x116, op38, op41, op45, op47, op52, op53 | scene-local pre-ready data only |
| 23.347 | 15232 | C→S | **op13 Ready** | client finished loading/rendering (~14.9 s) |
| 23.347 | 15232 | C→S | 102/1 | lobby catalogue request (same frame, after op13) |
| 23.509+ | 15235+ | S→C | 34× op49/19, op38/5…63 | scene content unlocked |
| 23.553 | 15238 | S→C | 41/106 state 6 (op41/16) | encounter state |
| 23.557 | 15239 | S→C | 102/2 | catalogue (6 rows) |
| 24.311 | 15244 | S→C | op150/82112, op147/427, op35, op129 | initial data block |
| 24.312 | 15449 | S→C | **41/114 state 4 + 41/2 (zeros)** | encounter entry |
| 24.312 | 15450 | S→C | 41/114 state 4 + op39/23 + op35 | duplicate details relay |
| 24.406 | 15453+ | S→C | op57/10709 (POIs), op36, op143, op62 | scene content tail |
| 24.381 | 15480 | C→S | 33/9 | race register (after 41/2) |
| 24.406 | 15496 | S→C | **op14 ZoneDoneSendingInitialData** | server side of the gate |
| 32.669 | 21375 | C→S | **op10 FinishedLoading** | 9.3 s after op13 |

Early encounter state 2 (`41/114`, frame 135) and the state transitions
`41/106` 2→3→4→5 arrive before the register; `41/107 EncounterZoneIsReady` is
an encounter-readiness flag, not a scene message.

## Rebinding to guid 1 / a fresh session

Captured session bytes `91 ca 73 95 6a cc 52 4b` are the little-endian u64
`0x4b52cc6a9573ca91`; the local identity is guid `1`. Session field offsets
(body-relative): `41/114` 28, `41/2` 8, `41/108` 8, `141/1` 0, `141/2` 0,
`33/9` 0. Encounter/instance (`917566`/`1008`) are content: `41/114` 0/4/8,
`41/2` 0/4, `41/106` 0/4. Rebinding the captured `41/114` state 4 and `41/2`
to guid 1 changes exactly the 8 session bytes each (probe asserts the diff
counts and writes `rebound-*.bin`). The op31 `id` field is 0 in every captured
frame (kart/soccer); a local scene id may be content there, but no native
`supports_location` literal exists in the pseudocode — that term is the
launcher's, not a recovered client symbol.

## How this is proven

`probe-kart-entry-encounter.py` reassembles the pinned racing pcapng with the
same transport/CRC model as `lab/reports/analyze-original-minigames.py` and
asserts: 25,259 application frames with 2 transport errors, and a frame-hash
set equal to the existing `racing-application-index.json`; op13/op10 inner
length 2 with empty payloads; op13 < op10 and op31 < op13 < op10; only the
listed ops before op13; op49/41-391/41-21/op14 present after op13 and before
op10; the full 41/114 prefix map on every captured details frame (states 2 and
4, 411/391); 41/2 first/later session rule; 102/1 empty, 102/2 row0, 141/1
session u64, 141/2 session prefix; and the byte-counted rebinding.

## Limits

* Read-only: no client run, no packet sent, no native playability claim.
* `op13`/`op10` semantics come from the community enum names plus capture
  position; the client's internal state machine around them is not traced.
* Field meanings beyond the offsets above (including `0xde2106fb`, the `41`
  byte and the trailing details block) stay unnamed.
* The gateway's behaviour is quoted from a read of `src/gateway.rs`; nothing
  in this note changes it.
