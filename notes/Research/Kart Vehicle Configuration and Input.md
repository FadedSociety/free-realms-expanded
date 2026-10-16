# Kart vehicle configuration and input binding

Read-only recovery, October 9, 2026 (UTC). Scope: the native kart **configuration** packets
(`33/11` create-proxied-vehicle, `33/34` set-race-config) and the native **input** packets
(`33/6` server-data, `33/7` gate hit, `33/17` pickup hit request) — constructors, parsers, consumers,
field offsets and the minimum that a legitimate local character needs to spawn and drive. The lobby
framing and the finish/results binding are reused from
[[Research/Kart Lobby Definition Framing]] and [[Research/Kart Race Finish Protocol]] and are not
re-derived. No production source, content, save, UI, release or process change; raw bytes stay in the
lab. No retail balance, reward or scene-parser behaviour is claimed.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | bindings, layouts, captured values, minimal spawn, unmapped |
| `lab/reports/kart-vehicle-config-input-20261009T162349Z/proof.json` | pinned proof, sha256 `f4b85c30bca8d2230fc343d59cf5be57b22dfe76bdae9d31267eca915744886a` |
| `lab/reports/kart-vehicle-config-input-20261009T162349Z/trace-kart-vehicle-config-input.py` | checker (re-runnable, `write_exact`) |
| `lab/reports/kart-vehicle-config-input-20261009T162349Z/checker-run.log` | run output |

## Dispatcher bindings

`FUN_00ba74d0` (outer 33 family) constructs and parses each subtype:

| Subtype | Class | Ctor | Parser |
| --- | --- | --- | --- |
| 11 | VehicleRacePacketCreateProxiedVehicleRace | `00b9c910` | `00ba7460` → `00ba7370` |
| 34 | VehicleRacePacketSetRaceConfig | `00b9a1e0` | `00ba3670` → `00ba0680` |
| 5 | VehicleRacePacketUpdateVehicleRaceClientData | `00b9be30` | `00ba1070` |
| 6 | VehicleRacePacketUpdateVehicleRaceServerData | `00b9bfb0` | sent only (no client parse case) |
| 7 | VehicleRacePacketGateHit | inline vftable in `00ba48a0` | sent only |
| 17 | VehicleRacePacketPickupHitRequest | inline vftable in `00ba48a0` | sent only |

All of them start with `FUN_00b9fab0`, which reads the two header words (`u16` opcode, `u16` subtype).

## `33/11` create proxied vehicle (configuration)

`FUN_00ba7370` reads, in order:

```
base header (op/sub)                     FUN_00b9fab0
vehicle definition                       FUN_009d7b50   (variable length)
u64 owner guid                           packet +0x10 / +0x14
4 x 12-byte vectors                      FUN_009d4230  (3 dwords each)
u8 tail byte (unmapped)                  packet +0xe4c
u8 tail byte (unmapped)                  packet +0xe18
```

So the **last 58 payload bytes are fixed**: `u64 owner | vector0 | vector1 | vector2 | position |
tail +0xe4c | tail +0xe18`. The vehicle-definition block is everything before it. All six captured frames confirm
this: the owner guid is always one of the eight racer guids from `33/14`, the first three vectors are
the identity basis, the fourth is the spawn position, and the trailing bytes are `(0 or 1, 0..7)`.
Both trailing bytes are parsed but **unmapped**: the native consumer picks locality from the packet
guid plus a predicate-driven stack flag, not from these bytes
([[Research/Kart Local Racer and Gate Binding]]).

| Frame | Payload | Definition | Owner in `33/14` guids | Position | Tail `+0xe4c` | Tail `+0xe18` |
| --- | --- | --- | --- | --- | --- | --- |
| `racing-13266-…-sub11` | 6166 | 6108 | yes | 795.0, 86.2, 365.0 | 1 | 0 |
| `racing-13273-…-sub11` | 5807 | 5749 | yes | 785.0, 86.2, 365.0 | 0 | 2 |
| `racing-13277-…-sub11` | 5793 | 5735 | yes | 800.0, 86.2, 390.0 | 0 | 3 |
| `racing-13282-…-sub11` | 5823 | 5765 | yes | 795.0, 86.2, 415.0 | 0 | 5 |
| `racing-13286-…-sub11` | 5792 | 5734 | yes | 805.0, 86.2, 415.0 | 0 | 6 |
| `racing-13290-…-sub11` | 5803 | 5745 | yes | 785.0, 86.2, 415.0 | 0 | 7 |

The definition block itself (`FUN_009d7b50`, 481 pseudocode lines) begins with a length-prefixed asset
name (`kt_kart08_body_bbe.cdt` and the speedster/zipster variants were already recorded) followed by
parts, animation names and further nested structures; its interior is **not** re-parsed here
(out of scope: "don't guess unknown blocks"). Its exact byte length is `payload - 58`, which the
table above pins per captured frame.

## `33/34` set race config (pre-race)

`FUN_00ba0680` reads exactly **84 payload bytes**:

```
i32 | i32 | f32 | 4 x 16-byte float4 (FUN_008e2410, 4 dwords each) | i32 | i32
```

