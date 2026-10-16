# Native inventory repair — 173814

The native client now restores the inventory after login. Independent screenshot inspection confirms the populated All Items list, wallet 120, Strong Arm Stew quantity 1, Student Chef Hat quantity 1, both hats in the Hats category, and the Chef Head gear choices. The equipped Student Chef Shirt renders in the preview.

Run: `client-baseline-20261008T173814Z`. Release: `eabbceee32e1a44c63521e9b9a5988aabcc1271a7b431c8df92526538715b8c0`.

The initial checkpoint is exactly the finalized 172324 checkpoint: 16 inventory instances, Chef XP 25, stage 4 completed/rewarded, Cooking completed once with no active game, and stage 5 unaccepted. Every byte of the 153542-byte original serialized self payload matches the captured startup self.

The actual startup catalog at frame 665 is 311622 bytes: header 35/37, payload length 311614, count 1647. The complete packet matches the independently compiled community writer, SHA256 `206ba7cb5006f5bcac1684f3746a775de558a9e0f151ec41b9ed01a3cbd4c9ce`. This corrects the previously demonstrated missing four-byte length prefix without changing catalog content or ordering.

Proof: `lab/runs/client-baseline-20261008T173814Z/analysis-live/native-inventory-visual-gate-proof.json`; reproduction: `lab/reports/preserve-native-inventory-173814.py`. Screenshots and the original self/catalog proof hashes are bound in that file. This is a preserved startup gate; later Miner progress is separate. Transport remains provisional until the run is finalized. The reviewer changed no source, gameplay saves, client movies, or native inputs.
