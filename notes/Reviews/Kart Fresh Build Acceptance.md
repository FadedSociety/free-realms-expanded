# Kart fresh-build acceptance

v14 source/release and focused native boost acceptance passed. Clean native022150 verifies a diagnostic pickup, Alt and Ctrl activation, four refill acknowledgements and the first authored checkpoint. Whole native three-lap results/reward/Close/reload acceptance remains open. See [[Kart Native Driving Failures]] for exact capture evidence and [[Kart Native Boost Acceptance]] for the current checklist.

## Native test sequence

1. Launch `../lab/reports/kart-playable-v14-20261009/start-playtest.ps1 -Seconds 1800` only after the release receipt is frozen. This copies the supplied client into an isolated run; use the package's character save, never the campaign save.
2. Select Capo Indigocauldron and enter the world. Close News. Enter `realm race` in local text chat. Leave Go unpressed until the driver is ready; click Go once, then wait for the countdown. A transient second Go panel disappears automatically after the original entrance request. Clicking it again can use a queued item after the race starts and confound keybind evidence.
3. Drive using the original client's acceleration/steering controls. Jump was previously verified by the owner. Test Ctrl and Alt separately after an item is available. Confirm refill acknowledgements and activation replies in the capture; a HUD icon alone does not prove use.
4. Drive through original pickup positions. Record collision request, exactly one grant/removal per live box and native item feedback. Current reconstruction grants boost type5 only. Other retail effects/random odds and respawns are not implemented.
5. Complete three ordered laps. Observe the native HUD at2/3 and3/3; compare captured checkpoint requests to the persisted ordered cursor. A private captured-route replay already proves93 accepted gates and once-only payment, but it is not a native full-race result.
6. Observe the results screen and toast. Expect the explicitly reconstructed reward50coins and25Kart Driver XP, exactly once. Check server banks and displayed values; do not substitute a manually injected finish or currency packet.
7. Use ordinary results Close. Verify world return and reward persistence, then exit the client through its menu. Let the harness finalize hashes/CRCs and stop its owned services.
8. Restart with the same disposable save, without resetting it to initial-character.json. Verify the bank/XP map and no repeat payout. Final acceptance includes this native reload.

The lead's UI tool supports taps/chords but has no held-key interface for sustained acceleration plus steering. Human driving supplies that input; the lead retains capture, inspection, bank checks and evidence review. Any failure returns to a scoped worker with exact frames/timing/source fingerprint. At most five workers may be active, and completed children are interrupted.

## Evidence boundary

The supplied2009Tangetrack video is historical: eight racers andfive laps. Its HUD/pickup cues guide comparison but do not establish settings, keybindings or reward balance for client1.910.1.530630. Solo three-lap v14,3m gate thickness, movement credit, fixed boost drops and50/25 balance are local reconstruction choices. Ordinary NPC/menu entry, rivals, other tracks and item types1..4 remain separate work.
