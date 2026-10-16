# 094 Reliable transport stress

Status: closed for review, 2026-10-09 05:16 UTC. This piece adds a several-client stress
harness over the existing production codec, sender and receiver, and closes the acceptance
gaps that no single-link test could reach: concurrent clients, session replacement, sequence
rollover beside real loss/reordering, bounded send queues and exact delivery. It also pins the
two-part gateway policy that makes replacement safe, including the unguarded counterfactuals
that would admit stale frames. No gameplay, economy, socket or timer behaviour is added, and no
native acceptance is claimed.

Owned paths: `prototype/osfr-tools/src/restoration/piece_094.rs`,
`prototype/osfr-tools/tests/piece_094.rs`, `scripts/pieces/piece_094.py`,
`tests/pieces/test_piece_094.py`, this note and `lab/reports/` (final pass
`piece-094-20261009T052118Z`; the intermediate passes `piece-094-20261009T051251Z` /
`piece-094-20261009T051626Z` and the checker RED `piece-094-20261009T051024Z` remain for
history). No shared `lib.rs`, gameplay, gateway, Cargo manifest, launcher, content or state file
was touched.

## Recovered / already-verified production facts this piece drives

Nothing new was reversed for 094; the wire spec already exists and is reused read-only:

* `prototype/osfr-tools/src/soe.rs` - bounded codec (`Codec` line 77) and the ordered reliable
  receiver (`ReliableReceiver` line 255, `receive` line 306): seeded CRC, zlib framing,
  selective (`17`) / cumulative (`21`) acknowledgement, bounded reorder window and fragment
  assembly. Provenance and vectors: `notes/Research/SOE Transport Notes.md`,
  `tests/fixtures/TRANSPORT.md`.
* `prototype/osfr-tools/src/soe_send.rs` - bounded reliable sender (`ReliableSender` line 80,
  `queue_message` line 131, `poll` line 206, `acknowledge` line 253, `pending_packets`
  line 291) with retransmission timing and terminal retry exhaustion.
* `prototype/osfr-tools/src/session.rs` line 6 - recovered handshake seed
  `CRC_SEED = 1_884_358_976`, the value the gateway keys each session with.
* `prototype/osfr-tools/src/bin/osfr-gateway.rs` - the session-replacement policy under test:
  `generation = generation.wrapping_add(1)` (line 301) and `seed = CRC_SEED ^ generation`
  (line 302) per replacement, plus the refusal of a negotiated zero trailer
  (`gateway requires a nonzero CRC trailer`, line 130). The running gateway itself is
  single-peer: datagrams from any other peer are ignored while a session is active.

## Reconstructed (stress policy only, labelled, not recovered behaviour)

* The deterministic loss / duplicate / reorder schedule (one seeded xorshift, no new
  dependency), bounded one-way latency in ticks, and ACK loss / duplicate delivery.
* The retained wire-byte accounting: distinct unacknowledged sequences are tracked by decoding
  each polled frame with the production codec, so the harness can assert the send queue stayed
  inside `max_queued_bytes` instead of trusting a counter.
* Session replacement semantics: a replacement gets a fresh codec, sender and receiver at
  sequence zero; frames still in flight keep their old session tag; unsent send-queue content
  is reported as discarded rather than replayed.

## Acceptance evidence

One Rust target exercises all four required hazards; the checker re-runs it and enforces the
rules in `scripts/pieces/piece_094.py`.

| scenario | what it proves | measured |
| --- | --- | --- |
| `several_clients` | 4 clients, 20% loss, 15% duplicates, 25% reorder: every queued message delivered exactly once, in order | queued 1200 / delivered 1200, loss 315, duplicates 216, reordered 322, retransmits 386, queue-full 285, retained peak 1614 / 8192 |
| `session_replacement` | with a second client live, replacement rejects in-flight frames from the previous session and then delivers its own stream exactly; the concurrent client stays exact | stale rejected 16, stale accepted 0, discarded 24, delivered 51 / queued 75, client 1 25 / 25 |
| `rollover_bounded_queue` | 65,600 messages cross the u16 sequence space beside a 4 KiB send queue | delivered 65,600 / 65,600, queue-full 5464, retained peak 2320 / 4096, no corruption |
| `stalled_client_isolation` | a partitioned client stops at its retry limit without disturbing the other client | `failed=1` for client 0 at `RetryLimit`, client 1 exact 60 / 60, receive errors 0 |
| `unsafe_zero_crc`, `unsafe_reused_key` | negative controls: the harness *can* observe stale acceptance, so the replacement result is not vacuous | stale accepted 16 each, delivery corrupted past the replacement boundary |

