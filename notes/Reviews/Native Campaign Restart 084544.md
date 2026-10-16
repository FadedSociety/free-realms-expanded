# Native campaign restart — 084544

Independent read-only review, 2026-10-08 UTC. The actual campaign reload preserves the completed cave reward and pending Pappy objective, and the original quest book visibly renders the restored state. A second click on the Other Quests bookmark is currently needed after its first blank page flip.

## Saved and native state

The run's starting snapshot is byte-identical to the preserved post-cave campaign `lab/state/opening-supported-after-native-cave.json`, SHA256 `60198b4566d6fcc42bf7ae5d2167b19d6a62a38e09ab679c17cb930d90126a0a`. This run uses `opening-supported-character.json`; it is not the synthetic combat controls branch.

All existing gameplay sections are exactly preserved: **70 coins, eleven item entries, rewarded encounter411,1000HP, no active encounter, quest910002 accepted with counters[1,0] and pending turn-in**. Hero's Journal910001 stays rewarded. Profiles, discoveries, old research domains and identity fingerprint are unchanged. Both identity files are byte-identical. The only added gameplay field is the compatible optional default:

```json
{"smelting":{"version":1,"next_instance_id":77000001,"completed":0,"active":null}}
```

The full153,719-byte self payload is byte-identical to output from the unmodified supplied C# serializer with that saved character/inventory/journal. Native quest title4419563, description4419564 and selected ID910002 agree. Goal1 is state2/current1/required1; goal2 is state1/current0/required1. The optional smelting save field adds no native self field. The independent decoder verifies all2,258 recorded frame hashes,2,237 gateway post-session CRCs and complete reliable reassembly (2,239 gateway frames;19 login frames excluded).

World Ready restores `[-1943.504883,-37.382439,371.920135]`. Later authorized navigation changes only position/facing; ending save values match the last native125 packet exactly as f32. No new rewards, damage results, quest updates or zone transitions are emitted. This verifies persistence and UI, not new quest completion.

## Visible journal acceptance

Independently viewed `quest-tab-second-click_small.png`, SHA256 `ac8ae876ce32585306640335d18ecff1bf0d243168a94b8b9628da000a0009db`. It clearly displays **Other Quests1/30**, **The Last Best Hope**, the original Darkthorne/Heartseed Chamber description, a green completion check for **Defend Darkthorne from the Gloam**, and an active marker for **Talk to Pappy Farnum**. This matches the preserved snapshot and native self objective counters.

The first bookmark click left even static page labels hidden. The same tab's second click shows the original MiscQuestPage while bypassing its page-flip hide branch. Peer proof `lab/reports/journal-other-quests-rendering-audit.json` records the original Lua/AS2 control flow. No arbitrary quest visibility flags or client movie patches were needed. The exact first-animation failure remains unresolved. Live objective updates still need their own visible acceptance.

## Reproduce

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-native-campaign-restart.py'
```

Private proof: `lab/runs/client-baseline-20261008T084544Z/analysis/native-campaign-restart-proof.json`, complete compiled self oracle, decoder output and original screenshot. Release SHA256 `c2380ee2434ccbfcc15176f14a72cb35062aa06dbea1a36e7103491112480950`. The reviewer made no native inputs, save changes or shared gameplay edits. Root's authorized UI/navigation inputs are recorded separately in the run.
