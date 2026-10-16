# Mystery Chest restoration

October 9, 2026. Original Windows client 1.910.1.530630. Inventory chest opening now works against the Rust gateway in a separate private test package. This does not establish explorer-world-chest interactions or original retail loot distribution.

## Intake and recovered behavior

The four supplied C# files were copied and hashed before analysis. Pristine files, SHA-256 inventory and native-reader evidence live in `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\mystery-chest-intake-20261009T192906Z`. The use handler consumes one owned Type28 chest and one Mystery Key definition78137, then saves before inventory updates. It does not grant a reward itself. The supplied check handler always reports success and does not check Type28 or key possession; our implementation verifies both.

Native constructors establish inventory42/16 Check and42/15 Use, each with one i32 instance ID. Server38/30 carries five i32 values: item, marketplace category243, bundle5021, title436474, body436472, followed by success byte. Inventory category408 is a different namespace. Lua routes success1 to the ordinary confirmation and Use request. False routes to the original marketplace UI; no purchasing backend is implemented.

Reward50/1 has a u16 root and u8 subtype. Reward kind16 selects the original Mystery Chest item panel. One generic item entry works for the Kangaroo mount definition11584 with icon7665, icon tint553, name378 and quantity1. Native x86 reader accepted all111 body bytes with no error and appended one correct item entry; 392 instructions executed, only allocation and empty-string infrastructure stubbed. See `native-bundle-reader-probe.json` and `verify_native_bundle.py` in the report. Ghidra's pseudocode incorrectly removes the entry loop; original assembly and executable reader behavior establish it.

The seven supplied IDs are rewards in the pinned catalogue:78142 Hoodie Jacket,78143 Loose Skinny Jeans,78144 Tight Sneakers,78139 Striped Beanie Cap,79370 Precursor Hover Boots,79373 Molten Precursor Hover Boots,11584 Kangaroo. The pinned supplied/community definitions contain no Type28 chest definition. The test package therefore reconstructs reserved definition910100 using original chest name436407, description436408 and icon7666. This is explicitly a local reconstruction, not a recovered historical definition. Its only configured reward is Kangaroo; selection for multiple configured rewards is deterministic local SHA-256, not historical odds.

## Rust implementation and safeguards

`mystery_chest.rs` owns strict codecs; `mystery_chest_loot.rs` owns optional validated bindings. Existing serialized catalogue metadata supplies presentation fields without duplicating a full asset parser. Missing `mystery_chests.json` preserves existing packages. Load validates Type28, key availability in the catalogue, known distinct reward definitions and content bounds. Gameplay routes requests before generic inventory handling, preserves existing activity guards, and clears transient eligibility on session reset.

An eligible Check records one instance ID. Use rechecks ownership and key possession, consumes both on a cloned inventory, grants the item and builds the reward packet before committing. Grant/allocation failure leaves the whole inventory unchanged. Replayed Use does not open another chest from a remaining stack without another Check. This is a protocol ordering guard, not proof of human consent: the original client sends no separate Cancel packet. Existing gateway/store code saves the snapshot before queued replies are transmitted.

## Verification

- Codec worker:12 runtime failures against explicit stubs, then12 passes. Lead reviewed framing against original native constructors and reader.
- Initial gameplay RED mistakenly sent subtype14; it is retained but is not valid request regression evidence. A separate scratch replay restores the original gameplay file byte-for-byte (SHA d09d4e75...) and sends correct subtype16; it fails at the missing confirmation assertion. This corrected baseline proof was run after integration and is labelled accordingly.
- Ordinary whole Rust suite:1549 passed,0 failed,70 explicit private checks ignored. Source hashes stayed identical during that gate. Four new private gameplay checks were separately executed and passed: confirmation/atomic grant/replay/disk restore, missing key/forged item/malformed conservation, allocation failure rollback, and session reset.
- Afterward one inaccurate codec documentation sentence about subtype14 was corrected; executable logic stayed unchanged.25 focused inventory/codec tests and all4 private chest checks passed again. Frozen native binary73d8b311... was built before this comment-only correction. Gate-source and final-source hashes are preserved separately.
- Native first run `client-baseline-20261009T194318Z`: ordinary inventory Use showed original confirmation. Cancel preserved chest/key2/2. Reopened confirmation/Okay displayed the Kangaroo panel. Reopening inventory showed chest1/key1/Kangaroo1 and5coins. The finalized capture verifies1623 total datagrams,1604 gateway frames,1602 gateway CRCs and complete bidirectional reassembly. Exactly two42/16 requests, one42/15, two successful38/30 replies and one item50/1 reward. Reward instance13, allocator14. All other logged gameplay summaries were identical across the chest commit.
- Fresh native restart `client-baseline-20261009T194943Z`: inventory visibly retained chest1/key1/Kangaroo1. Whole gameplay save equality was checked against the first opening snapshot.1531 datagrams,1514 gateway frames,1512 verified gateway CRCs and complete bidirectional reassembly; no chest request or repeat reward. Both runs and their owned client/server/login processes stopped. Screenshot is `native-restart-inventory.jpg` in the report; finalized hashes/proofs are in `receipt.json`.

The user's active gameplay save and source client were not altered. Test identity and save are private report copies. One chest/key remains in this disposable branch. No source publication, external messages or game-asset redistribution occurred.

## Remaining work

Recover actual retail chest definitions, drop tables/odds and item entitlement rules; test other reward categories; implement/verify mount riding separately. Kind16 notification and inventory ownership do not prove riding. Explorer-world-chest prompts and discovery logic remain separate. Existing housing, racing and historical opening-area gaps remain open.

## Reproduce

From `%USERPROFILE%\Documents\Projects\CartoonMMO\project`, set `OSFR_CHEST_CONTENT` to the report's `test-content` folder, then run `scripts/cargo.ps1` with arguments `test --manifest-path prototype/osfr-tools/Cargo.toml --test mystery_chest_gameplay -- --ignored --nocapture`. In PowerShell use an argument array to preserve `--`. Original donor files and raw/decompiled assets stay outside Git. Native launcher exact command is in the report receipt; it uses the matching opening-mining-starter journal overlay and separate native identity/save files.

Related: [[Reward Toast and Deferred Bank Findings]], [[../Tasks]], [[../Current Progress]].