The single captured frame decodes as `(3, 1, 1.0)`, then the identity basis in vectors 0-1, `(0,0,-1)`
in vector 2, and a **spawn transform** in vector 3 = `(716.0, 82.0, 423.0, 1.0)`, with the trailing
pair `(16638, 16634)`. It is the first kart packet after the client's race-entry subtype 9, so it is
the race's start transform/config and not a per-racer record.

## Input: `33/7` gate hit and `33/17` pickup hit request

Both are 24-byte payloads and share a layout:

```
u64 local character guid | 3 x f32 position | f32 1.0
```

The sender is `FUN_00ba48a0` (the input/broadcast builder): it writes header words `0x21` + `7`
(`uStack_1e8 = 7`) with `VehicleRacePacketGateHit::vftable`, and `0x21` + `0x11` with
`VehicleRacePacketPickupHitRequest::vftable`; the guid comes from the local racer
(`processor+0x2cc` → `+0xe60/+0xe64`) and the four dwords from the local vehicle state
(`FUN_00c54660() +0x30/+0x34/+0x38/+0x3c`). The capture holds **31** gate-hit frames and **33**
pickup-hit frames, all 24-byte payloads; the first frames carry `(762.762, 80.781, 502.118, 1.0)` and
`(645.162, 72.210, 532.522, 1.0)` respectively. These are the client's spatial gate/pickup claims:
they are **not** directly trustable server input. Validation requires an authenticated sender (the
guid must belong to the connected member), a server clock (accept in server time), and
speed/location corroboration against the previous accepted sample, with rejection on failure
([[Research/Kart Local Racer and Gate Binding]]).

## Input: `33/6` server data (bulk)

Class `VehicleRacePacketUpdateVehicleRaceServerData` (`00b9bfb0`), emitted from `FUN_00ba48a0` at
`00ba58fa`. The capture holds **4,679** frames, every payload 85 bytes. The sender copies field
groups from the local vehicle state in a 3-float pattern, but the exact field boundaries are **not
recovered**; the count and length are pinned, the semantics are not.

## Local guid, slot and profile 48

* **Local guid:** the racer object at `processor+0x2cc` (the local racer in the list at `+0x298`),
  guid at `+0xe60`/`+0xe64`; every input packet above and the `33/11` owner field use it.
* **Slot (observed):** the `33/11` trailing byte `0xe18` is `0..7` and matches the slot values
  carried by `33/14` (`payload+0x08`) on capture; the correlation is observed, not a recovered
  consumer.
* **Profile 48 (Kart Driver):** no job/profile id is present on the recovered race wire. The race
  `41/114` details carry no 48, and the pinned executable export contains no reference to the Kart
  Driver name id 2917 or badge 7494. The link between the race and profile 48 is data-side (lobby /
  job catalogue), not a packet field.

## Minimum legitimate vehicle spawn

To spawn the original controller for the **existing** character without replaying captured actor
guids:

1. Send `33/34` once (84 bytes) with a start transform; the captured `(716, 82, 423, 1.0)` and the
   basis vectors are a working template, and the trailing pair is carried verbatim.
2. For each racer send `33/11` with **that racer's real guid** (`+0xe60/+0xe64`) in the owner slot,
   the identity basis, a spawn position, and the two captured trailing bytes (parsed but unmapped).
   The vehicle-definition block can be taken verbatim from a captured frame for the desired kart;
   nothing in the fixed tail depends on the captured actor guids.
3. Drive the input side from the client: `33/7`/`33/17` arrive with the local guid and position, and
   `33/6` carries the bulk state. A server-side gate check may **not** use the `33/7` position
   directly: the position is client-reported and must be corroborated (authenticated sender, server
   clock, and speed/location consistency against the previous accepted sample) before acceptance
   ([[Research/Kart Local Racer and Gate Binding]]).

## Unmapped (explicit)

* Vehicle-definition interior (`FUN_009d7b50`): asset name and parts are known, the full block layout
  is not; the note treats it as an opaque, size-pinned blob.
* `33/6` field boundaries inside its 85-byte payload.
* The `33/34` trailing `(16638, 16634)` meaning.
* The `33/11` two trailing bytes (`0xe4c` = 1 only on the first captured frame, `0xe18` = 0..7);
  parsed but never read by the visible consumer case.

## Verification

```
& '<lab>/tools/wiki-env/Scripts/python.exe' trace-kart-vehicle-config-input.py
{"bulk_frames": 4679, "config_bytes": 84, "create_frames": 6,
 "input_frames": {"7": 31, "17": 33},
 "proof_sha256": "f4b85c30bca8d2230fc343d59cf5be57b22dfe76bdae9d31267eca915744886a"}
```

The checker re-derives the dispatcher ctor/parser bindings and each parser's field order from the
pinned Ghidra export, decodes all six `33/11`, the single `33/34`, all `33/7`/`33/17` and all `33/6`
captured frames, and asserts the sizes, the owner-guid membership, the basis/position vectors, the
slot range and the frame counts before writing `proof.json` with `write_exact` semantics.

Cross-links: [[Research/Kart Lobby Definition Framing]], [[Research/Kart Race Finish Protocol]],
[[Research/Go Kart Native Integration Readiness]].
