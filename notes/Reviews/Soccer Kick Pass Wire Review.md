# Soccer kick/pass wire independent review

Read-only review of `notes/Research/Soccer Kick Pass Wire Mapping.md` revision 2 (SHA-256
`EB8287EC…`), `scripts/pieces/soccer_wire.py` (`B24D4891…`), `tests/pieces/test_soccer_wire.py`
(`530CE4AA…`) and the private bundle `lab/reports/soccer-wire-rev2-20261009T120723Z/`, 2026-10-09.
Reviewer owns this note and `lab/reports/soccer-wire-independent-20261009T1220Z/` only; no source,
research note, test, lib, gateway, UI, state or process was changed. Skills: RTK (normal commands),
Ponytail (one small independent audit script), Caveman (terse report), reverse (raw exe/vtable/state
table derivation).

Verdict: the sub-14 action-channel mapping and the per-flow interpretation are correct and were
re-derived independently; the sub-13 "server-to-client only" conclusion is **overstated for the
evidence presented** and is downgraded here to "no send path was located; the single located
dispatch passes want-to-send = 0". Native kick/pass key binding and a captured sub 13 remain
missing, exactly as the note says; nothing in it is a native-acceptance claim.

## Independently verified

`audit.py` re-derives the following from the preserved exe/pseudocode/corpus alone
(`audit.log`, `audit-report.json`, `AUDIT_OK = True`):

| Claim | Re-derived result |
| --- | --- |
| Inputs pinned | exe `26CC1C52…`, pseudocode `F84A5867…`, corpus `ABF8010F…` all re-hashed equal to the note |
| Vtable slots | `SoccerBall+0x18` → `0x00CDE9B0`, `SoccerBallBase+0x18` → `0x0107BCE0`, `SoccerPlayer+0x0C` → `0x00CDC5C0`, `SoccerPlayer+0xA0` → `0x00CDC810`, `SoccerPlayerBase+0x98` → `0x01072020`, `SoccerProcessor+0x10` → `0x00B80AC0`, all read from raw exe bytes |
| Constructors | both packet constructors embed their vftable (`0x018232F4`, `0x01823354`) at function offset 57; sub ids 0x0D/0x10 |
| State table | all 64 names at VA `0x01BA2338` resolve; 16–34 are the kick/pass states, 35 StealTackle, 36–38 Slide*, 51–63 `cSoccerGoalieState*`; index 44 is `cSoccerPlayerGetupBack` (naming quirk) |
| Sub 14 sender | the sender body writes player guid `param_1[4..5]`, state byte `(u8)param_1[0x26]`, previous state `param_1[0x27]`, sequence `param_1[0x2c]`, aux byte `(u8)((float)param_1[0x29]*const)` and three float4s — the note's offsets are correct |
| Setter | `param_1[0x26] = param_2` plus the documented remaps (2→0x0B, 8→0x0E, 9→0x0F, 1→0x3D goalie) and the `+0x0C` sender dispatch are present |
| Flow interpretation | own decode of the 2978-record corpus: sub-13 records **0**; sub-14 client flow (1) states `{1:11, 2:16, 35:1, 36:3, 37:3, 38:3}` — no kick/pass state; server flow (0) carries `{17:2, 22:2}` plus goalie states; sub-16 client bits `{0,1,2}` only, server bits include `{8,17,65,256}` — matching the note |
| Corpus precision | 251 raw sub-14 records, **250 clean 62-byte** ones; the extra is a zero-length flow-0 artifact (`soccer-01284-flow0-frame808-op47-sub14.bin`, 0 bytes) that must be excluded, as the note's "250 decode cleanly" wording implies |
| Checker reproduction | my re-run of `soccer_wire.py` produced a byte-identical `report.json` (`DB3FE6B5…`) and `checker.log` (`EA3B558F…`) to the rev2 bundle; `test_soccer_wire.py` ran **24/24 OK** in my own log |

## The sub-13 universal negative is not supported as stated

The note's revision-2 headline says the client "never authors sub 13" and that sub 13 "is a
server-to-client packet in this client". The supporting evidence is bounded:

* `check_want_to_send_zero` (`soccer_wire.py:489-520`) reads a **single 64-byte window** at VA
  `0x00B82871` and asserts `8b 52 18` → `6a 00` → `ff d2`. My own read of that window reproduces the
  order (`slot_load +21`, `push 0 +24`, `call edx +48`), so *that one site* passes want-to-send = 0.
