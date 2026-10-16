# Native Loss Boss Display

Finalized loss run163829 visibly leaves the living Gloam Overlord target-health bar over the original TRY AGAIN title. The initial read-only community/native/wire investigation identifies an existing boss-registration cleanup packet. That investigation preserved Rust, movies, saves and the active native run; the later narrow source repair is recorded below.

Pinned Sanctuary PR77 revision `c5f123e67e246b6aeb3aa39f560ff1b5d31537ad` documents and serializes CombatPacketEnableBossDisplay as main32/sub9, then GUID(u64) and enable(bool). The source-only research provenance and exact file hashes are preserved. Existing Rust already writes this packet through `boss_display`.

Original native constructor00a2a030 sets subtype9. Parser00a2a8a0 reads the GUID into offsets0x10/0x14 and the enable byte at0x18. Dispatcher00a2c370 case9 invokes00a2b910. False clears the actor's boss bit0x80 and calls009852a0, removing its GUID from the boss registry and refreshing BaseClient.Bosses; true sets the bit and registers through00985200. Capstone confirms both original calls. This operation does not kill or remove the actor.

Original Lua HUD:hideMinigame already calls Target:Hide, so merely assuming the score path forgot a hide is incorrect. Target:onDS hides when BossesDataSource has zero rows, but can show again when it has a nonempty row. The precise later frame that re-shows the bar was not traced. Surviving registration and missing loss cleanup are directly proved.

| Final run | Boss display packets |
| --- | --- |
| Win163518 | true at59.156s; false at79.406s before terminal presentation |
| Loss163829 | true at34.609s; no false |

Boss GUID is5715144128442986768. Exact false vector is `2000090010f100004e45504f00` (13bytes). Existing Rust emits false only on a boss death, which explains the difference between killing the win boss and leaving the loss boss alive.

Proposed narrow next repair: emit the existing false packet for surviving active dungeon bosses during one-shot terminal-loss presentation, before results presentation. Preserve actor health/position, save state, actual Close and Retry behavior. Cover restored terminal loss as well. Existing Retry spawn emits true again; verify that with distinguishing ordinary and eventual native Retry checks. Do not substitute RemovePlayer or a fake death.

Acceptance remains pending: a fresh native loss must show TRY AGAIN with3 defeated/zero totals and no boss bar, remain open until actual X, return with preserved economics, and allow Retry to restore boss display. The previous win/loss lifecycle acceptance remains valid with this cosmetic limitation recorded.

Proof `lab/reports/native-loss-boss-display-proof.json` SHA256 `ae2ffd86166ed4d0eaf7592c795be37d68e31bb8d7d6837641fc8713e3c1736e`, reproducible with `trace-native-loss-boss-display.py`. It includes original source/executable/Lua/Rust/community hashes, ten native function extracts, machine bytes and both authoritative wire comparisons.

## Narrow source repair, October8

The existing once-only `present_results` path now emits `boss_display(guid,false)` for living configured dungeon bosses on loss, before score data/presentation. The ten-line addition changes no saved state, actors, health, positions, economy, completion or zoning. Win behavior remains identical. Restored terminal loss uses the same path; ordinary Retry initialization registers the bosses again.

A distinguishing regression first failed because the expected13-byte false packet was absent. After the repair, all15 ordinary encounter checks and both explicitly enabled private original-content/compiled-writer oracles pass (17 total); the two changed files pass formatting. The new regression includes a previously killed boss, surviving boss and surviving ordinary actor, checks exact packets and ordering, compares the entire saved state to the expected ordinary knockout, excludes RemovePlayer/rewards/premature completion, and covers duplicate suppression, restored loss and subsequent Retry registration/loss cleanup.

Patch proof `lab/reports/loss-boss-display-patch-proof.json` SHA256 `fcdc026b9ea8a66f2f4cb7463b76c6f6813cc4681c8beb6cbf61d479b6092f70`, reproducible with `check-loss-boss-display-patch.py`. Reversing only this addition reproduces the original evidenced encounter source SHA256 `1494b4b12b4fd22291ca1a1378fc3e436d579ad58ade197e86b57fea0f292faa`. Red/green/private/format logs are hash-bound in that proof. No release build, native input, save mutation or active-binary change occurred.

Independent combat-results reviewer accepts the patch without blocking findings: repeated all17 encounter checks, verified215 original instruction bytes/Capstone and source hashes, independently decoded the original win/loss32/9 packets, and checked the exact reverse source delta. Review `lab/reports/loss-boss-display-independent-review.json` SHA256 `cb42317573d8cafe014ee09284cc03d4827b99bd07da6da56bdabd60d615674f`. Fresh native visual loss/Close/Retry acceptance remains pending.
