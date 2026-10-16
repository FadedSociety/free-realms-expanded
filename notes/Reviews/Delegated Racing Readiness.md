# Delegated racing readiness (read-only audit)

Read-only audit, 2026-10-09 (UTC). Scope: kart racing + demolition derby and the
delegated work items around them (queue 004 kart racing, 005 soccer, 007 derby,
024 furniture, 041 racer/derby progression), against the actual repository and
gateway routes. No production/source/content/save/UI/binary edits; no client
input. Owner: delegated-racing-audit-20261009. Evidence:

| Path | Role |
| --- | --- |
| `lab/reports/delegated-racing-audit-20261009/kart-core-tests.log` | derby/kart/zone focused run |
| `lab/reports/delegated-racing-audit-20261009/gateway-and-pieces-tests.log` | pieces 018/024/041 + default gateway gate |
| `lab/reports/delegated-racing-audit-20261009/gateway-ignored-green.log` | 3/3 private gateway checks reproduced |
| `lab/reports/delegated-racing-audit-20261009/gateway-ignored-repro.log` | fixture-dependency failure when the packaged content is used |
| `lab/reports/delegated-racing-audit-20261009/soccer-005-check.log` | 005 soccer claim reproduced |
| `lab/reports/delegated-racing-audit-20261009/audit.json` | machine-readable findings + source pins |

## Verdict

No kart or derby activity is launchable on the current build. The only
registered kart behaviour is the opt-in **102/1 lobby-list reply**; the race
stream (inner 33), derby stream (inner 34), matchmaking (141) and the
progression star packet (21) have no gateway route and fall through to
`Combat::handle`'s `_ => Ok(CombatOutcome::default())`, i.e. a client `33/9`
register is silently inert. `kart_racing`, `derby` and the progression module
are not in `lib.rs`. The user-facing phrase "rocket racing" has **no recovered
Free Realms counterpart**: the original activities are kart/vehicle racing
(TYPE 10/13/19/20) and demolition derby (TYPE 11); "rocket" exists only in
item/cosmetic locale strings.

## Registration vs routing (source pins in audit.json)

