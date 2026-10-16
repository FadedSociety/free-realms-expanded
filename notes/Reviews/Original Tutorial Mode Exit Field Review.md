# Original tutorial mode exit field review

Read-only review, October 9, 2026 (UTC). Scope: pin the exact `41/2 EncounterPacketPlayerEnter`
field that feeds `player+0x780` through `008a9200` (caller `00aa38b8`), the manager teardown at
`00aa38ce`, and the `41/114` field at `00aa3c7f`; map each byte offset and type to the pinned
community writer and to the current `src/encounter.rs` writer; and qualify the accepted checkpoint's
writer list. No game process, save, UI, content or production source was touched. The accepted
[[Research/Original Tutorial Mode Exit]] checkpoint and the microtutorial 45/7 findings are reused,
not re-investigated.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | field mapping, bounds qualification, recommendation |
| `lab/reports/original-tutorial-mode-exit-field-review-20261009T141632Z/proof-v2.json` | pinned proof, sha256 `59fb8f1a83c533ed8163fca3fa17125aeae92824ed19d6216bebd6cf4d66d9ad` |
| `lab/reports/original-tutorial-mode-exit-field-review-20261009T141632Z/proof.json` | first revision, superseded (inline-writer filter and instruction alignment were too narrow) |
| `lab/reports/original-tutorial-mode-exit-field-review-20261009T141632Z/trace-original-tutorial-field-mapping.py` | reproducer |
| `lab/reports/original-tutorial-mode-exit-field-review-20261009T141632Z/trace-run.log` | run output |

