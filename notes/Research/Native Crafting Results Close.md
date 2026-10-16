# Native Crafting Results Close

Read-only support for the Cooking finish/results investigation; original client methods distinguish a genuine results Close, an in-game Quit, and the automatic state-removal acknowledgment. No Cooking service, save, movie or native input was changed in this investigation.

| UI/protocol path | Original native behavior |
| --- | --- |
| `Ui.LeaveMiniGameOrEncounter` | Bound method00c172c0 calls008d2730→009bd240. It sends encounter41/109 only when native active-state byte+0x61 is true. It also destroys local minigame state and sends the closed-results notification below. |
| Closed results notification |0092f300→008a2eb0 constructs CommandPacketClosedMinigameEndScreen. Dedicated sender00925c90→writer0090ac70 serializes only two u16 values:26,42. Exact packet is four bytes `1a002a00`; constructor argument0 is not serialized. |
| `Ui.QuitMiniGame` | Bound method00c17470→00939100 routes a nonencounter state to009be8e0→009bdc60(6,-1,-1):39/6 with current instance,-1,-1. |
| Server39/19 StateRemove |009bf190 emits client39/7 acknowledgment while presenting/removing native state. This is not the later user Close. |

The original Lua `SetMiniGameEndData` selects `ScoreScreen:Show(status,9)` for type9. This supports preparing the original Cooking results page; it does not prove a complete native finish/retry lifecycle.

Additional source localization: PlayerStartMiniGame009beb70 assigns state+0x61 from its third parameter; ordinary JoinGame009bfdb0 passes0, while encounter path009bb2d0 explicitly sets the flag1. Those functions were inspected read-only; the exact current Cooking launch route/flag remains a separate acceptance detail, rather than treating MiniGameInfo's boolean fields as this flag.

Proof `lab/reports/native-crafting-close-proof.json` SHA256 `2f91cb89894117be3a405159c40e9c14c48849c845464294ba1bce28f4b49f2b`, reproduced by `trace-native-crafting-close.py`. It pins unchanged original executable/pseudocode/Lua hashes, community PR76 packet sources, exact method-table references, eighteen function extracts and original Capstone instruction bytes. It deliberately leaves runtime Cooking flag mapping and fresh genuine Close/Retry wire acceptance pending. No assumption that Cooking's results X sends combat41/109 is justified.
