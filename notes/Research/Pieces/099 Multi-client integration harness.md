# 099 Multi-client integration harness

Status: headless harness + recovered vocabulary, 2026-10-09 05:42 UTC. Queue item 099.
Deterministic launch/log/state comparison now runs in-process for party, trade and activity
scripts, so two-client behaviour can be regression-tested without opening two native windows.
This is **not** native or gameplay acceptance: no game process is launched and no UI is touched.

Owned artifacts (all new):

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_099.rs` | Harness: scenario scripts, canonical run document, first-divergence comparison, recovered trade family-59 vocabulary and commands, recovered family-175 intent subset |
| `prototype/osfr-tools/tests/piece_099.rs` | 9 behavioral tests; `#[path]`-includes the module (and the candidate piece 020) so no shared `lib.rs` edit is needed |
| `scripts/pieces/piece_099.py` | Read-only evidence checker over the preserved corpus and the shipped harness source |
| `tests/pieces/test_piece_099.py` | 16 stdlib tests: hermetic parsers, a synthetic end-to-end corpus, real-corpus pins |
| `../lab/reports/piece-099-20261009T054208Z/` | Private bundle: `evidence.json`, `provenance.json`, `run.log`, test logs, `report.md` |

## How the harness works

A `Scenario` is a fixed client roster plus ordered steps keyed by `(tick, client)`; validation
requires strictly increasing keys and known clients, so a run has no hidden ordering. `run` feeds
each step to a dispatcher closure and records one canonical record per step: verdict
(`accepted`/`refused`/`unhandled`), refusal detail, hex of the recovered wire bytes, reply count and
digest, snapshot digest and change flag. The final state and the launch record (harness version,
case, seed, roster, step count) complete the document.

`compare` diffs the canonical JSON of two runs and reports the first divergence as a concrete path
with both values, e.g. `log[2].client`, `launch.clients[0].name` or `state.groups[0].capacity`.
Because the document is key-ordered and clock-free, two runs of the same case are byte-identical.
Each record also carries the reply count and a digest over the length-prefixed replies, so a run
that answers differently cannot compare equal.

## Cases

| Case | Script | Asserted behaviour |
| --- | --- | --- |
| `Case::Party` | open, invite, accept, ready x2, owner kick, re-invite, accept, quit | deterministic roster changes; owner-only kick; final owner-only formation |
| `Case::Trade` | invite reply, lock x2, offer item x2, accept x2, cancel | recovered family-59 wire on every step; the server under test answers or is recorded `unhandled` - no fabricated settlement |
| `Case::Activity` | open, invite, accept, ready x2, launch, non-member enter, late cancel | one launched instance with a 2-member roster; `NotInInstance` and `AlreadyLaunched` refusals without state change |

## Recovered vocabulary (original)

**Trade family 59 (`0x3b`).** `FUN_00c6a7b0` stores `0x3b` in the packet family slot; every
`Trade*Packet` constructor calls it with its sub-opcode immediately before the vftable store. The
table is 18 rows: 2 Invite, 3 InviteReply, 4 StartSession, 5 EndSession, 6 Accept, 7 Cancel, 8 Lock,
9 OfferItem, 10 CoinCount, 11 UpdateCoinCount, 12 UpdateAddItem, 13 UpdateItemCount, 14
UpdateAccept, 15 Message, 16 Rejected, 17 UpdateLock, 18 Confirm, 19 ItemRemoved. The implemented
community reader opens the family with a **one-byte** sub-opcode and names the same rows; the pinned
older server enum agrees on `BaseTradePacket = 59` and every value.

**Client-to-server trade commands.** The pinned community serializer recovers six command layouts,
which the harness encodes and round-trips: `InviteReply` (session, accepted), `Accept` (session,
revision), `Cancel` (session), `Lock` (session, revision, locked), `OfferItem` (session, item,
resulting count), `CoinCount` (session, resulting coins). The scripts use a scripted session id and
revisions, not captured server assignments.

**Grouping family 175.** The harness scripts the recovered client requests owned by
`src/restoration/piece_020.rs`: `InviteMemberRequest` 20, `InviteResponse` 17, `MemberReady` 18,
`KickMemberRequest` 21, `MemberQuit` 19, `OwnerLaunchRequest` 22. Opening a group, cancelling a
formation and entering an instance have **no** recovered client request, so those intents carry no
wire bytes and stay harness-level steps.

## Commands

```powershell
# RED before the module existed: tests\..\src\restoration\piece_099.rs was missing.
& '.\scripts\cargo.ps1' test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_099
# GREEN: 9 passed; 0 failed.

& '.\scripts\cargo.ps1' test --manifest-path prototype/osfr-tools/Cargo.toml --lib
# 24 passed; 0 failed.

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_099.py `
    --lab ..\lab --out ..\lab\reports\piece-099-20261009T054208Z --stamp 20261009T054208Z
# 18 trade rows / 6 grouping intents / 0 problems; evidence_sha256 a07a322a...620a

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces `
    -p "test_piece_099.py" -v
# 16 passed.
```

Three deliberate mutations (a swapped launch sub-opcode, a changed `Confirm` discriminant and a
suppressed `state_changed` flag) each failed the expected test and were reverted; details are in the
lab `report.md`.

## Reconstructed vs original vs gaps

Reconstructed: the three scripts, the scripted session values, the canonical run document and the
comparison policy. The harness deliberately contains **no** trade settlement rules - the original
server's two-sided lock/accept, revisions, disconnect rollback and staleness handling are
unrecovered and remain owned by the trading item.

The local server currently dispatches neither family: a gateway without gameplay content drops
application messages, and `Gameplay::handle` ends in a silent `Ok(out)` for unknown families. The
harness records that as `unhandled`, which is an integration gate rather than a pass.

## Remaining gates

1. Lead registers `pub mod piece_099;` (and the candidate `piece_020`) when accepted, then the
   `#[path]` includes in `tests/piece_099.rs` go away.
2. Party (089), trade (085) and activity modules plug in as dispatchers; the trade case must flip
   from `unhandled` to a real two-sided settlement assertion.
3. Native gate: two actual clients on one desktop or a second host for party, trade and activity.
   This harness never claims that gate.

Caveat observed during this item: the whole-crate `cargo test` run currently stops on other
workers' in-flight tests (`tests/piece_051.rs` missing its module, `tests/piece_097.rs` type errors).
The owned test target and the library tests pass independently.
