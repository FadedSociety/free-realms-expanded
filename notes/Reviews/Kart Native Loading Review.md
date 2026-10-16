# Kart Native Loading Review

Read-only diagnosis of the first native kart entry stall, October 9 2026 (UTC).
Owned artifacts: this note and `lab/reports/kart-native-loading-review-20261009/`.
Nothing in the run directory, gateway source, content or client was modified;
no second run was performed and no long command was repeated.

Run: `lab/runs/client-baseline-20261009T214420Z` (manifest sha256
`92720c71…`), 1449 captured datagrams (1430 gateway + 19 login), CRC length
**3** (native; the archive model is 2), 1417 CRC-verified frames, complete
reassembly, 11 server `op6` 2-byte keepalives that carry no CRC.

## Root cause (two independent blockers, both evidenced)

1. **The kart track geometry is not served.** The client issued **951 of its
   1000 asset HTTP requests** for 16 distinct `sg_kt_track_*.gcnk` chunks
   (e.g. `/562/sg_kt_track_8_4.gcnk`, 66 requests alone) and **every one
   returned 404**; `asset_delivery_proofs` is empty and
   `lab/reports/kart-playable-20261009/content` contains only
   `kart_entry.json` / `kart_lobby.json`, no `sg_kt_track_*` chunk at all.
   The scene loader therefore never finishes streaming the track, so the
   client never sends the kart-scene Ready (`op13`) and cannot advance.
2. **The encounter entry is incomplete.** At t=136.781 the gateway sent
   `op31 sg_kt_track` plus **`41/114` state 2 only**. The archived pre-register
   sequence requires the later **`41/114` state 4** and the **`41/2`
   PlayerEnter** (plus the post-Ready scene stream); neither appears in the
   run. The client never sends `41/108` (participant request entrance) or
   `33/9` (race register), so the 33/* kart exchange never starts.

Both blockers are real and independent: the assets alone cannot make the
client render the track, and the missing 41/2 entry alone cannot make it
register.

## Observed ordering (t = seconds in the run)

| t | Dir | Packet |
| --- | --- | --- |
| 76.453 | C→S | `op13 PacketClientIsReady` + `102/1` (starting-area Ready) |
| 76.562 | S→C | `op14 ZoneDoneSendingInitialData` + world baseline (`35` x4, `62`, `41/132`, `41/133`, `38`) |
| 76.578 | C→S | `op10 PacketClientFinishedLoading` |
| 77.796 / 77.859 | C→S / S→C | `102/1` then `102/2` catalogue (2393 B, 6 rows) |
| 94.593 | C→S | `141/1` selection (19 B) — **no server `141/2` reply** |
| 136.765 | S→C | `op38` (394 B) |
| 136.781 | S→C | `op31 sg_kt_track` + `41/114` state 2 |
| 151.9 … 301.9 | S→C | `op6` 2-byte keepalives every ~15 s until the client was killed |

The run's client→server application set is
`13, 10, 102/1 x2, 105 x3, 109 x9, 125, 126 x15, 141/1, 15, 156, 168, 169,
193, 194 x7, 26 x19, 47/13, 66 x5, 88` — no `41/108`, no `33/*`. The
server→client set carries the world baseline plus `31` and one `41/114`.

## Live frames vs archived originals (byte level)

* `op31` live (87 B, sha `a8c716f5…`) differs from the archived
  `sg_kt_track` frame (`bace4cba…`) in exactly **4 bytes**: the scene `id`
  field is `0` in the archive and **`48000001`** live (the local zone id,
  matching the gateway's `character_saved`).
* `41/114` live state 2 (418 B, sha `c352f0a0…`) differs from the archived
  state-2 frame (`7fbc0afc…`, file
  `racing-12959-flow0-frame135-op41-sub114.bin`) in exactly **12 bytes**:
  instance `1008` → `48000001` (4 B) and session
  `91 ca 73 95 6a cc 52 4b` → guid `1` (8 B). Encounter `917566`, state 2,
  spawn `719.60999, 72.0, 421.5`, constant `0xde2106fb` and body length are
  unchanged. The gateway's scene/encounter injection is a faithful **rebound
  early pair**; it stops before the late pair.

## Missing native templates vs archived proof

The gateway reference (`lab/references/rust-gateway-reference/manifest.json`)
holds 8 login + 38 ready world templates only (opcodes
`41,2,166,43,144,35,193,12` then `67,57,35 x18,38,38,36,38,38,209,209,165,93,
14,38,38,87`). The archived kart scene
(`lab/reports/original-minigame-packets`, `kart-encounter-binding-20261009`)
carries, and this run never sends:

* encounter entry: `41/114` **state 4** (398 B), `41/2` PlayerEnter (28 B),
  `41/106` state transitions, `41/107`, `41/109`, `41/120`, `41/121`;
* scene stream: `34 x op49/19`, `op150` 82112 B, `op147/427`, `op129/5`,
  `op39/23`, `op36/44`, `op57/10709`, `op143/55` + `/56`, `op62/45`,
  `op35/16, /17, /24, /170, /334`;
* the whole `33/*` family (`34`, `11`, `5`, `14`, `1`, `3`, `4`, join `2`).

Client-side templates also absent: the `41/108` entrance request, the `33/9`
register and a second `op13` for the kart scene.

## Why the pending root fixes matter

* `op13` currently replays the **world** baseline Ready block for any scene
  (`src/gateway.rs:587-599`), and the reference templates are that world block
  — "Ready13-specific" means a kart scene must answer `op13` with the
  kart-scene stream instead.
* After scene zoning the gateway sets `ready = false; loaded = false`
  (`src/gateway.rs:697-706`, `746-755`) and then drops every client
  `Event::Application` while `!loaded` (`src/gateway.rs:665`). The archive
  shows the client authors `33/9` ~0.07 s after `41/2` and only sends `op10`
  **9.3 s later**, so "allow 33 before Loaded" is required for the entry to
  work even after the assets and 41/2 exist.

## Verification

`probe-kart-native-stall.py` (sha256 `b319dfdd…`, derived from the existing
`analyze-native-created-character.py` decode/reassembly) re-read every gateway
frame, verified 1417 CRCs at length 3, reassembled both directions
completely, and wrote `stall-proof.json` (`bb7c996e…`),
`live-applications.json` (`f3187b6c…`) and one `live-app-NNNN.bin` per
application. `run.log` (exit 0) holds the recorded run.

## Limits

* Read-only: no gateway/content/client change, no second run.
* The 11 `op6` 2-byte server frames carry no CRC and stay unverified.
* The `141/1` at 94.593 has no observed reply; whether the gateway answered it
  on another socket or dropped it is not established here.
