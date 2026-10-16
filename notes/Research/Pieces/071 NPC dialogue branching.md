# 071 NPC dialogue branching

Status: recovered the original base-command dialog family, the recovered response-option menus and
the per-phase Cleetus Farnum conversation lines; a bounded Rust model of that contract plus a
definition validator is implemented and tested. No native client run was made by this worker, so
nothing here is playable-accepted.

Queue item: `notes/WorkItems/071.md` (acceptance: original dialogue/menu/choice definitions and
quest phase handling). Worker: crack_builder, DeepSeek V4.1 Flash, 2026-10-09 UTC.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_071.rs` | Recovered dialog constants, the definition schema, quest-phase selection and the two Cleetus Farnum definitions |
| `prototype/osfr-tools/tests/piece_071.rs` | 12 focused tests; `#[path]`-includes the module until the lead registers it in `src/lib.rs` |
| `scripts/pieces/piece_071.py` | Read-only extractor/checker over the pinned lab sources; emits `dialogue-definitions.json` + `provenance.json` |
| `tests/pieces/test_piece_071.py` | 22 stdlib tests (fixtures + real-source checks) |
| `../lab/reports/piece-071-20261009T043725Z/` | Private evidence bundle: RED/GREEN cargo logs, checker runs and the emitted definitions |

## Original facts

### Dialog command family

- The supplied server (`lab/decompiled/server/LibSOE/SOE/BaseCommandPackets.cs`) names
  `CommandPacketShowDialog = 3`, `CommandPacketEndDialog = 4`, `PacketDialogResponse = 6`,
  `CommandPacketPlayDialogEffect = 28` in the `BaseCommandPacket` enum.
- Two pinned community repositories carry the same family inside tunnel opcode **26**:
  `FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs` (sub-opcodes
  3/4/6/28) and `OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs`. Both are
  source-level evidence, not executed readers.
- Preserved native constructors in `lab/decompiled/client/FreeRealms/pseudocode.c` (95 MB Ghidra
  dump, not hashed; line numbers below are from that file):
  `FUN_009f5890` (line 1248527) builds `PacketDialogResponse` with opcode 6 and its **single i32
  defaulted to -1**; `FUN_009f5760` (1248468) builds the field-less `CommandPacketEndDialog` with
  opcode 4; `FUN_00a99bd0` (1374925) builds `CommandPacketShowDialog` with opcode 3;
  `FUN_00a9ad20` (1375903) builds `CommandPacketPlayDialogEffect` with opcode 28.
- `CommandPacketShowDialog`'s constructor fills two strings, three 4-float vectors and several
  booleans. That shape is *not* a proven byte layout and is deliberately not modelled.

### Response menus

- Supplied `UI/UiModules/Main/wndDialog.xml` is the `Conversation` window (sha256
  `77cca6eb95be923e15f3924f2f233ad0c0e22c70af560b21011ac062f4d2ca13`) with response buttons
  `btn0..btn4` - five options, no sixth.
- Preserved client Lua `lab/decompiled/client-ui/ScriptsBase.lua` (sha256
  `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`):
  `DialogHandler:SelectOption(opt)` forwards a clicked response to `Ui.DialogSelect(opt)`, the
  dialog context quits through `Ui.DialogEnd()`, and the NPX-tutorial path pushes a context that
  auto-selects option 0.
- Quest offer (`QuestHandler:AddDialogResponseNodes`, Lua 15669):
  `AddResponseNode(Ui.GetString("questYes"), 300, 17, 1)` and, only while the dialog is
  declinable, `AddResponseNode(Ui.GetString("questNo"), 302, 18, 2)`. The tutorial `noDecline`
  state removes option 2 at runtime; the definition keeps both.
- End screen (Lua 15496): exactly one node,
  `AddResponseNode(Ui.GetString("questThanks"), 300, 17, 3)`.
- `Main_wndQuestStart_swfQuestStart_OnUserEvent` maps option 1 to `QuestScreen.AcceptQuest()`, 2
  to a plain hide; the end-screen handler maps 3 to `QuestScreen.EndQuest(true)`.

### Labels and default text

From the supplied locale export (`reports/locale-id-to-text.json`, sha256
`f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`) and
`Resources/CodeStringMappings.txt` (sha256
`1da9690fcc034657a1cd980cd96fc18cdc143a2fcf2a4373f8deccd032353e5a`):

| Code string | Locale id | Text |
| --- | --- | --- |
| `questYes` | 71313 | `Yes, I accept!` |
| `questNo` | 71314 | `No! I decline.` |
| `questThanks` | 1911 | `Thanks!` |
| (default NPC line, `Ui.GetString(8125)`) | 8125 | `Can you help me?` |

### Per-phase NPC lines (community evidence)