| Module | lib.rs | gateway/gameplay route | focused tests |
| --- | --- | --- | --- |
| `kart_lobby_native` | yes | yes: exact inner `102/1` reply, dev-tools + `kart_lobby.json`, cached once, malformed refused | 10 passed, 1 ignored |
| `kart_progress_native` (33/14) | yes | **none** - no reference outside `lib.rs` | 9 passed, 1 ignored |
| `zone_transition` (op31) | yes | **none** - gateway emits op31 via `encounter::zone_begin_packet` instead | 9 passed, 1 ignored |
| `kart_racing` (33/1/5/14 session) | **no** | none - `#[path]` test harness only | 13 passed |
| `derby` (34/*) | **no** | none - `#[path]` test harness only | 11 passed |
| `soccer`, `soccer_match` | yes | none - no gameplay/gateway reference | 11+1 ignored, 10 passed |
| `restoration/piece_018/024/041` | no | not routed | 11+2i, 6+1i, 11 passed |

Route detail: `gateway.rs` forwards unknown inner opcodes as
`Event::Application`; `gameplay.rs::handle_in_world` handles only the
dev-gated `102/1` kart reply before the opening/quest/combat chain, and
`Gameplay::handle` ends in `Combat::handle`, whose catch-all returns an empty
outcome. A `33/9`, `34/*` or `141/1` frame therefore produces no reply, no
state change and no error.

## Kart phase gaps (spawn / control / results / reward / reload)

* **Spawn - absent.** Nothing produces `33/34`, `33/11`, `33/5` or `33/14`.
  `kart_racing::RaceSession` is a local checkpoint/lap authority, not a wire
  sender, and it is unregistered. The exact start contract (33/9 request,
  SetRaceConfig, CreateProxiedVehicleRace with the kart body asset + 400.0,
  client-data and progress blocks, counters) is recovered in
  `notes/Research/Kart Native Race Entry Contract.md` but has no implementation
  or route.
* **Control - absent.** No handler for client `33/6/7/17`; the module treats
  the 5/14 streams as opaque observations (`observe_racer_state`) and never
  derives position from them.
* **Results - absent.** No finish/results message is recovered; the capture's
  33 stream simply stops and is followed by 141 matchmaking + a fresh 102/2.
  `RaceSession::finish` is reconstructed checkpoint policy with no wire
  binding.
* **Reward - unbound.** `RaceSession::claim_reward` grants only from a priced
  `TrackLayout`; default is `Unpriced`, and nothing in the gateway/economy
  path calls it.
* **Reload - unbound.** The session has snapshot/restore logic and tests, but
  no gameplay/save persistence and no gateway member ownership binding.
* **Scene entry - mismatched writer.** The gateway's op31 is
  `encounter::zone_begin_packet`, which hardcodes position/rotation `w = 1.0`;
  the captured `sg_kt_track` scene frame carries rotation `(0, 0, 1, 0)`. The
  registered `zone_transition` codec preserves the real `[f32; 4]` rotation and
  should be the writer used when a kart scene is ever emitted.

Derby gaps are wider: `derby.rs` recovers the envelope, subtype/length table,
participant keys, vehicle-config asset names, vehicle-hit and announcer
layouts, but arena geometry, health/elimination, scoring values and the result
table are unrecovered; there is no launch route and no reward path (queue 007
stays open).

## Rocket racing vs recovered activities

`MiniGameData.txt` (783 rows, pinned client snapshot) has no rocket row and no
rocket type; its racing entries are TYPE 10 (68 rows: kart/vehicle races),
TYPE 11 (48: demolition derby), TYPE 13 (5: "Vehicle Race In The Wilds",
"Shipwreck Cove") and TYPE 19/20 (1 each: "Wildwood Kart Racing"). A name scan
finds zero activity rows containing "rocket"; the locale export only has
Rocket Fuel (67383), Rocket Ship (6557), Fireworks Rocket and Rocketstep Shoes
as item/cosmetic strings. A "rocket racing" request must be specified as new
constructed content, not presented as restoration.

## Delegated-piece review (artifacts, not reported status)

* **004 (kart racing)** - `tests/kart_racing.rs` 13/13 and `tests/kart_review.rs`
  12/12 reproduced green against the `#[path]`-included source; the module is
  still unregistered and has no route, matching its "physics/integration/native
  pending" queue status.
* **005 (soccer)** - 11 passed +1 ignored and 10 passed reproduced; modules are
  registered but unrouted, matching "authoritative physics/native pending".
* **007 (derby)** - 11/11 reproduced; wire-study only, no launch route.
* **024 (furniture catalogue)** - 6 passed +1 ignored; unrelated to racing
  (included only because it shares the delegated batch).
* **041 (racer and derby progression)** - 11/11 reproduced plus the recorded
  RED (unresolved imports against the skeleton) and the Python checker run;
  the job/rank/vehicle definitions are recovered, but the outcome link
  (`MiniGameStarsEarnedPacket`, inner 21) has no route and the credit contract
  is reconstructed, so nothing can grant progression today.

## Focused test receipts (this audit)

| Command | Result |
| --- | --- |
| `cargo test --test kart_racing --test kart_review --test kart_lobby_native --test kart_progress_native --test zone_transition --test derby` | 64 passed, 0 failed, 3 ignored |
| `cargo test --test kart_lobby_gameplay -- --ignored` with a catalogue-free flat content dir + `rust-gateway-reference` | 3 passed, 0 failed |
| `cargo test --test kart_lobby_gameplay -- --ignored` pointed at the packaged content (includes `kart_lobby.json`) | 1 failed (phase-1 "no catalogue inert" expects a catalogue-free fixture) |
| `cargo test --test soccer --test soccer_review` | 21 passed, 1 ignored |
| `cargo test --test piece_041 --test piece_018 --test piece_024` | 28 passed, 3 ignored |

Reproduction note: the private gateway gate requires `OSFR_KART_LOBBY_CONTENT`
to be a flat content directory **without** `kart_lobby.json` (the test writes
its own synthetic catalogue) and `OSFR_GATEWAY_REFERENCE` =
`lab/references/rust-gateway-reference`. Pointing it at
`opening-native-lobby-20261009T163603Z/content` fails by fixture design; this is
a reproducibility requirement, not a product regression.

## Concrete next steps for builders

1. Register `kart_racing` + `derby` (lead-owned `lib.rs`) and route inner 33/34
   through `Gameplay::handle_in_world`; route 33/14 through the already
   registered `kart_progress_native`.
2. Implement the recovered start exchange (33/9 -> 33/34 + one 33/11 per
   vehicle + 33/5 + 33/14, then 33/2 -> 33/3/1/4) from
   `notes/Research/Kart Native Race Entry Contract.md`; keep client 33/6/7/17
   observation-only until physics is recovered.
3. Use `zone_transition::encode` (rotation w preserved) for any kart scene
   frame; do not reuse `encounter::zone_begin_packet` for `sg_kt_track`.
4. Nothing may grant kart/derby rewards until a result message and a priced
   track/derby schedule exist; piece 041 must stay uncredited until then.
5. Derby (007) needs arena/launch/scoring recovery before a route or native
   test path can be assigned.

No native kart/derby run exists under `lab/runs`; the only reproducible
native-shaped gate today is the private gateway lobby test above.

Cross-links: [[Research/Kart Native Race Entry Contract]],
[[Research/Kart Racing Restoration]], [[Research/Pieces/007 Demolition derby]],
[[Research/Pieces/041 Racer and derby progression]],
[[Research/Pieces/018 Minigame native test routes]],
[[Reviews/Kart Lobby and Finish Binding Review]].