Pinned inputs: original PE sha256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`;
Ghidra export `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`; captured retail
`41/2` frame `demoderby-23087-flow0-frame5078-op41-sub2.bin`
`ecf5eda4a3dcd73ea1ab492ed676ad235880d4f32d9acdb5b80e31f68603e7bf`; pinned community
`EncounterPacketPlayerEnter.cs` `30d12f07559625f8241bcf38287e19a14da7d71b36dedc26afdc473ba3cf41e4`
and `EncounterDetailsResponsePacket.cs`
`14547ac621d94c351dfc1f592e7ad948a7d1ae1116e1f5d2e79c05b7a2a523aa`; current
`prototype/osfr-tools/src/encounter.rs`
`68cfc114b65ae9f8648ef94bf3a566789f772a988daba87c2c02c2cc2de4e65e`.

## 41/2 field that feeds player+0x780

ORIGINAL. `FUN_008a9200` stores its byte argument at `player+0x780`
(`008a9204 mov byte ptr [ecx + 0x780], al`). Its three direct call sites are `00aa38b8`, `00aa3c7f`
and `00c17005`.

The `41/2` site is inside `FUN_00aa36c0`, sub-opcode case 2 (jump table `0xaa44e8` index 0 ->
`0xaa388b`). The packet object is built at `esp+0x40` (`lea ecx, [esp + 0x40]` -> `FUN_00a99e10`,
the `EncounterPacketPlayerEnter` constructor), and the value pushed into the setter is read from
`esp+0x60` (`00aa38b0 mov eax, dword ptr [esp + 0x60]`). `0x60 - 0x40 = 0x20`, so the argument is
**object+0x20**.

The reader chain is `FUN_00a9f760` -> `FUN_00a9ccc0` -> `FUN_008d6690` (shared 12-byte encounter
header: word, word, dword, dword -> object `+4`, `+8`, `+0xc`, `+0x10`), then `FUN_00a9ccc0` reads
**8 bytes** into object `+0x18`/`+0x1c` and **1 byte** into object `+0x20`, normalised to 0/1 with
`cmp byte` / `setne` (`00a9cd1c`-`00a9cd21`). The object constructor `FUN_00a99e10` presets those
slots (`+0x18`, `+0x1c`, byte `+0x20 = 0`).

Wire mapping for a 21-byte `41/2` body:

| Body offset | Field | Client object |
| --- | --- | --- |
| 0..1 | `u16` opcode 41 | object+0x04 |
| 2..3 | `u16` sub-opcode 2 | object+0x08 |
| 4..7 | `i32` encounter id | object+0x0c |
| 8..11 | `i32` instance id | object+0x10 |
| 12..19 | `u64` | object+0x18 / +0x1c |
| 20 | trailing byte | object+0x20 -> `player+0x780` |

That trailing byte is the pinned community writer's `Unknown` field:
`EncounterPacketPlayerEnter` serializes `OpCode`, `SubOpCode`, `EncounterId`, `InstanceId`,
`PlayerGuid`, `Unknown` in exactly that order, so body byte 20 is `Unknown`. Type: one boolean byte
on the wire; the client normalises any non-zero to 1.

The captured retail frame agrees on the shape: 7-byte tunnel envelope, then a 21-byte body with
opcode 41, sub-opcode 2, encounter id `1310785`, instance id `1008`, guid `0`, trailing byte `0`.
That capture is a demolition-derby entry, not the cave; it proves the position of the byte in retail
traffic, not the cave's own value.

## 41/114 is a different field

ORIGINAL. The `41/114` site is inside the same dispatcher (`0xaa3c35`; `FUN_00aa2e90` constructs the
details object at `esp+0x324`). The value pushed into `008a9200` is read from `esp+0x372`
(`00aa3c74`), i.e. **object+0x4e**. The reader is `FUN_00aa3430` -> `FUN_00aa32d0`, which runs the
shared 12-byte header reader and then reads a nested record through `FUN_00a29120` at `object+0x18`.
That nested reader stores three consecutive boolean bytes at sub-record `+0x34`, `+0x35`, `+0x36`
(`00a291f3`, `00a29219`, `00a2923f`), and `0x18 + 0x36 = 0x4e`, so the flag input is the **third
boolean of that triple**.

The pinned community `EncounterDetailsResponsePacket` writes exactly three bytes in that region -
`Unknown5 = true`, `Unknown6 = false`, `Tutorial` - so the client's `+0x4e` byte is the community
`Tutorial` field. The current `src/encounter.rs` writes `[1, 0, 1]` there, i.e. `Tutorial = 1`. The
same `41/114` branch additionally writes `player+0x704` (from object+0x48), `player+0x958`,
`player+0x781` and `player+0x782` (from comparisons of object+0x44 with 6/8/9), so it is an
encounter-state write rather than a dedicated tutorial exit.

Answer to "is `41/114` naturally the same field": **no**. Both bytes reach `player+0x780`, but
`41/2` uses the packet's trailing byte after the `u64`, while `41/114` uses the third byte of the
details common triple and carries the encounter profile writes with it.

Open mapping question in the community model (recorded, not repaired): after the three booleans the
client reads two `i32`s (8 bytes) before `MiniGameInfo`, while the pinned community writer emits one
byte `Unknown8` plus one `i32 RespawnTime` (5 bytes). The client and the current Rust writer agree
with each other; the community model is the outlier. This does not move the `Tutorial` byte because
it sits before that divergence.

## Manager teardown

ORIGINAL. In the `41/2` success branch the client runs, in order: `player+0x780 = object+0x20`
(`00aa38b8`), `009b9500(player+0xf8)` (`00aa38c6`), then `FUN_008a8a30(player)` (`00aa38ce`).
`FUN_008a8a30` loads `player+0x77c` and tail-jumps to `FUN_00a46c40` only when it is non-zero.
`FUN_00a46c40` flips `manager+8 = 1` and unregisters through `00ae3750`; it is idempotent and it
**never writes `player+0x77c`**.

Literal displacement scan of the mapped image finds three writers of `player+0x77c` on the player
object:

| Site | Instruction | Role |
| --- | --- | --- |
| `008a8a0b` | `mov dword ptr [esi + 0x77c], eax` | `FUN_008a89a0` zone true path: creates the manager only when the slot is 0 |
| `0093695c` | `mov dword ptr [esi + 0x77c], ebx` | owner teardown `FUN_009359e0`: loads the slot, calls `00a46bf0` (manager vftable reset), frees via `00d82463`, then nulls the slot |
| `00942396` | `mov dword ptr [esi + 0x77c], ebx` | `BaseClient` constructor cluster zeroing |

The scan also matches `0078b161 mov dword ptr [ebx + 0x77c], esi`, but that is a different class:
the enclosing `FUN_0078aa90` writes vftable-style pointers to `+0x770`/`+0x774` immediately before
it, so the displacement is a coincidental layout overlap, not the tutorial slot. Displacement scans
cannot separate object types on their own; this one was checked by hand.

Consequence: the `41/2` dispose unregisters the manager but leaves the slot set. A later tutorial
zone sets `player+0x780 = 1` again but cannot recreate the manager, because `FUN_008a89a0` only
constructs when the slot is 0; only the owner teardown `FUN_009359e0` nulls it. Any later tutorial
work that expects a fresh manager in the same client object must expect this.

## Reachability bounds (qualifying the "complete writer list")

The accepted checkpoint's writer list is precisely the **three direct `E8 rel32` call sites** of
`008a9200` (`00aa38b8`, `00aa3c7f`, `00c17005`). That is a bound on direct calls. It is not proof
that no other writer exists, and it is not proof that indirect or computed calls are absent. The
review found these additional literal writers of the same byte field on the player object:

| Site | Instruction | Context |
| --- | --- | --- |
| `008a89d1` | `mov byte ptr [esi + 0x780], 1` | `FUN_008a89a0` zone true path (callers `009472e9`, `00947cbe`) |
| `00936d71` | `mov byte ptr [esi + 0x780], 0` | runtime path in `FUN_00936d00`, conditional (clears unless `[esi+0x700] == 1` and `[esi+0x78c] == 7`); callers `0093891e`, `0094a4d2`, `0094b56a` |
| `0094239c` | `mov byte ptr [esi + 0x780], bl` | `BaseClient` constructor cluster zeroing, caller `00401068` only |

So the accurate statement is: "the only direct callers of the setter are these three, and one
further runtime path clears the byte inline." Additional bounds and their limits:

* `0x008a9200` appears nowhere in the mapped image as a 4-byte immediate or data word, so there is
  no `mov reg, imm32` / `push imm32` / pointer-table route to the setter. An address computed at
  runtime is not excluded.
* The displacement scan covers literal `disp32 == 0x780` operands with every nearby instruction
  alignment. A pointer-aliased write (`lea reg, [obj+0x780]` then a store through `reg`) is not
  attributed to this field, and same-displacement writes on other classes are indistinguishable
  without checking the enclosing function, so the inline list is a lower bound, not a complete set.
* `FUN_00aa36c0` has no direct callers; it is reached through the handler table around `0x0181884c`
  (referenced from `00aa06e2`/`00aa07dc`), i.e. the encounter dispatch itself is indirect.
* `FUN_008a8a30` (manager dispose) has exactly one direct caller (`00aa38ce`) and no immediate or
  data reference; indirect reachability is likewise not excluded, and the separate owner teardown
  at `0093695c` destroys the manager through `00a46bf0` + `00d82463` without using `008a8a30`.

## Writer compatibility

| Field | Client read | Community writer | Current Rust writer | Captured value |
| --- | --- | --- | --- | --- |
| `41/2` flag | body byte 20 -> object+0x20 -> `player+0x780` | `EncounterPacketPlayerEnter.Unknown`, last serialized byte | `encounter_header(2, id, instance)` + `u64 1` + `push(0)` -> byte 20 = 0 | demo-derby frame byte 20 = 0 |
| `41/114` flag | details common byte 3 of `Unknown5`/`Unknown6`/`Tutorial` -> object+0x4e -> `player+0x780` | `EncounterDetailsResponsePacket.Tutorial` | `b.extend([1, 0, 1])` -> `Tutorial = 1` | no capture |

The frozen current-writer vector in `tests/piece_060.rs`
(`290002009b01000041420f00010000004000000000`) decodes to the same 21-byte layout with trailing
byte 0, so the current writer and the original capture agree on the byte that drives the flag.

## Does the current real cave entry clear flag and manager?

Yes, with the caveats below. The gateway writes `41/2` only from
`src/encounter.rs::on_zone_ready` (`self.details(true, profile_id)` and the following state
packets), which runs from `gameplay.rs::initialize_world` while the cave encounter is **active**;
when no encounter is active the function returns world initialization only. That is the genuine
encounter of the reconstructed cave (`sg_npx_cavern_01`, dungeon 411, whose zone transition carries
`"tutorial": true`), not a fabricated encounter.

The writer emits a 21-byte body with trailing byte 0, which by the mapping above makes the client
write `player+0x780 = 0` and call the manager dispose on that real packet. `FUN_00a9f760` returns
success only when the reader consumed the whole buffer (or over-ran), so the trailing byte is
required: a truncated body would leave the branch unfired. No extra packet, Lua call, client patch,
memory write or fake encounter is needed for the exit.

Caveats. (1) `41/114` runs immediately before it in the same flow and writes `Tutorial = 1`, so the
flag is set then cleared within the entry sequence; the net result is off. (2) The manager exists
only if the zone packet set `tutorial` true; the cave's zone transition does. (3) The dispose leaves
`player+0x77c` set, so a later tutorial zone in the same client object cannot recreate the manager
until the owner teardown runs. (4) No retail capture of the cave's own `41/2` exists; the cave value
is inferred from the writer and from the demolition-derby frame, not observed.

## Recommendation

Keep the current real cave entry as the tutorial-mode exit: it already performs the original flag
clear plus manager dispose through the authentic `41/2` with trailing byte 0. Do not add a synthetic
`41/2`, a `107/5` Lua callback, a client patch or a memory write for this purpose. Two follow-ups
for the lead: reword the accepted checkpoint's "complete writer list" to "the three direct callers
of `008a9200`, plus one inline runtime clear at `00936d71`" (direct calls are not proof about other
writers), and record the `player+0x77c`-never-nulled-on-dispose behaviour as a known limitation for
re-entering tutorial mode in one session. The community `EncounterDetailsResponsePacket`
`Unknown8`/`RespawnTime` width should be re-checked against a capture before that model is trusted
for anything after the `Tutorial` byte.

## Verification

```
& '<lab>/tools/wiki-env/Scripts/python.exe' trace-original-tutorial-field-mapping.py
{"flag_direct_call_sites": ["00aa38b8", "00aa3c7f", "00c17005"],
 "flag_field": "packet body byte 20 (after the 12-byte encounter header and the u64); object+0x20; boolean byte",
 "flag_inline_writes": ["008a89d1 in 008a89a0", "008a9204 in 008a9200", "00936d71 in 00936d00", "0094239c in 00941ba0"],
 "manager_slot_inline_writes": ["0078b161", "008a8a0b", "0093695c", "00942396"],
 "proof_sha256": "59fb8f1a83c533ed8163fca3fa17125aeae92824ed19d6216bebd6cf4d66d9ad"}
```

The reproducer asserts every pinned source hash, the two call-site windows, the jump-table entry,
the reader chains and their offsets, the manager teardown sequence, the writer field orders, the
captured frame, the frozen test vector, and re-runs the direct-call, immediate and
alignment-tolerant displacement scans before writing `proof-v2.json` with `write_exact` semantics.

Cross-links: [[Research/Original Tutorial Mode Exit]],
[[Research/Original Opening Microtutorial Recovery]].