The read-only wiki corpus (`references/freerealms-wiki/wiki.sqlite`, sha256
`b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`) carries the Cleetus Farnum
step lines. Community text, CC BY-SA, revision-pinned:

- `Quest:Your First Weapon`, revision **27815**: offer, in-progress and turn-in lines; next quest
  `Quest:Fighting off the Pack`; coin reward 16.
- `Quest:Fighting off the Pack`, revision **28955**: offer and turn-in lines only; next quest
  `Quest:Eviction Notice`; coin reward 18.

Line order maps to phase by the recorded step count - one quote is the offer, two are offer +
turn-in, three insert the in-progress line in the middle. Any other count is rejected by the
extractor rather than guessed. The wiki keeps the original U+2019 apostrophes; the module uses an
ASCII apostrophe (`That's`, `you'll`), which is a labelled normalization, and the emitted
definition keeps the wiki revision beside every line.

## Reconstructed policy (not historical)

- The definition document schema (`DialogueDefinition`/`DialogueNode`/`DialogueChoice`), its
  evidence tags and every validation bound.
- The `quests::QuestProgress` -> phase mapping: no record or `!accepted` is
  `NotAccepted`; `accepted && !completed` is `InProgress`; `completed && !rewarded` is
  `ReadyToTurnIn`; `rewarded` is `Rewarded`. The mapping reads existing engine flags only.
- The rule that a `Conversation` node without a menu is closed by the server
  (`CommandPacketEndDialog`) instead of sending a response index.
- The two assembled definitions themselves: they are wiki lines arranged on the recovered windows.
  No original node graph, branch condition, numeric quest id or effect list is claimed.

## Evidence gaps

- The original server dialogue database did not ship, so no original node graph, branch condition,
  original quest id or choice effect list is recovered. `original_quest_id` stays `null`; the
  supplied journal binds 13 opening ids (2513, 2514, 2521-2530, 2566) without titles and the wiki
  pages carry no numeric id.
- `CommandPacketShowDialog`'s exact payload layout (strings, three 4-float camera vectors,
  booleans) is not established; the module therefore never serializes a dialog packet.
- The community `InteractionData` layout (`Id, IconId, ButtonText, Type, Param1, Param2,
  TooltipId`) declares a `DescString` field it does not serialize; how a conversation is offered
  in the interaction list is therefore only partially recovered.
- `Quest:Fighting off the Pack` records no in-progress line, so that phase has no node and
  `select_node`/`respond` return `None`/`Err` instead of inventing text.
- Related but out of scope: the client binary also contains a TradingCardsClient
  `CustomChoiceDialogState` (`..\common\common\state\CustomChoiceDialogState.cpp`, up to three
  buttons, per-button effects, `#targetN#` substitution) driven by `CommandObject_CustomChoiceDialog`
  and `CommandObject_SetPhaseDialog`. That is the card-game choice system, not the MMO NPC dialog
  path, so it is recorded here and not modelled.

## Verification

```
prototype/osfr-tools> & '..\..\scripts\cargo.ps1' -CargoArgs @('test','--test','piece_071')
  test result: ok. 12 passed; 0 failed
prototype/osfr-tools> & '..\..\scripts\cargo.ps1' -CargoArgs @('build','--lib')
  Finished `dev` profile (exit 0)
project> & '..\lab\tools\wiki-env\Scripts\python.exe' -B scripts\pieces\piece_071.py --check
  piece 071: 0 error(s)  (17 evidence checks, rust-binding agrees)
project> & '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_071.py" -v
  Ran 22 tests ... OK
```

RED evidence: `../lab/reports/piece-071-20261009T043725Z/red-cargo-test-piece_071.txt` (module
missing, exit 101). GREEN and checker logs sit beside it. The checker's `rust-binding` row fails
if a module constant or embedded line drifts from the pinned sources.

## Integration and native gate

- Lead-owned: register `pub mod piece_071;` (or the `restoration::piece_071` path chosen by the
  lead) and drop the `#[path]` include from the test. No shared file, manifest, content or save
  file was touched by this item.
- The module never mutates quest state, grants rewards or writes packets: it returns a
  `DialogueReply` and the existing quest/gameplay owners apply it.
- Native acceptance still required: open a real NPC conversation in the supplied client, confirm
  the recovered window/option behaviour, then exercise the offer -> accept -> objective -> turn-in
  chain against a running server. Nothing in this item proves the native dialog renders.

## Skills applied

RTK (normal commands first), Ponytail (reuse the preserved client evidence and stdlib; no new
dependency, no packet fabrication), Caveman (terse updates), game-recon (identifying the supplied
client/server artifacts), game-research-websearch and reverse-engineering (pinned wiki revisions,
native constructor extracts). asset-pipeline and game-automation were not needed: this item has no
asset output and does not own the shared game window.