* The exact dispatch string `(*piVar9 + 0x18))(&local_500)` occurs **exactly once** in
  `pseudocode.c`, and `FUN_00cde9b0` itself occurs **twice** — its header and definition only, i.e.
  **zero textual callers**. The listing therefore cannot enumerate callers; every invocation is
  vtable-mediated.
* The exe contains **122** occurrences of the generic `8b 52 18` (slot-0x18 load) pattern. Deciding
  which of them dispatch the `SoccerBall` vtable requires resolving each site's receiver type; that
  was not done, and a byte-pattern count cannot substitute for it.
* The send branch inside `FUN_00cde9b0` is gated on `param_6 != 0`; the located dispatch passes 0,
  but "no other caller exists" would additionally need (a) an enumeration of every vtable that
  stores `0x00CDE9B0` — note `SoccerBallBase+0x18` is overridden by `SoccerBall`, so base-class
  dispatch of that slot resolves to a different function — plus callers of each, and (b) coverage of
  any other module/indirect path the decompiler did not reconstruct.

Correct qualified wording for the note: *the one located dispatch of `SoccerBall vftable+0x18`
passes want-to-send = 0 and the impulse sender has no located caller, so no client sub-13 author
path was found in this decompiled listing; absence of all send paths is not established.* The
checker docstring "Prove the only sub-13 dispatch" should be softened to match the single-site
assertion it performs. The note's own "Missing"/"Limitations" text already admits the caller was not
located; it is the headline/scope phrasing that overreaches.

## Authority and routing checks

* Sender/profile auth: the note correctly requires a server to bind a sub-13 impulse to the
  authenticated session and possession (the packet carries no player id). The same client-authored
  guid rule applies to sub-14 (state byte + guid) and sub-16 (guid + control bits); the note states
  it only for sub-13. The production slice already enforces the binding —
  `soccer_match::apply(sender, now, intent)` rejects `sender != intent.player()` — so integration
  must route the decoded sub-14/16 guid through that guard rather than trusting the field. Low
  documentation gap, no code defect in the reviewed work.
* Community route: the pinned `Sanctuary` and `FreeRealms-Legacy` trees contain only
  `src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs` mentions of
  `ImpulseBall`/`UpdatePlayerState`/`UpdatePlayerControls` — the name table, no serializer, fields or
  sample. The root's "community pinned decompile exact type/order" ask therefore has no community
  artifact to reuse; the exact type/order in the note comes from the native decompile, and the note
  says so. That is a route divergence to acknowledge, not a fabricated community claim.
* Missing, confirmed: no captured sub-13 vector (0/2978), no key/mouse binding table, input-action
  ids in `FUN_00afb4a0` still unnamed, aux byte and float4 semantics unproven. No native acceptance
  claim is made anywhere.

## Evidence

| Artifact | SHA-256 |
| --- | --- |
| reviewed note rev2 | `EB8287ECA61EEC1B1FADCF94CD7B28FCF62B2A0DD1250B8820641D3FEF7F37D5` |
| `scripts/pieces/soccer_wire.py` | `B24D4891DB8DDEE4…` (full value in `final-hashes.txt`) |
| `tests/pieces/test_soccer_wire.py` | `530CE4AA70969E03…` |
| `lab/reports/soccer-wire-independent-20261009T1220Z/audit.py` | `A64711801CA3D980…` |
| `…/audit.log` | `DF71D01159843A69…` |
| `…/audit-report.json` | `BE372A24388A2CEB…` |
| `…/tests.log` | `AB689B0A191FBDE7…` (24/24 OK) |
| `…/checker.log`, `…/report.json` | `EA3B558F1F363454…`, `DB3FE6B526AC6744…` (identical to rev2 outputs) |
| exe / pseudocode / corpus | `26CC1C52…` / `F84A5867…` / `ABF8010F…` |

Commands: `python -m unittest discover -s tests/pieces -p "test_soccer_wire.py" -v` (24 OK);
`python scripts/pieces/soccer_wire.py --output <private>/report.json`; `python audit.py`
(`AUDIT_OK = True`). All reads are over preserved lab inputs; no game, gateway, save, UI or process
was touched. Cross-links: [[Research/Soccer Kick Pass Wire Mapping]],
[[Native Cave Replay 114509]], [[Design/MMO Worker Contract]].
