# Kart native boost acceptance — October 9, 2026

Clean original-client run `client-baseline-20261010T022150Z` used accepted v14 release `f9f4af0b7d8278f9af8849d69082f69440963b44251ce3c15c803cfbef0d25b2`. All accepted source and test-log hashes were independently matched after capture. Full Rust gate: 1694 passed, 0 failed, 81 ignored; eight private end-to-end kart checks passed. No production source changed during native acceptance.

## Verified in the original client

- One Go click started the race; the transient second panel cleared without another click.
- Thirty-seven pickup actors spawned. Six collision requests against diagnostic handle1 resulted in exactly one type5 grant and one removal. This actor was deliberately placed half a metre from the grid; canonical retail placements remain separate.
- Four owned refill requests at 168.485, 201.797, 247.172 and 280.469 seconds each received an empty S33/23 response within 31 milliseconds. The first came approximately 3.281 seconds after native Go. The rejected fixed20-second policy is removed.
- Recorded isolated Alt_L at 219.563 seconds produced C33/19 and S33/20 type5, followed by 222 boost-control poses. Control_L at 228.594 seconds produced the same exchange and 226 boost-control poses. Key labels come from recorded UI actions; packets alone do not encode the key.
- Ctrl visibly produced flame exhaust and forward movement. Original purple question-mark actors were visible beside the first bend. This visual observation does not prove collection at those retail positions.
- One authored checkpoint was accepted and emitted a progress reply. No complete lap occurred.
- Coins stayed309; inventory and XP maps were unchanged. No race reward was claimed.
- Normal Escape/Exit Game/Yes closed the client with exit0 at486.532 seconds, before the600-second harness limit. Client/login/server stopped and their recorded PIDs were absent.

Final audit verified43,427 frame hashes and43,406 gateway CRCs with zero unchecked controls. Its38,174 direct driving messages include14 compressed messages and448 boost poses. Direct CRCs are a subset of the gateway total and must not be counted twice.

## Evidence and corrections

Private immutable receipts, applications and summaries: `../lab/reports/kart-playable-v14-20261009/client-baseline-20261010T022150Z/`. `clean-boost-receipt.json` binds key actions, exact request/reply frames, native boost bursts and unchanged accepted hashes. Reproduction: run `../lab/tools/wiki-env/Scripts/python.exe ../lab/reports/kart-native-refill-probe-20261009/verify-clean-boost-run.py` from the project directory.

Prior native021651 had an extra Go click after entrance; its item activation cannot establish a keyboard binding. The later Alt tap had an empty queue, so no request does not establish a broken Alt binding. The clean022150 run resolves both ambiguities.

The read-only v14 audit was corrected to recognize native direct framing and verify its seeded CRC before decompression. Fourteen compressed driving messages had been incorrectly routed to the SOE control reader. Their hashes, CRCs and decompressed lengths now pass the independent direct audit. This was an evidence-reader fix, not a runtime protocol fix.

## Remaining acceptance

- [x] Diagnostic native pickup, one grant/removal and boost use.
- [x] Native Alt and Ctrl activation separately.
- [x] Repeated native refill acknowledgements with queue cap3.
- [x] First authored native checkpoint progress and unchanged banks on interrupted race.
- [ ] Collection through original retail pickup positions.
- [ ] Full three-lap HUD:2/3,3/3, finish/results.
- [ ] Exactly one reconstructed50coin/25Kart Driver XP toast and bank delta.
- [ ] Ordinary results Close, world return, post-win reload and no repeat payout.
- [ ] Other item effects/types1..4, historical random drops/respawns, rivals and normal NPC/menu entry.

The solo3lap course, fixed boost drops,3m curtain tolerance and50/25 balance are local reconstruction choices. The supplied video is historical2009 gameplay, not authoritative evidence for this later client or reconstructed settings. Client-driven energy charging is recovered; server energy simulation remains unimplemented. See [[Kart Fresh Build Acceptance]] for the complete driver sequence.

## Canonical next drive prepared

The owner requested a fresh launch for human testing. New canonical run`client-baseline-20261010T030742Z` is open in the world with live capture and an1800-second harness bound. User input was detected while dismissing News; lead refreshed and stopped UI input so the owner retains control. Enter `realm race` in chat to load the original start menu. Canonical save was preserved rather than reset. No new acceptance claim yet.

Canonical v14 run `client-baseline-20261010T023328Z` is open at the original Wildwood Go menu. The race clock has not started. Content fingerprint is canonical`d626af55dd0d4c0d63ee731b8fe614ed93296030f649bc3edd4ec85ef1826631`, with the37 original captured pickup positions. Initial coins309, job48XP0; expected once-only reconstructed finish banks are359/25. Harness bound1800seconds; race bound600seconds starts after one Go click. Live capture is enabled. This is a prepared test, not a finished race receipt; its save must not be reset before post-win reload.

Normal NPC entry reader was returned once for incorrect method/guard descriptions, then corrected and independently checked: reuse `KartGateway::begin(character)` through a native admission branch in `handle_in_world`. The inactive inner handler already permits entry and has no chat-command test. The original offer actor is still unidentified; trainer/wiki/locale names alone do not establish an actor GUID. Exact report: `../lab/reports/kart-native-entry-route-20261009/README.md`. No production route was added.

Nonboost research identifies type1missile,3mines,4shield; native typed queue can carry them, but Rust content and saved-queue validation currently allow only5. Production extension must preserve typed restore and rollback and implement the world effects/lifetime. The lead rejected a toolbar-only port as sufficient gameplay. Research report/checker was returned for bounded single-pass citation reading, restore-scope omissions and clearer expiry/collision authority evidence. This has not changed the accepted runtime.

The corrected nonboost checker independently passes9 frames/160 counted subtypes/84 citations in5.7seconds. Shield expiry1 calls the same togglerfalse that use4 calls true; that is a static mapping, while native expiry/lifetime acceptance is still open. A hit packet must undergo server ownership/live-victim/finite bounded collision checks; a blind relay is not accepted.

Lead rejected the NPC followup's16-byte AddNpc classification: two candidate packets match the exact captured35/20 player-state bytes at15451/15496 by full applicationSHA256. They carry local playerGUID and flags0x401, not NPC identity/pose. The remaining actor mapping is unresolved; see `../lab/reports/kart-native-entry-route-20261009/LEAD-REVIEW.md`. Both research children finished and were interrupted. No production source or test count changed.
