# SOE Protocol Rust Crate Assessment

Reviewed October9,2026 Pacific. Verdict: **worth retaining as a high-value community transport reference and candidate dependency**. Its likely substantial value is multi-session networking and transport verification. Our present blockers are mostly game application behavior and integration; this crate does not supply housing, quests, inventory, combat, minigames, databases or a whole game server.

The user supplied [docs.rs](https://docs.rs/soe-protocol/latest/soe_protocol/). Inspected published version2.3.0, commit `f1c01ad8fb0292bdb72d2cb27bc4f842c620728a`, official crate checksum `1e650ba4b1a5e6780bbe27332aba5f4a5a10fd17403cb70f29bc49fff8ced15f`. Repository formerlyyungcomputerchair redirects to [FabledRealmsProject/soe-protocol](https://github.com/FabledRealmsProject/soe-protocol). The published version is distinct from evolvingmain. Exact source/metadata/test evidence is retained in `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\soe-protocol-assessment-20261009T181451Z`.

## Coverage and project fit

The crate implements SOEv3 sessions, CRC0–4bytes, optionalzlib and RC4, negotiated UDP sizing, reliable fragmentation/reassembly/sequence handling, acknowledgements and retransmission, four reliable channels and an unreliable path, heartbeats/timeouts, a caller-clock/I/O-independent core and multiple-client multiplexer. Blocking sockets and optionalTokio adapters are provided. It draws on Sanctuary.SoeProtocol and OSFR references. Source defines protocolversion3 and enforces it during handshake; some older channel features do not establish everySOEversion compatibility.

Our `src/soe.rs` and `src/soe_send.rs` already implement bounded CRC/zlib/ordering/fragmentation/reliability, and the login/gateway executables supply session/runtime handling. Currentgateway runtime owns a single `Option<Session>`. Native FreeRealms captures negotiate CRC3 and our existing implementation has proven real client compatibility. A richer reusable multiplexer and referenceimplementation could reduce future network plumbing work, but is not a shortcut for the application/game systems.

## Executed verification

- Published checksum matched registry metadata before extraction; originalsource preserved.
- Private core-test copy passes76unit tests,8data-channel roundtrip/fuzz-style cases,1real loopbackUDP client/server test, and1doc compile: **86passed/0failed**.
- All Rustsource bytes match the published crate. Only the private copy's manifest omitted unconditionalTokiodev dependency so default/core tests could run.
- Original all-feature run failed in our Windows GNU linker tooling (missingdlltool, then an assembler subprocess issue after locating bundleddlltool). Tokio adapters are therefore **not locally verified**, and this is not evidence of a cratebehavior failure.
- No differential replay of our FreeRealms captures, native clientconnection or migration has been tested. No productiondependency/source/save/release changed.

## Concrete limitations

Reliable output currently usesgo-back-N: a timeout resends the unacknowledged window. SelectiveACK support does not mean selective retransmission. [Open issue1](https://github.com/FabledRealmsProject/soe-protocol/issues/1) proposes selective resends; source `channel/output.rs` confirmscurrentbehavior. Netstatus semantics are incompletely documented. Passing synthetic/loopbacktests does not establish our game's exact wirequirks or realInternet/multiplayerload behavior.

Published Cargo metadata and LICENSE sayMIT, whereas README saysGPL-3.0-or-later. Record that inconsistency rather than assigning a clean license verdict; clarify the exact version/source before reuse. See [published Cargo](https://github.com/FabledRealmsProject/soe-protocol/blob/f1c01ad8fb0292bdb72d2cb27bc4f842c620728a/Cargo.toml) and [LICENSE](https://github.com/FabledRealmsProject/soe-protocol/blob/f1c01ad8fb0292bdb72d2cb27bc4f842c620728a/LICENSE).

## Recommended project memory

Remember: soe-protocol2.3.0/FabledRealmsProject is a testedRust communitySOEtransport reference and potentialmulti-client foundation, not a completeFreeRealmsgame backend. Before replacing our proven transport, pinversion, clarifylicense discrepancy, replayCRC3/fragments/ACKs/sequencewrap/RC4 against preservedcaptures, verify native creation/world/minigame/results/reconnect parity, and keep rollbackavailable. No timeframe reduction is established by thisassessment.

This note records the assessment in project/Obsidian knowledge. No globalCodex memoryupdate or dependencyintegration was performed. Cross-links: [[SOE Transport Notes]], [[Verified Recovery Workflow]], [[../Current Progress]].
