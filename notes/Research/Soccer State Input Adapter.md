# Soccer State Input Adapter

## Scope

Lab-only candidate adapter between the recovered sub-14 soccer state request
(`u64 player guid, u8 state, u8 aux, u32 word, f32[4] x3`) and the existing
server-owned `soccer_match` intent model. It is **not** registered in the
production crate (`prototype/osfr-tools/src/lib.rs` is unchanged), makes no
save/content/UI change and adds no native acceptance claim. Owned artifacts:
`lab/reports/soccer-state-adapter-20261009T174320Z/` (crate, tests, logs,
`receipt.json`) and this note.

## Evidence split

| Class | What | Source |
| --- | --- | --- |
| Original | vecA is the action direction (`player+0xF0..+0xFC`, wire `0x0E`); aux is the quantized charge (`player+0xA4`, wire `0x09`, `aux / 255`); the wire word is the `player+0xB0` state-change counter; vecC is `player+0x20..+0x2C` | `notes/Research/Soccer Kick Direction Binding.md` and the independent `notes/Reviews/Soccer Direction Semantics Review.md` (`e9b141e7…`), primary `FreeRealms.exe` (`26cc1c52…`) and `pseudocode.c` (`f84a5867…`) |
| Caller-supplied | field bounds, physics values, and the pass target: the authority-facing id must be the sender-verified teammate, never guessed from the unnamed `player+0x1C == 3` selector | `soccer_match::Authority::register` and the adapter's `target` argument |
| Reconstructed | `max_power` scaling of the proven charge, the `charge_floor`, the shot-only kick policy | labelled in `src/adapter.rs` and in the tests; not retail |
| Missing | a captured kick/pass sample, the local `+0x20/+0x28` writer, `FUN_00cd97d0` vecB consumer semantics, the `+0x1C == 3` selector name | listed as limits, never guessed |

## Mapping

| Wire field | Adapter use |
| --- | --- |
| `state` (payload `0x08`) | `classify_state` (kick 16-26/31-34, pass 27-30); must equal the typed `StateRequest.action` |
| `aux` (payload `0x09`) | `charge_from_aux(aux) = aux / 255`; `power = charge * policy.max_power` |
| `word` (payload `0x0A`) | advisory only; the transport sequence is the only sequence |
| vecA (payload `0x0E`) | `direction = [x, y, z]`; `w` is dropped |
| vecB (payload `0x1E`) | unused by this candidate |
| vecC (payload `0x2E`) | unused; never changes a position |

The adapter builds the existing server-owned
`Intent::Kick { player, sequence, kind, direction, power, target }` and routes
it through `Authority::apply(sender, now, intent)`, so the authority keeps
possession, the ball, the clock, the score and every bound.

## Policy (RECONSTRUCTED)

`AdapterPolicy { max_power, charge_floor, min_direction_norm }` is validated
before it shapes an intent: `max_power` and `min_direction_norm` must be finite
and positive, `charge_floor` must be finite and in `0..=1`. A kick state may
not carry a caller-supplied target, and a pass state without an explicitly
verified teammate target is refused. Native `aux = 0` `KickRight` may be a
legitimate uncharged kick; this candidate's floor refuses zero/near-zero
charge and labels that as its own narrow policy. Zero power that reaches the
authority is still refused by the authority's own `(0, max_kick_power]` bound.

## Verification

Suite: `lab/reports/soccer-state-adapter-20261009T174320Z/tests/state_adapter.rs`,
run with the private toolchain
`scripts/cargo.ps1 test --manifest-path <private Cargo.toml>`.

| Run | Result | Log |
| --- | --- | --- |
| RED | exit 101, `error[E0432]` unresolved adapter imports | `logs/red-final.txt` |
| GREEN | exit 0, 14 passed / 0 failed (+ 0 doc tests) | `logs/green-final.txt` |

The suite exercises actual ball motion: a shot along vecA gives ball velocity
`(4, 0, 0)` with an empty possession, and a following `advance` moves the ball
along `+x`; an `aux 128` charge yields `(128/255) * max_power` on the `+z`
axis. It also covers invalid direction, invalid policy, mismatched
action/state, spoofed sender, replayed transport sequence, power above the
authority bound, pass without a verified teammate target, kick with a pass
target, advisory word handling, server-only packet refusal (GoalMade 2,
UpdateSoccerBall 4, ImpulseBall 13) and full-snapshot equality after every
refusal. All bytes are synthetic; no raw capture is embedded.

## Source pins

The candidate was compiled against the root gate source
`3cc6652b0da68542748c0af28fa9ef60abf5c6554e4b0f4dce5e4d3ced76ddfc`
(`mining.rs`), identical in behavior to the independently accepted Mining
`8b32d7c5cf646f794d73b4f4426ecd053f5273f3cf63a419429e5512d625c83d` except one
documentation anchor. Pinned dependencies: `soccer_match.rs`
`33f642e2…`, `soccer.rs` `c68bb857…`. Artifact hashes are in `receipt.json`.

## Limits

No native acceptance: no real client kick/pass was driven and no ball motion
was observed in the client. The physics, power scale, zero-charge floor and
shot-only policy are reconstructed. The pass target must come from the
caller's server-verified roster; the unnamed `+0x1C == 3` selector and the
vecB consumer `FUN_00cd97d0` remain unresolved.

Cross-links: [[Research/Soccer Kick Direction Binding]],
[[Research/Soccer Kick Pass Wire Mapping]],
[[Research/Soccer Match Authority]],
[[Soccer Direction Semantics Review]],
[[Soccer Native Adapter Readiness Review]].


## Lead independent behavioral verification

Thirteen additional lead-authored tests pass against frozen adapter11d291, with actual shot/pass ball motion and full-snapshot conservation. Policy max_power is finite/positive only; the authority bounds the resulting aux/255 * max_power. Scale100/aux10 succeeds, while aux255 exceeds the power bound and is refused. The earlier RED receipt proves missingAPI compilation only, not test-first behavioral proof for all14tests. See [[../Reviews/Soccer State Input Adapter Review]]. Candidate remainsprivate, unregistered and native-untested.
