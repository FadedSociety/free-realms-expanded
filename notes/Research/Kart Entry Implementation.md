# Kart Native Race Entry Implementation

Implementation, 2026-10-09 (UTC). Scope: the bounded native kart race-entry slice - the client `33/9`
register exchange and its ordered server answer - implemented against the recovered contract. No
gateway/gameplay/lib/Cargo/content/save/UI edits; the lead owns registration and routing. No finish,
no payout, no physics, no derby.

Owned artifacts:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/kart_entry.rs` | typed entry handler, context, content, ordered replies |
| `prototype/osfr-tools/tests/kart_entry.rs` | synthetic vectors + ignored private capture gate |
| `lab/reports/kart-entry-repair-20261009/build-private-content.py` | optional private content package builder |
| `lab/reports/kart-entry-repair-20261009/race-entry-context.json` | captured context (lead seam) |
| `lab/reports/kart-entry-repair-20261009/race-entry-content.json` | captured content (lead seam) |
| `lab/reports/kart-entry-repair-20261009/*.log`, `receipt.json`, `final-hashes.txt` | run evidence |

## Verdict

The entry exchange now exists as code and is verified against the preserved capture. A `33/9`
register from the owned session produces, in order, `33/34`, one `33/11` per entry vehicle, `33/5`
and one `33/14` per racer; the client `33/2` join is answered with `33/3` and `33/1`; the later
recovered bundle contributes `33/1` (state 4) and `33/4` (npc count). Every one of those inner
frames re-encodes **byte-identically** to the captured frames, and the op31 scene frame re-encodes
byte-identically through the registered `zone_transition` codec with rotation `w = 0` preserved.

The exchange is off by default. While the gate is disabled nothing is parsed, nothing is answered
and no handler state moves; refusals, spoofed sessions and malformed frames never mutate state and
never touch economy.

## What the handler is

`src/kart_entry.rs` is one flat handler over three data types, reusing the registered codecs:

* `RaceEntryContext` - the owned entry identity: `session` (u64), `instance_id`, `encounter_id` and
  the typed `zone_transition::ZoneTransition` scene (scene name, spawn, rotation, sky, tail).
* `RaceEntryContent` - the recovered wire blocks: `RaceConfig` (the captured `33/34` body as its 21
  positional words), `entry_vehicles` (`33/11`: name, the `f32` that is `400.0` in all six captured
  frames, verbatim attribute block), `client_data` (verbatim 85-byte `33/5` body), `progress`
  (`33/14` payloads typed by `kart_progress_native::ProgressPayload`) and the recovered
  `player_count` / `npc_count` counter values.
* `KartEntryHandler` - gate, phase (`Waiting -> Started -> Joined -> Streaming -> Closed`), the open
  context, and one retired session GUID.

Existing codecs are reused rather than duplicated: `crate::kart_racing::VEHICLE_RACE_INNER_OPCODE`
for the family opcode, `crate::kart_progress_native` for the `33/14` layout (`encode_progress`,
`GAME_STATE_SUBTYPE`, `PROGRESS_SUBTYPE`) and `crate::zone_transition` for the op31 scene frame -
`open` even uses the zone codec as the scene validator, so a scene the codec would refuse cannot
open an entry.

Two deliberate non-decisions keep opaque captured state opaque instead of inventing semantics:

* the `33/34` body is carried as its 21 little-endian words with no field names beyond the wire
  order (the note's own field map leaves the leading counts, flags and the trailing triple
  unrecovered);
* the `33/11` attribute block and the `33/5` body are preserved verbatim. Only the recovered
  structural prefix of `33/11` (`u32 name length | name | f32`) is decoded.

## Lead route and dev seams

1. Register the module in `src/lib.rs` (lead-owned): `pub mod kart_entry;`. The test target includes
   the source with `#[path]` until then and uses real crate imports for everything else.
2. Load the content package through serde, exactly like the existing `kart_lobby.json` path:
   `serde_json::from_slice::<RaceEntryContent>(&bytes)` (`deny_unknown_fields`, bounded lengths).
   Optional split context: `serde_json::from_slice::<RaceEntryContext>`.
3. In `gameplay.rs::handle_in_world`, before the opening/quest/combat chain, mirror the lobby gate:

   ```rust
   if inner.starts_with(&[33, 0]) {
       match self.kart_entry.handle_inner(inner) {
           Ok(EntryOutcome::Replies(replies)) => return Ok(Outcome { replies, ..Outcome::default() }),
           Ok(EntryOutcome::Refused(_)) => return Ok(Outcome::default()),
           Err(error) => return Err(error.into()),
       }
   }
   ```

   The gateway gives `Gameplay` inner bytes and wraps every reply itself, so the handler emits inner
   frames only (`u16 33 | u16 subtype | payload`) and never a tunnel envelope.
4. Scene: `handler.scene_frame()` returns the complete op31 frame (envelope included) written by
   `zone_transition::encode`. Send it as-is on the world connection, or route
   `handler.scene()` (typed context) through the lead's zoning path. Do **not** send it through
   `encounter::zone_begin_packet`, which hard-codes rotation `w = 1.0`, and do not put it in the
   inner reply list (that would nest the tunnel envelope).
5. Open the entry when the player enters the speedway (`handler.open(context)`), enable the dev gate
   only from the launcher's explicit dev flag (`handler.set_gate(EntryGate::Enabled)`), and call
   `handler.on_race_ready()` when the server decides the racer stream starts. `close()` retires the
   session so a late register is reported as `StaleSession` rather than silently accepted.

## Bounds and refusals

Owned GUID/instance bounds: `session`, `instance_id` and `encounter_id` must be non-zero (zero is the
captured empty value inside `41/2`), every racer GUID in the content must be non-zero, distinct and
inside `MAX_RACERS`, every racer slot inside `0..MAX_RACERS`, and every register/join must match the
owned session GUID exactly.

Typed refusals (no replies, no state movement):

| Refusal | Trigger |
| --- | --- |
| `GateDisabled` | the default gate, checked before any parse |
| `NoOpenContext` | a register/join with no open entry and no matching retired session |
| `ForeignSession` | a session GUID the open entry does not own (spoofed/foreign) |
| `StaleSession` | a replay after the phase already consumed the request, or a retired session |
| `InvalidPhase` | a join before `Started`, or a race-ready before `Joined` |
| `UnsupportedSubtype` | client control subtypes (`33/6`, `33/7`, `33/17`, ...) stay inert |

Malformed input is a typed `EntryError` instead: short inner frames, a foreign inner opcode, and any
payload whose length is not exactly the recovered one. Content/context bounds are validated once at
`new`/`open`. `EntryError -> io::Error` is implemented so the gameplay seam can use `?`.

The module contains no grant, price, payout or economy call, no finish/results path and no reward
type; there is nothing for a refusal to leave half-done.

## Capture derivation detail (session byte order)

The contract note renders the session as the wire bytes `91ca73956acc524b` (its field map calls the
field `session_u64_le`). Read little-endian like every other field in this family - and like the
registered `kart_progress_native` reader - those bytes are the value `0x4b52cc6a9573ca91`. The
implementation and the private gate use the little-endian value; the private gate additionally
asserts the byte order so the two conventions cannot drift.

## Verification

```
# synthetic vectors (module behaviour, ordering, bounds, refusals, scene round-trip)
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test kart_entry
# 13 passed; 0 failed; 1 ignored

# reused codecs still green alongside the new target
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml \
    --test kart_entry --test kart_progress_native --test zone_transition --test kart_racing
# 13+9+13+9 passed; 2 ignored

# private capture gate (hash-pinned frames, byte-identical re-encode)
$env:OSFR_KART_ENTRY_CAPTURE='.../lab/reports/original-minigame-packets'
$env:OSFR_KART_ENTRY_CONTENT='.../lab/reports/kart-entry-repair-20261009/race-entry-content.json'
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test kart_entry '--' --ignored
# 1 passed; 0 failed

# private content package builder (read-only over the capture)
& lab/tools/universal-modder-env/Scripts/python.exe \
    lab/reports/kart-entry-repair-20261009/build-private-content.py
```

Test-first evidence: the target failed to compile before the module existed (`red.log`, exit 1:
`couldn't find file tests\..\src\kart_entry.rs`), then passed after implementation.

The private gate pins eleven captured frames by sha256, rebuilds the context and content from them
(encounter/instance read from the `41/114` body), and asserts the handler's replies equal the
captured inner frames byte for byte; it also loads the private content package and asserts it equals
the derived content. It is `#[ignore]`d so no captured bytes are needed in the default run and no
captured bytes are committed.

## Remaining gaps

* No finish, results or payout exists anywhere in the recovered capture; the `33` stream simply stops.
  `kart_racing::RaceSession` stays unbound and unpriced by default; nothing may grant from this slice.
* Client control subtypes (`33/6` server data, `33/7` gate hit, `33/17` pickup hit) are not handled
  here; the readiness note's plan keeps them observation-only until physics is recovered.
* The continuous `33/5`/`33/14` update stream is untouched: the later captured bundle repeats both
  with *different* payloads (byte-compare in `receipt.json`), so `on_race_ready` emits only the two
  recovered state/count transitions instead of re-authoring bulk blocks.
* Encounter delivery (`41/114` state 4 + `41/2`), the pre-register timing (scene ~16 s, player-enter
  ~0.07 s before the register in the capture) and the return scene (`op31 FabledRealms`) stay with
  the lead's encounter/zoning path; this module only exposes the seams.
* Field *semantics* remain unrecovered for the `33/9` words, most `33/34` words and the `33/11`
  attribute block. They are carried verbatim rather than named.
* No native client run was performed: the lead controls the shared window, and this slice's private
  gate is a capture re-encode, not a live session. Subtypes 12/13/19-33 remain uncaptured.

## Skills applied

RTK (normal commands first, condensed output), Ponytail (smallest working handler, reuse before
writing, one flat struct, no speculative abstraction), Caveman (concise communication; this note
stays complete), `reverse-engineering` (reading the recovered native reader offsets/subtype table
from the pinned notes and probe rather than guessing), `game-recon` (engine/format route already
pinned: ForgeLight-family, managed-vs-native split). No game-automation was used - the worker does
not control the shared game window.

Cross-links: [[Research/Kart Native Race Entry Contract]],
[[Research/Kart Community World Binding]], [[Research/Kart Native Progress Codec]],
[[Research/Native Zone Transition Codec]], [[Reviews/Delegated Racing Readiness]].
