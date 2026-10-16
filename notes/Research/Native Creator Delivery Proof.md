# Native Creator Delivery Proof

2026-10-08 UTC. Run `client-baseline-20261008T065152Z` successfully created and selected Bennett Eaglebeach through the native creator. Its delivered character payload matches the supplied compiled C# serializer byte for byte. The client did **not** reach World Ready: its first profile load requested missing `human_m_head_caucasian_08.dds` and received HTTP 404. This proves creation, persistence and delivery, not a playable opening world.

The run used Rust gateway SHA256 `447161067beaa33fc82f8c62d85deaf984e9dcbdf874b3f2e4dc7e95dccc6d9e`. It predates the typed opening journal integration in [[Native Opening Quest Journal]]. Comparing it to the subsequently rebuilt gateway would give the wrong expected journal. Its 25 inherited research quests are preserved in this oracle, and are explicitly not accepted as the new campaign journal.

Independent offline reconstruction filtered `manifest.frames` to `service == "gateway"`: 985 gateway frames, with 70 login frames excluded. All 985 saved frame lengths and hashes match. The negotiated three-byte CRC verifies for all 983 frames after handshake. Established zlib/control-bundle/reliable-fragment/application-bundle helpers reconstruct both directions without outstanding fragments or queued sequences. The resulting full self message is 161,293 bytes, SHA256 `e6455df8f61aa350215252d30c1bc0395b47331fa848da889a69cec873f662be`.

The independent oracle starts from the original `PacketSendSelfToClient.json`, applies the saved identity, position, profile progression and inventory as typed JSON fields, and invokes `Gateway.Login.ClientPcData.ReadFromJSON` in the **supplied compiled Gateway.dll**, SHA256 `7e53bc376e2fbdd14480c5c8c7c2d6ab5f37842ee6009896789bf47e84e3ba48`. Every one of the 161,280 serialized payload bytes equals the actual reassembled self payload. This includes all preserved surrounding fields and the trailing data, not only the inspected prefix. Original serializer compatibility was already established against the complete original captured self; see [[Native Opening Quest Journal]].

The parsed prefix confirms GUID1, model1, Caucasian head ADR, shaggy hair, hair color10, eye color51, `skintone8`, skull face paint, empty customization, five appearance IDs `[1,7,8,1,0]`, Bennett/Eaglebeach and zero coins. Position is exactly the saved f32 vector `[-1904.883056640625,-39.709800720214844,412.6023864746094]`; rotation is zero. The compact serialized saved identity produces the stored identity fingerprint exactly.

All ten inventory records equal the snapshot, including all instance fields and tints. The first seven equal the current fresh opening inventory. Three additional creator items are equipped on profile1:

| Slot | Instance | Definition | Tint | Creator clothing |
| --- | --- | --- | --- | --- |
| 5 | 8 | 28721 | 235 | Work boots, `freestyle-leatherlaces-L1` |
| 4 | 9 | 17975 | 235 | Baggy pants, `freestyle-baggyjeans-L1` |
| 3 | 10 | 1026 | 229 | Hoodie, `freestyle-tanundershirt-L1` |

The delivered opcode35/subtype37 packet equals all 1,632 prepared serialized item definitions exactly. Joining those delivered definitions to the emitted self equipment and item records reproduces the saved creator attachment model, texture, `dyetint`, tint and composite effect fields. No separate class activation packet or visible world outfit is proved before Ready. Saved profile1 is active, profiles1/43 are unlocked, and all 17 opening stages remain unaccepted with zero counts and no discoveries.

The login log independently records opcode5 `created:true` and opcode7 `selected:true`. No native Ready6 or Loaded10 request appears in the gateway application stream. The client log contains neither `World Ready new loc=` nor `WaitForWorldReady: waiting complete`; it records the repeated failed `/642/human_m_head_caucasian_08.dds.z?2961482716` request. The original and current public manifests both specify CRC2961482716 and compressed size8996 for that texture. Correct `.z` delivery is established separately in [[Native Asset HTTP Delivery]].

Private reproducible evidence:

- [Wire proof](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/runs/client-baseline-20261008T065152Z/analysis/created-character-wire-proof.json>) and [transport reconstruction](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/runs/client-baseline-20261008T065152Z/analysis/gateway-decode-proof.json>).
- [Offline SOE reconstruction helper](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/analyze-native-created-character.py>) and [compiled serializer comparison helper](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/verify-created-self-oracle.py>).
- Reconstructed binary, expected JSON and compiled serializer output remain under that run's `analysis` directory, outside Git. No keys or login ticket literals are printed by the helpers.

Re-run from PowerShell:

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/analyze-native-created-character.py'
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-created-self-oracle.py'
```

Both helpers passed on 2026-10-08. They read saved input and run only the compiled serializer oracle; they do not launch the game, change shared gateway/harness code or send input. Native world entry, the opening/dungeon playthrough, and a restart remain separate acceptance gates.

## Supported second identity: actual world and first quest

Run `client-baseline-20261008T072258Z`, release SHA256 `5063442e993cae0b7af2d69223b6b6885f03a5c24d5713c40311d8ad64bcb52d`, creates **Capo Indigocauldron**, selects it and reaches native World Ready/Initialized-Client. This is a new supported appearance (`skintone3`), preserving Bennett's earlier failed identity/save. The saved identity, rather than shorthand session labels, establishes the name.

Independent reconstruction verifies all4,836 gateway frame hashes and4,834 post-handshake CRCs, excludes26 login frames, and completes reliable reassembly. The full153,375-byte self has SHA256 `3b24ccc83ce7eece4f2ba4b8c5956c061d57242983151a02ee678e52f00c9654`. Its entire153,362-byte payload matches the supplied compiled Gateway.dll serializer after applying the saved identity, fresh inventory/profiles, configured initial position and **empty native opening journal**. The root contains questcount0/completed0/selected-1/AutoSwitchfalse/tail10,30. All25 inherited research entries are absent. All10 items and1,632 delivered definitions match; three creator slots bind to the saved clothing/tints.

Actual native Jobs request26/13 selects Brawler43. Profile blocks in emitted38/21 activations for profile1 then43 exactly match the original compiled serializer; those packets additionally carry trailing attachment data. The first native Darkthorne menu selection accepts reconstructed Hero's Journal910001. Its emitted49/3 record is327bytes (334bytes including base header/selectbool), independently matches the supplied compiled serializer and parses completely: overallprogress0.5, goalstates[2,1], currentcounts[1,0], required[1,1], select=true. Brawler was already selected before acceptance, so **no49/9 counter-update packet is emitted in this run**. This establishes an add record containing progress, not a live incremental-update retest.

The final snapshot persists active43,10coins, discoveries[1,3], profileexperience25each for1/43 and the firstquest accepted/uncompleted/unrewarded with[1,0]. Remaining16 opening stages are unaccepted. World Ready and native Loaded10 are corroborated by client log/application stream. The quest's turn-in, original crafting/minigames, complete opening chain and dungeon are not established. The first reconstructed class-selection goal has localeIDs0; native text may be blank despite correct state/count bytes.

Private report: [supported creation and quest wire proof](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/runs/client-baseline-20261008T072258Z/analysis/supported-creation-quest-wire-proof.json>). Helper `lab/reports/verify-supported-native-creation.py` invokes the original compiled serializer; `analyze-native-created-character.py --run client-baseline-20261008T072258Z` reconstructs transport. Both pass. No launch/input/shared-code edits were made by this independent verification. Later restart073831 and the user's combat report are separate evidence; see [[Native NPC Motion and Animation]].
