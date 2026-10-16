# Delegated native acceptance audit

Lead audit, October 9, 2026. **The handoffs are not uniformly one in-game test away from completion.** Many workers delivered useful isolated modules, codecs and tests, but production entry, server authority, content and save bindings remain absent. This note supersedes broad interpretations of "ready for lead review". It does not undo previously accepted opening/cave/crafting evidence.

## All assignments accounted for

[[Delegated Task Readiness]] and its adjacent JSON contain 100 unique work-item rows: six initial systems, 93 subsequent pieces and one coordinator. Lead checked that every subsequent piece's listed module, Rust test, Python checker/test and research note actually exists. This is artifact completeness, not a fresh execution of every test or 100 playable features. Piece081 is included as `inventory::instance_ids` and has a real allocation caller. Most subsequent piece modules remain isolated test candidates; some cover subjects already served by other production code.

The corrected overlap counts classify subjects, not delivered features. Housing codec checks and launch-model tests are explicitly separated from native gameplay. Review v1 is retained as superseded evidence; its original "0/100 native-testable" conclusion and v2's first overbroad accepted-successor wording must not be used.

## Current readiness

| System | What is verified | Remaining gate |
| --- | --- | --- |
| Character creation, local opening and cave | Prior original-client acceptance: supported creator, 17 reconstructed local stages, cave/boss, ordered objectives and restart | Historical starting-area coverage and full retail quest set remain incomplete |
| Cooking, Smelting, Forging | Prior original-client rounds, results/Close, atomic recipes/rewards and persistence | Additional recipes and historical balance/content coverage |
| Mystery Chest | Prior native Cancel/confirm, fixed Kangaroo item reward and whole-gameplay reload | Retail chest/loot tables, other rewards and mount riding |
| Mining | Current native entry/render, pause/resume; observed manual/unattributed progress; timed loss | Full quotas/win, ordinary Quit/results/Close, payout/reload; reproducible held multi-tile input |
| Chef Harvesting111 | Current original job selection, offer/board, developer cancel/world return and unchanged-gameplay restart | Goal names/icons; gesture proof; ordinary Quit/Close; full win and atomic outputs |
| Housing | Current 121 focused Rust checks, 100 Python checks, 44 captured-body codec roundtrips | No127 routing, runtime lot/content or canonical atomic save; real inventory linkage and native placement/visit |
| Kart racing / derby | Current64 core checks; three private lobby integration gates | Vehicles, race entry/controls/finish, priced rewards/save; derby arena/damage/scoring recovery |
| Fishing | Current30 focused checks and corrected community serializer comparison | No138 routing, register/schools/bobber, bite/reel/catch authority or rewards/save |
| Soccer | Current focused authority/codec checks; prior private adapter review | Production match entry/roster/tick/ball update/end/return; native match not playable |
| Other delegated activities | Artifact-by-artifact inventory and individual missing gates in the100-row audit | Register and connect only reviewed slices, with actual authority/content/save/native proof |

## Native tests performed now

Frozen gateway `73d8b311da54d6e9cf8f51825c835ff8b9f39ef3737d035809ab429755b08ec2`; original Windows client `1.910.1.530630`. Disposable saves, existing identity, private `opening-mining-starter-20261009T180011Z` content and matching journal overlay. No production code, original inputs or active user checkpoint were changed.

1. **Mining `client-baseline-20261009T200750Z`.** Original offer/board loaded. A recorded Sky endpoint drag produced no accepted chain. Manual input occurred outside recorded test actions; root stopped input and asked, then resumed only after the owner answered "Continue testing it yourself". The13 chains/5400 score are therefore manual/unattributed progress, not automation-only proof. Final state Lost, collected[0,0], completed0, coins309. Native timer expired; launcher stopped at600seconds. No ordinary Close/world return or victory established. All other gameplay is unchanged except initialization of an empty Harvest state. Capture:5817 gateway datagrams,5815 verified CRC3, complete reassembly,19 login datagrams excluded.
2. **Harvest `client-baseline-20261009T202011Z`.** Selected Chef in the original Jobs panel, then entered with `realm harvest`. Farnum's Farm offer and original board render. Four baskets show unresolved `##farmingValue60...` labels and no visible produce icons; retained screenshot is `../lab/reports/delegated-native-audit-20261009T200750Z/harvesting-placeholder-goals.jpg`. A Sky drag across three grapes produced0chains/0score. The supported API supplies two endpoints but no held multi-point path; input sampling is an unresolved testing limitation, not an established server defect. Leave clicks and a double click did not exit; Escape opened the general game menu. `realm exit` showed TRY AGAIN, then returned to the world automatically. This is developer cancellation, **not ordinary Quit or results/Close acceptance**. Native Exit Game finalized the run. Final activeNone/completed0/next58000002; inventory byte-equivalent as a parsed structure, coins309; only Chef selection and Harvest instance allocation differ. Capture:2664 gateway datagrams,2662 CRC3, complete reassembly,17 login datagrams excluded.
3. **Cancellation restart `client-baseline-20261009T202622Z`.** Loaded the same post-cancel branch, entered the original world as Chef, with no running activity. Whole-gameplay equality is checked against run2 before/after in the lead receipt. This proves cancellation persistence only; no win or payout is credited.

Evidence and reproducible verifier: `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\delegated-native-audit-20261009T200750Z\verify_audit.py` and `lead-receipt.json`. Each stopped capture is hash/CRC checked and reassembled using the existing `analyze-native-created-character.py`; no live-prefix result is substituted for finalized capture proof.

## Findings returned

Findings were sent to the original kart/fishing/harvesting/mining tasks, housing025/026, housing migration028 and racing progression041. These are queued internal follow-ups with evidence and acceptance conditions; they do not mean fixes have already been implemented. Coordinator received two corrections to its readiness inventory and produced the corrected v2.

- Housing: connect127 and canonical crash-atomic save/inventory ownership before trying native placement. `127/2` placement selection is not an owned item identity. No lot/fixture runtime package exists.
- Kart: use the recovered race-entry contract and `zone_transition` preserving captured rotationw=0; current encounter zone writer forces1. Only lobby102/1 is connected. Keep finish/reward claims open.
- Fishing: useful pinned PS3/PC2009 serializers are now documented; their sub15 width differs and fixedTrout418 experiments are not a historical catch implementation. Close replay/Idle-result findings before adding reward authority.
- Harvest/Mining: repair or recover goal presentation and ordinary exit, establish a recorded native gesture, then complete quotas/results/Close/restart. Do not promote model victories or manual input to automated native acceptance.
- Soccer: current widths and private state adapter reviews supersede older wire findings, but a real match route remains missing.

[[Delegated Housing Readiness]], [[Delegated Racing Readiness]], [[Delegated Fishing Readiness]], [[Delegated Harvesting Readiness]], [[Soccer State Input Adapter Review]] carry the detailed gates. The Harvest reviewer repeatedly returned intent-only finals without artifacts; lead wrote the Harvest disposition directly. Those replies are not test evidence.

Next implementation priorities: finish the already connected Mining/Harvest native flows; then implement one complete housing or kart entry-to-save slice with lead-owned integration. The other isolated pieces remain available for reuse. No additional work is marked done merely because its test harness compiles.
