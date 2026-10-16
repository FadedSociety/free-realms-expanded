# Kart local racer and gate binding

Read-only follow-on, October 9, 2026 (UTC). Two integration gaps left open by
[[Research/Kart Vehicle Configuration and Input]]: (1) the local-racer ownership semantics of the
`33/11` tail, and (2) the fact that `33/7` gate hits carry a position rather than a gate id. No
production source, content, save, UI, release or process change; all raw bytes stay in the lab.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | consumer binding, gate-hit analysis, authority requirement |
| `lab/reports/kart-local-gate-binding-20261009T162829Z/proof.json` | pinned proof, sha256 `9856405598bb4c02561d675b2c7b912b95a80c59da4c7dc42e111b034d11f60d` (includes the full 31-hit sequence) |
| `lab/reports/kart-local-gate-binding-20261009T162829Z/check-kart-local-gate-binding.py` | checker, sha256 `ced568e68a97a2664c27b9dc2e06c2ad4af0a2d0caffaacb82ac73e957014c60` |
| `lab/reports/kart-local-gate-binding-20261009T162829Z/check-run.log` | run output, sha256 `5926007a3610c353594ababfed7311d02772ce7ebaa4505e5f06b38aab713ce3` |
| `notes/Research/Kart Vehicle Configuration and Input.md` | corrected in place (see section 3), sha256 `d86b8f3b8c423055cd9728fa3812d88b4cb6b18bdc5361ec67b2d06bf727feff` |

## 1. Local-racer ownership: the consumer, not the tail byte

`FUN_00ba74d0` case `0xb` (subtype 11) is the consumer:

1. the packet's owner guid is looked up with `FUN_00b9e0c0(guid)`;
2. when no racer with that guid exists, a proxied vehicle is created (`FUN_00b9d720` /
   `FUN_00c637e0`) and its speed field is seeded from `processor+0x2a0`;
3. only under a **stack flag** does the client mark it local:

```
if ((char)pppuStack_11c8 != '\0') {
    *(int *)(param_1 + 0x2cc) = iVar9;     // local racer
    *(byte *)(param_1 + 0x2c8) = bStack_64; // second value
}
```

That flag is **not** the packet's trailing byte. It is a stack local that the same case block sets
to 1 when `FUN_00705c30(...)` returns true (`pppuStack_11c8 = CONCAT31(…,1)`), and the whole case
block never reads the parser's tail fields `+0xe4c` / `+0xe18` (0 occurrences of either offset in the
10,240-character case body). The earlier note's "flag byte / slot" naming for the two tail bytes was
therefore an over-read; they are parsed but **unmapped**, and locality is decided by the client's own
guid/predicate logic.

Capture correlation (supporting, not proof of the byte's meaning): the first `33/11` frame — the only
one with tail byte 1 — is the frame whose owner guid equals the local player, and the other five
carry other racers' guids with tail byte 0. The local id is established by `33/9`'s first 8 bytes,
`0x4B52CC6A9573CA91` (5427625259080862353), which is also the guid on every `33/7`, `33/17` and the
local `33/14` rows.

Server guidance: keep sending tail byte 1 for the local player's own vehicle and 0 for others (as
captured) because the client's consumer also uses the surrounding creation path, but do not treat
the byte as the authority for ownership; the guid is what the client binds.

## 2. `33/7` gate hits carry a position, not a gate id

All **31** captured gate hits are 24-byte payloads `u64 guid | 3 x f32 position | f32 1.0`, and every
one belongs to the same local guid. The positions trace the circuit over 51.6 s, from
`(762.8, 80.8, 502.1)` to `(715.3, 76.1, 358.9)` (full sequence in `proof.json`). No gate id, gate
index or gate volume appears anywhere in the payload; the packet has no client parse case at all
(it is send-only), and the recovered `33/34` config carries a start transform, not gates.

**Original gate ids and gate geometry are unrecovered.** What can be built is a bounded
reconstruction:

* treat each accepted hit as an ordered waypoint sample (dedupe by radius, order by server time);
* expect the racer to approach the next waypoint in order, with a distance window and a time window
  derived from the previous accepted sample;
* keep the waypoint list per track as *reconstructed content*, explicitly labelled, never as
  "original gates".

## 3. Authority requirement (correction to the prior note)

The gate-hit position is **client-reported and must not be trusted directly**. A server-side gate
check needs, at minimum:

1. an authenticated sender: the guid in the packet must be bound to the connected member, and only
   that member's own guid may appear;
2. a server clock: hits are accepted in monotonic server time, not in client time;
3. movement corroboration: the displacement between consecutive accepted samples must be reachable
   at the kart's plausible speed (and the first sample ideally cross-checked against the spawn
   transform from `33/34`/`33/11`);
4. rejection on failure (teleport, back-tracking, duplicate, or out-of-order hits) rather than
   silently accepting the client's word.

[[Research/Kart Vehicle Configuration and Input]] was updated in place (sha256 as above): its gate
sentence now says the position is **not** directly trustable input and lists the required
corroboration, and its `33/11` tail wording is corrected to "parsed but **unmapped**" (capture table
columns renamed `Tail +0xe4c` / `Tail +0xe18`).

## Verification

```
& '<lab>/tools/wiki-env/Scripts/python.exe' check-kart-local-gate-binding.py
{"case11_tail_refs": 0, "gate_frames": 31, "local_guid": 5427625259080862353,
 "proof_sha256": "9856405598bb4c02561d675b2c7b912b95a80c59da4c7dc42e111b034d11f60d"}
```

The checker re-reads the case `0xb` block from the pinned Ghidra export (guid lookup, predicate-set
flag, `processor+0x2cc`/`+0x2c8` assignment, zero tail-field references), re-decodes all 31 gate-hit
frames from the timestamped index, asserts the single local guid and the first/last positions, and
confirms the flag=1 `33/11` owner is that guid; outputs use `write_exact` semantics.

Cross-links: [[Research/Kart Vehicle Configuration and Input]], [[Research/Kart Race Finish Protocol]],
[[Research/Kart Lobby Definition Framing]].
