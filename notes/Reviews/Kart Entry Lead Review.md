# Kart entry lead review

2026-10-09. Independent bounded review of the native kart race-entry slice
(`src/kart_entry.rs`, its vectors, receipt and the lead's new `kart_gateway`
binding). No implementation file was edited by this review; the note and
`lab/reports/kart-entry-independent-20261009/` are the only artifacts.

## Verdict

The codec, ordering, phase and bounds work in `kart_entry.rs` holds up: the
synthetic vectors, the ignored private capture gate (byte-identical re-encode
of twelve pinned frames) and the reuse gates all pass, and the lead's gateway
now rebinds exactly the three GUID slots the capture proves. Two integration
gaps and two low-severity details remain; the most important is that the
85-byte `33/5` client-data block carries per-player state (a transform and the
client's own register words) that no rebinding path touches.

## Identity semantics (the lead's 41/114 question)

Reader- and capture-proven: the value in the capture
(`0x4b52cc6a9573ca91`, wire bytes `91ca73956acc524b`) is the **player's
cross-activity actor GUID**, not a race- or vehicle-scoped session.

- A byte scan of all 65,807 preserved frames finds it in **16,550 frames**
  across five families: `33` racing (6,364), `34` demolition derby (9,668),
  `76` soccer (518), `141` matchmaking (6) and `41` encounter (12). A
  race-scoped value cannot appear in derby, soccer and matchmaking.
  Evidence: `session-identity.json`.
- The client takes the `33/9` register session from its **local player
  object**, not from a server-supplied field: the register sender
  (`pseudocode.c` 1583830-1583836) writes `uStack_1c/uStack_18` from the object
  returned by the local-player accessor and `0x98/0x9c` for the two trailing
  words. The server therefore cannot impose a fresh GUID and expect the client
  to echo it.
- The `33/14` handler resolves the payload racer GUID through
  `FUN_00b9e0c0` and only writes the local progress fields when the resolved
  entity equals the local racer (`param_1 + 0x2cc`, pseudocode 1585114-1585122).
  A fresh GUID that does not match the client's own world identity would leave
  the local lap/timer fields unbound.
- The own-vehicle `33/11` GUID sits at **attribute-block offset
  `block_len - 58`** (6,136 -> 6,078 in the captured own kart) and equals the
  same value; the five proxied karts carry their own racer GUIDs at the same
  offset (`0000002001700652`, `...30...`, `...50...`, `...60...`, `...70...`),
  each matching its `33/14` progress row. Evidence:
  `vehicle-attribute-guid.json`.

Conclusion for the adapter: bind the **character GUID** (the test character
`1`) everywhere, which `kart_gateway::begin` already does
(`context.session = character.guid`, then `rebuild_entry`/`details`). A fresh
server GUID is only safe if the client's world entity for that racer also
carries it, which nothing in the capture shows.

## What the gateway gets right

- Rebind of the three proven slots: `entry_vehicles[0].attribute_block[
  vehicle_guid_offset..]`, `progress[*].racer_guid` and `progress[*].tail_0x29`
  where they equal the captured session; `details()` rewrites instance at
  `[8..12]` and session at `[32..40]`.
- Load-time ownership proof: the content is refused unless the `41/114`
  templates name the same encounter/instance/session, the own vehicle's GUID
  slot equals the captured session, and some progress row carries the session
  in both GUID fields.
- The `41/2` first-entry frame the gateway builds is the 21-byte inner form,
  matching the captured first-entry frame (`05 00 01 15 ... 3e 00 0e 00 f0 03
  00 00` + nine zero bytes); the later captured entry carries the session in
  those bytes, which the gateway does not replay.

## Findings

**K1, medium, `33/5` client data is replayed unrebound.** The 85-byte block has
no GUID: `FUN_00b9fb10` reads six 12-byte blocks (`FUN_009d4230` = three
dwords each), then a dword at 72, a byte at 76 and two dwords at 77/81. The
captured bytes are a 4x4 basis with translation `(795.0, 86.19, 365.0)` at
36..48 and the client's own register words `0x2a6ac44a, 1` at 77..85 - the
same pair the client sent in `33/9`. The dispatcher selects the target racer
by the byte at 76 (`(byte >> 3) & 7`, pseudocode 1584651) against the client's
local table, so there is no foreign-GUID leak, but the transform and the
register words are captured-player state. `kart_gateway::rebuild_entry` does
not touch `entry.client_data`. Decide and test one policy: rebind the
translation from the character position (and the words from the live register
payload), or declare the captured transform a fixed entry spawn.

**K2, medium, positional ownership assumption.** `RaceEntryContent` accepts up
to eight entry vehicles, but `KartGatewayContent` carries one
`vehicle_guid_offset` and `rebuild_entry` rewrites index 0 only; the load proof
also checks index 0 only. Nothing rejects a bundle whose owned kart is not
first. Either assert `entry_vehicles.len() == 1` (what the capture shows) or
carry an owned flag per vehicle.

**K3, low/medium, malformed `33/x` errors the whole input chain.**
`KartEntryHandler::handle_inner` returns `Err(EntryError)` for malformed
frames, `KartGateway::handle_entry` maps that to `io::Error`, and
`Gameplay::handle_kart_entry` propagates it, so one short `33/9` payload from a
client becomes an `io::Error` for the whole `handle_in_world` chain. Refusals
are correctly swallowed. A hostile or buggy client should not be able to turn
one frame into a connection-level error; consider logging and dropping
malformed entry frames like refusals (the existing gateway test asserts the
error, so this is a deliberate policy choice to confirm).

**K4, low, double `close()` clears the retired-session marker.**
`close()` assigns `retired_session = self.context.take().map(...)`, so a second
`close()` with nothing open sets it to `None`; a late replay for that session
then reports `NoOpenContext` instead of `StaleSession`. Still refused, only
the label and log differ. `src/kart_entry.rs` 583-586.

**K5, low, receipt hash drift.** The receipt pins `tests/kart_entry.rs`
`70e0e7ce...`; the file is now `4b013802...` (edited after the receipt).
`src/kart_entry.rs` still matches its pinned `f13423bd...`. Refresh the receipt
when the test edit settles.

**K6, informational, live workspace.** `tests/kart_service.rs` (another
worker's in-flight target, 2:41 PM) currently fails to compile (85 errors,
`KartService: Digest` and missing types), so a full kart suite run cannot
include it yet. It is outside this review's scope and was excluded from the
gate run below.

## Checks executed

- Pinned-capture hash verification: all twelve frames in the receipt match
  (`guid-offsets.json`, `all_pinned_match: true`).
- Byte probes (new, read-only, in the report directory): session occurrence
  scan over 65,807 frames; 33/5 structure; 33/11 attribute-block GUID offsets
  across all six captured sub-11 frames; capture-wide 33/5 vs 33/14 frame
  pairing (six same-frame pairs; the 33/5 first eight bytes are always
  `000000003f800000`, never a GUID).
- Trusted gates: `kart_entry` 13 passed / 1 ignored; private capture gate
  (`-- --ignored` with `OSFR_KART_ENTRY_CAPTURE` + `OSFR_KART_ENTRY_CONTENT`)
  1 passed, byte-identical re-encode; `kart_gateway` 5; `kart_rewards` 11;
  `kart_progress_native` 9/1 ignored; `kart_racing` 13; `zone_transition`
  9/1 ignored. Logs in the report directory.

## Residual risk

The 33/5 transform's effect on the local racer is not proven either way (the
dispatch path shown only branches for a non-local racer), so K1 is a
policy-and-test gap rather than a confirmed visual defect. The entry bundle is
still a capture re-encode, not a live session: no native entry run exists, and
the lead owns that gate.

Evidence bundle: `lab/reports/kart-entry-independent-20261009/`.