The two counterfactuals are the reason the gateway needs both halves of its policy: rotating the
key while negotiating `crc_length = 0` admits 16 stale frames, and keeping a nonzero trailer
while reusing the key admits 16 as well. Only keyed CRC + a changed key isolates the replacement
session.

## Commands (all run from the project root)

```
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_094','--','--nocapture','--test-threads=1')
  -> 5 passed; 0 failed; 0 ignored (3.9 s)

& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_094','--test','soe_transport','--test','soe_send','--','--test-threads=1')
  -> piece_094 5 passed / soe_send 7 passed / soe_transport 10 passed, 1 ignored

& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_094.py'
  -> status=pass output=...\lab\reports\piece-094-20261009T051626Z

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_094.py" -v
  -> Ran 10 tests, OK

rustfmt --edition 2024 --check (both owned Rust files) -> clean
```

The first RED for this piece was the missing module
(`couldn't find file tests\..\src\restoration\piece_094.rs`, exit 101). A second RED is retained
in `lab/reports/piece-094-20261009T051024Z/`: the checker initially reported `status=fail`
because `cargo test -- --nocapture` appends a test's captured output to its `test <name> ...`
line, so the summary prefix is not always at column zero. That parser defect is fixed and now
has its own regression test (`test_parses_a_summary_appended_to_a_harness_line`).

The full workspace suite is currently not a usable gate: `cargo test` stops on other workers'
in-flight targets (`tests/piece_082.rs`, `tests/piece_098.rs` reference modules that do not
exist yet). That is unrelated to this piece and was not modified.

## Integration steps and remaining gates

* Lead registration: add `pub mod piece_094;` to `src/lib.rs`. The module already uses
  `crate::soe`, `crate::soe_send` and `crate::session`, so no import rewrite is needed; the test
  target then drops its three `#[path]` crate-mirror modules and the `#[path]` include.
* Native / integration gate (not claimed): there is no original-client multi-session capture.
  The gate that would close this item is a real client restart against the running gateway
  showing that pre-restart frames are dropped, the replacement stream is exact, and the
  gateway's save/restore around `Session::new` survives the handover. That needs the lead's
  shared client window and is not something this harness can assert.
* Multi-client serving is genuinely absent, not merely untested: the gateway handles one peer.
  Piece 099 owns the multi-client harness; 094 deliberately does not fake it.

## Limits and explicit gaps

* Replacement is modelled as discard-unsent. The harness proves stale frames cannot reach the
  replacement session and that post-replacement delivery is exact; it does not prove the
  gateway's own snapshot/save ordering around the swap.
* Frame isolation relies on the keyed CRC. No original session identifier appears on the post-
  handshake wire in the recovered schema, so a zero trailer genuinely cannot be compensated for
  by anything else in this design.
* `duplicate_acks` in the summary counts acknowledgements the production sender refused as
  duplicate or stale. Repeat cumulative ACKs from one window make that number normally larger
  than the number of intentionally duplicated ACK frames.
* The original client's own retransmission behaviour was not instrumented; only the emulator's
  sender/receiver pair is stressed.

## Skills applied

* RTK - every command ran normally first; no blind prefixing.
* Ponytail - the harness reuses the production sender/receiver/codec and the standard library
  only; no new dependency, no second implementation of framing or ACK semantics.
* Caveman - progress updates stayed terse; this note stays complete.
* game-recon, game-research-websearch, reverse-engineering, asset-pipeline and game-automation
  were not re-run: this piece recovers no new format and owns no native testing. The recovered
  schema it drives is the pinned `notes/Research/SOE Transport Notes.md` evidence, and the
  community-reader-first rule was satisfied there already.
