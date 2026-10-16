---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# FRTW Game

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

282 accepted functions: 168 confirmed, 78 likely, 0 speculative, 36 unknown. Network-linked (`wire`): 0.

Subsystems: `ui.frtwgame.scaleform` (175), `game.frtwgame.display` (107)

### `game.frtwgame.display`

**Engine::ClonesWarsSetCampaignIDFromMMO** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01257200` | Engine::ClonesWarsSetCampaignIDFromMMO::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kSetCampaignIdDisplayAction registration tag; no arguments or state. |
| `01257210` | Engine::ClonesWarsSetCampaignIDFromMMO::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01257170` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::ClonesWarsSetCampaignIDFromMMO::vftable (EH frame first); when the int a… |

**Engine::FRTWEngineAttackDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254d00` | Engine::FRTWEngineAttackDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kAttackDisplayAction registration tag; no arguments or state. |
| `01254d10` | Engine::FRTWEngineAttackDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01254d70` | Engine::FRTWEngineAttackDisplayAction::VTableSlot2 | likely | Vtable slot [2] body: calls FUN_012e4830() with no arguments, passes its eax result as ECX to FUN_012eeb40(), then returns 0 (ret 8). |
| `01254c70` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineAttackDisplayAction::vftable (EH frame first); when the int ar… |

**Engine::FRTWEngineConsiderPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254bb0` | Engine::FRTWEngineConsiderPlayer::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kConsiderPlayerDisplayAction registration tag; no arguments or state. |
| `01254bc0` | Engine::FRTWEngineConsiderPlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01254b20` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineConsiderPlayer::vftable (EH frame first); when the int argumen… |

**Engine::FRTWEngineDeathDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254f50` | Engine::FRTWEngineDeathDisplayAction::FRTWEngineDeathDispla… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineDeathDisplayAction::vftable (EH frame first); when the int arg… |
| `01254fe0` | Engine::FRTWEngineDeathDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kDeathDisplayAction registration tag; no arguments or state. |
| `01254ff0` | Engine::FRTWEngineDeathDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01255050` | Engine::FRTWEngineDeathDisplayAction::VTableSlot2 | likely | Vtable slot [2] body: calls FUN_012e4830() with no arguments, passes its eax result as ECX to FUN_012eeb40(), then returns 0 (ret 8). |

**Engine::FRTWEngineDefendDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254de0` | Engine::FRTWEngineDefendDisplayAction::FRTWEngineDefendDisp… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineDefendDisplayAction::vftable (EH frame first); when the int ar… |
| `01254e70` | Engine::FRTWEngineDefendDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kDefendDisplayAction registration tag; no arguments or state. |
| `01254e80` | Engine::FRTWEngineDefendDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01254ee0` | — | unknown | Vtable slot [2] body: pushes the float at global DAT_017ea718 as the argument of FUN_012e4830, passes its eax result as ECX to FUN_012eeb70(), then returns 0 (… |

**Engine::FRTWEnginePlayAnimationDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255150` | Engine::FRTWEnginePlayAnimationDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kPlayAnimationDisplayAction registration tag; no arguments or state. |
| `01255160` | Engine::FRTWEnginePlayAnimationDisplayAction::ScalarDeletin… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `012550c0` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEnginePlayAnimationDisplayAction::vftable (EH frame first); when the… |
| `012551c0` | — | unknown | Vtable slot [2] body: gets DisplayAction argument index 3 via FUN_012f8f60(3), requires its type field at +4 to equal 1 (boolean; otherwise asserts via FUN_012… |

**Engine::FRTWGameActionCompleted** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012553c0` | Engine::FRTWGameActionCompleted::FRTWGameActionCompleted | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameActionCompleted::vftable (EH frame first); when the int argument… |
| `01255450` | Engine::FRTWGameActionCompleted::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kActionCompletedDisplayAction registration tag; no arguments or state. |
| `01255460` | Engine::FRTWGameActionCompleted::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |

**Engine::FRTWGamePlayingPower** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255260` | Engine::FRTWGamePlayingPower::FRTWGamePlayingPower | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGamePlayingPower::vftable (EH frame first); when the int argument is… |
| `012552f0` | Engine::FRTWGamePlayingPower::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kPlayingPowerDisplayAction registration tag; no arguments or state. |
| `01255300` | Engine::FRTWGamePlayingPower::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |

**Engine::FRTWGamePurgeCacheDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254110` | Engine::FRTWGamePurgeCacheDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kPurgeCacheDisplayAction registration tag; no arguments or state. |
| `01254120` | Engine::FRTWGamePurgeCacheDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01254080` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGamePurgeCacheDisplayAction::vftable (EH frame first); when the int … |

**Engine::FRTWGameShuffleBasesDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254260` | Engine::FRTWGameShuffleBasesDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kShuffleBasesDisplayAction registration tag; no arguments or state. |
| `01254270` | Engine::FRTWGameShuffleBasesDisplayAction::ScalarDeletingDt… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01265120` | Engine::FRTWGameShuffleBasesDisplayAction::VTableSlot2 | likely | Vtable slot [2] body: reads ValueData bool argument 1 (FUN_012f8f60(1); kBooleanTypeID assert at line 0x12f otherwise). Over zone indices 0 to 3 it appends, th… |
| `012541d0` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameShuffleBasesDisplayAction::vftable (EH frame first); when the in… |

**Engine::FRTWGameSpecialEffectDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012543b0` | Engine::FRTWGameSpecialEffectDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kSpecialEffectDisplayAction registration tag; no arguments or state. |
| `012543c0` | Engine::FRTWGameSpecialEffectDisplayAction::ScalarDeletingD… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `012655b0` | Engine::FRTWGameSpecialEffectDisplayAction::VTableSlot2 | likely | Vtable slot [2] body: reads int ValueData arguments 1, 2 and 3 (FUN_012f8f60, kIntegerTypeID assert at line 0x129) and switches on argument 1 over effect codes… |
| `01254320` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameSpecialEffectDisplayAction::vftable (EH frame first); when the i… |

**Engine::FRTWGameTieDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253fc0` | Engine::FRTWGameTieDisplayAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) holding the kTieDisplayAction registration tag with source file and line; no arguments or state. |
| `01253fd0` | Engine::FRTWGameTieDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `01253f30` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameTieDisplayAction::vftable (EH frame set up first); when the int … |

**Engine::FRTWGameUpdateCreatureActionsDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01263ed0` | Engine::FRTWGameUpdateCreatureActionsDisplayAction::VTableS… | likely | Vtable slot [2] body: reads ValueData argument 1 with FUN_012f8f60(1) (kIntegerTypeID assert at ValueData.h line 0x129); passes its payload to FUN_012638a0, or… |

**Engine::FRTWResizeWindow** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01257750` | Engine::FRTWResizeWindow::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kResizeWindow registration tag; no arguments or state. |
| `01257760` | Engine::FRTWResizeWindow::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) throu… |
| `012576c0` | — | unknown | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWResizeWindow::vftable (EH frame first); when the int argument is non… |

**Engine::GameAddPlayerDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012663a0` | Engine::GameAddPlayerDisplayAction::GameAddPlayerDisplayAct… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameAddPlayerDisplayAction::vftable into *this; when the flag … |
| `01266430` | Engine::GameAddPlayerDisplayAction::GetDebugName | confirmed | Returns the constant C string kAddPlayerDisplayAction : 0 : gamehandlers.cc:107 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266440` | Engine::GameAddPlayerDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameCancelDrawDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266d00` | Engine::GameCancelDrawDisplayAction::GameCancelDrawDisplayA… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameCancelDrawDisplayAction::vftable into *this; when the flag… |
| `01266d90` | Engine::GameCancelDrawDisplayAction::GetDebugName | confirmed | Returns the constant C string kCancelDrawDisplayAction : 0 : gamehandlers.cc:194 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266da0` | Engine::GameCancelDrawDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameDebugGameStatusDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266250` | Engine::GameDebugGameStatusDisplayAction::GameDebugGameStat… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameDebugGameStatusDisplayAction::vftable into *this; when the… |
| `012662e0` | Engine::GameDebugGameStatusDisplayAction::GetDebugName | confirmed | Returns the constant C string kDebugGameStatusDisplayAction : 0 : gamehandlers.cc:91 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `012662f0` | Engine::GameDebugGameStatusDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameFinishedConnectDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01265f30` | Engine::GameFinishedConnectDisplayAction::GameFinishedConne… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameFinishedConnectDisplayAction::vftable into *this; when the… |
| `01265fc0` | Engine::GameFinishedConnectDisplayAction::GetDebugName | confirmed | Returns the constant C string kFinishedConnectDisplayAction : 0 : gamehandlers.cc:60 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01265fd0` | Engine::GameFinishedConnectDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameGetTargetDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266660` | Engine::GameGetTargetDisplayAction::GameGetTargetDisplayAct… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameGetTargetDisplayAction::vftable into *this; when the flag … |
| `012666f0` | Engine::GameGetTargetDisplayAction::GetDebugName | confirmed | Returns the constant C string kGetTargetDisplayAction : 0 : gamehandlers.cc:119 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266700` | Engine::GameGetTargetDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameOfferAdjournDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266e50` | Engine::GameOfferAdjournDisplayAction::GameOfferAdjournDisp… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameOfferAdjournDisplayAction::vftable into *this; when the fl… |
| `01266ee0` | Engine::GameOfferAdjournDisplayAction::GetDebugName | confirmed | Returns the constant C string kOfferAdjournDisplayAction : 0 : gamehandlers.cc:211 (6 bytes: mov eax, imm32; ret); no arguments or state. |

**Engine::GameOfferDrawDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266ba0` | Engine::GameOfferDrawDisplayAction::GameOfferDrawDisplayAct… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameOfferDrawDisplayAction::vftable into *this; when the flag … |
| `01266c30` | Engine::GameOfferDrawDisplayAction::GetDebugName | confirmed | Returns the constant C string kOfferDrawDisplayAction : 0 : gamehandlers.cc:187 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266c40` | Engine::GameOfferDrawDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GamePauseDisplayActionsDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266080` | Engine::GamePauseDisplayActionsDisplayAction::GamePauseDisp… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GamePauseDisplayActionsDisplayAction::vftable into *this; when… |
| `01266110` | Engine::GamePauseDisplayActionsDisplayAction::GetDebugName | confirmed | Returns the constant C string kPauseDisplayActionsDisplayAction : 0 : gamehandlers.cc:75 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266120` | Engine::GamePauseDisplayActionsDisplayAction::ScalarDeletin… | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |
| `01266180` | Engine::GamePauseDisplayActionsDisplayAction::VTableSlot2 | likely | Vtable slot [2] body: reads int ValueData argument 1 (FUN_012f8f60(1); kIntegerTypeID assert at line 0x129 otherwise). When the value as a float exceeds the co… |

**Engine::GameRemovePlayerDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266500` | Engine::GameRemovePlayerDisplayAction::GameRemovePlayerDisp… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameRemovePlayerDisplayAction::vftable into *this; when the fl… |
| `01266590` | Engine::GameRemovePlayerDisplayAction::GetDebugName | confirmed | Returns the constant C string kRemovePlayerDisplayAction : 0 : gamehandlers.cc:113 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `012665a0` | Engine::GameRemovePlayerDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameSetPlaymatModeDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266a50` | Engine::GameSetPlaymatModeDisplayAction::GameSetPlaymatMode… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameSetPlaymatModeDisplayAction::vftable into *this; when the … |
| `01266ae0` | Engine::GameSetPlaymatModeDisplayAction::GetDebugName | confirmed | Returns the constant C string kSetPlaymatModeDisplayAction : 0 : gamehandlers.cc:171 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266af0` | Engine::GameSetPlaymatModeDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameUpdateElementDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01266900` | Engine::GameUpdateElementDisplayAction::GameUpdateElementDi… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameUpdateElementDisplayAction::vftable into *this; when the f… |
| `01266990` | Engine::GameUpdateElementDisplayAction::GetDebugName | confirmed | Returns the constant C string kUpdateElementDisplayAction : 1024 : gamehandlers.cc:147 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `012669a0` | Engine::GameUpdateElementDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::GameUpdateHintsDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012667b0` | Engine::GameUpdateHintsDisplayAction::GameUpdateHintsDispla… | confirmed | Constructor (thiscall, int flag): stores Engine::IDisplayActionHandler::vftable and then Engine::GameUpdateHintsDisplayAction::vftable into *this; when the fla… |
| `01266840` | Engine::GameUpdateHintsDisplayAction::GetDebugName | confirmed | Returns the constant C string kUpdateHintsDisplayAction : 1024 : gamehandlers.cc:133 (6 bytes: mov eax, imm32; ret); no arguments or state. |
| `01266850` | Engine::GameUpdateHintsDisplayAction::ScalarDeletingDtor | likely | Scalar deleting destructor (thiscall, byte flag): stores Engine::IDisplayActionHandler::vftable into *this; when flag bit 0 is set calls _free(this) through th… |

**Engine::IDisplayActionHandler** (25)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254030` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254180` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012542d0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254ad0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254c20` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254d90` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254f00` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255070` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255210` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255370` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01257120` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01257670` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01265e90` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01265ee0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266030` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266200` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266350` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `012664b0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266610` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266760` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `012668b0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266a00` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266b50` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266cb0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |
| `01266e00` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Fastcall (ECX = this): stores Engine::IDisplayActionHandler::vftable into *this and returns; no other stores, no calls. |

### `ui.frtwgame.scaleform`

**Engine::ClonesWarsSetCampaignIDFromMMO** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01260690` | Engine::ClonesWarsSetCampaignIDFromMMO::Vslot2 | confirmed | Reads int argument 0 (kIntegerTypeID assert) and calls 01327790 with it when 012583b0 succeeds, then queues a StartCampaignGame ScaleformAction (012ec110, 012e… |

**Engine::FRTWAttackTargetFromSquare** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012562f0` | Engine::FRTWAttackTargetFromSquare::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name AttackTargetFromSquare; no arguments or state. |
| `01256310` | Engine::FRTWAttackTargetFromSquare::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the AttackTargetFromSquare registration tag (name, version 0, source file and line); no arguments… |
| `01256320` | Engine::FRTWAttackTargetFromSquare::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01256280` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWAttackTargetFromSquare::vftable (EH frame first); when the int … |

**Engine::FRTWConsiderAIResume** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012547a0` | Engine::FRTWConsiderAIResume::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ConsiderAIResume; no arguments or state. |
| `012547c0` | Engine::FRTWConsiderAIResume::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ConsiderAIResume registration tag (name, version 0, source file and line); no arguments or st… |
| `012547d0` | Engine::FRTWConsiderAIResume::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01254730` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWConsiderAIResume::vftable (EH frame first); when the int argume… |

**Engine::FRTWEngineRequestForCampaignInfo** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01257480` | Engine::FRTWEngineRequestForCampaignInfo::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestCampaignInfo; no arguments or state. |
| `012574a0` | Engine::FRTWEngineRequestForCampaignInfo::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestCampaignInfo registration tag (name, version 0, source file and line); no arguments or… |
| `012574b0` | Engine::FRTWEngineRequestForCampaignInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01257410` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWEngineRequestForCampaignInfo::vftable (EH frame first); when th… |

**Engine::FRTWEngineStartQACampaignGame** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01257330` | Engine::FRTWEngineStartQACampaignGame::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name StartQAGame; no arguments or state. |
| `01257350` | Engine::FRTWEngineStartQACampaignGame::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the StartQAGame registration tag (name, version 0, source file and line); no arguments or state. |
| `01257360` | Engine::FRTWEngineStartQACampaignGame::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `012607d0` | Engine::FRTWEngineStartQACampaignGame::Vslot2 | confirmed | Reads the int argument CampaignID (012eb290, default 0x65), passes it to 012583b0 and 01327790, creates a ScaleformAction named StartCampaignGame (012ec110), q… |
| `012572c0` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWEngineStartQACampaignGame::vftable (EH frame first); when the i… |

**Engine::FRTWEngineUpdateElementDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125e100` | — | unknown | Reads ValueData arguments 1 and 2 as integers (kIntegerTypeID assert) and argument 3 as a value entry; after a 0131bd10 lookup it queues an Invoke ScaleformAct… |

**Engine::FRTWGameActionButtonSelected** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255bd0` | Engine::FRTWGameActionButtonSelected::FRTWGameActionButtonS… | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameActionButtonSelected::vftable (EH frame first); when the in… |
| `01255c40` | Engine::FRTWGameActionButtonSelected::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ActionButtonSelected; no arguments or state. |
| `01255c60` | Engine::FRTWGameActionButtonSelected::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ActionButtonSelected registration tag (name, version 0, source file and line); no arguments o… |
| `01255c70` | Engine::FRTWGameActionButtonSelected::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |

**Engine::FRTWGameCloseInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255670` | Engine::FRTWGameCloseInfo::FRTWGameCloseInfo | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameCloseInfo::vftable (EH frame first); when the int argument … |
| `012556e0` | Engine::FRTWGameCloseInfo::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name CloseInfo; no arguments or state. |
| `01255700` | Engine::FRTWGameCloseInfo::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the CloseInfo registration tag (name, version 0, source file and line); no arguments or state. |
| `01255710` | Engine::FRTWGameCloseInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `0125fd00` | Engine::FRTWGameCloseInfo::Vslot2 | confirmed | Creates an Invoke ScaleformAction whose MethodName is CloseRichInfoDialog (012ec220), queues it (012e5730, 012e5820) and returns 1 (ret 8). |

**Engine::FRTWGameExecuteAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255d20` | Engine::FRTWGameExecuteAction::FRTWGameExecuteAction | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameExecuteAction::vftable (EH frame first); when the int argum… |
| `01255d90` | Engine::FRTWGameExecuteAction::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ExecuteAction; no arguments or state. |
| `01255db0` | Engine::FRTWGameExecuteAction::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ExecuteAction registration tag (name, version 0, source file and line); no arguments or state. |
| `01255dc0` | Engine::FRTWGameExecuteAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |

**Engine::FRTWGameGameTimerComplete** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255af0` | Engine::FRTWGameGameTimerComplete::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name GameTimerComplete; no arguments or state. |
| `01255b10` | Engine::FRTWGameGameTimerComplete::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the GameTimerComplete registration tag (name, version 0, source file and line); no arguments or s… |
| `01255b20` | Engine::FRTWGameGameTimerComplete::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01255a80` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameGameTimerComplete::vftable (EH frame first); when the int a… |

**Engine::FRTWGameMoveCardDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125db40` | Engine::FRTWGameMoveCardDisplayAction::Vslot2 | confirmed | Returns 1. Gets the thread-owned object (0124d010), reads ValueData argument 1 as an integer (012f8f60, kIntegerTypeID assert), looks up a model object (0131b3… |

**Engine::FRTWGameQuerySpellInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255fe0` | Engine::FRTWGameQuerySpellInfo::FRTWGameQuerySpellInfo | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameQuerySpellInfo::vftable (EH frame first); when the int argu… |
| `01256050` | Engine::FRTWGameQuerySpellInfo::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name QuerySpellInfo; no arguments or state. |
| `01256070` | Engine::FRTWGameQuerySpellInfo::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the QuerySpellInfo registration tag (name, version 1, source file and line); no arguments or stat… |
| `01256080` | Engine::FRTWGameQuerySpellInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `0125ffa0` | — | unknown | Returns nothing; when the thread-owned object (0124d010) and the object from 0131b3c0 are non-null it reads the int ActionID (012eb290), fills three local stri… |

**Engine::FRTWGameRequestInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255520` | Engine::FRTWGameRequestInfo::FRTWGameRequestInfo | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameRequestInfo::vftable (EH frame first); when the int argumen… |
| `01255590` | Engine::FRTWGameRequestInfo::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestInfo; no arguments or state. |
| `012555b0` | Engine::FRTWGameRequestInfo::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestInfo registration tag (name, version 0, source file and line); no arguments or state. |
| `012555c0` | Engine::FRTWGameRequestInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `0125f700` | Engine::FRTWGameRequestInfo::Vslot2 | confirmed | Reads the int argument ElementID (012eb290) and looks up a card with 0131bd10; when found and 013809a0 returns 0x1ae03 or 0x1ae04 it queues an Invoke Scaleform… |

**Engine::FRTWGameRotateList** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012557c0` | Engine::FRTWGameRotateList::FRTWGameRotateList | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameRotateList::vftable (EH frame first); when the int argument… |
| `01255830` | Engine::FRTWGameRotateList::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RotateList; no arguments or state. |
| `01255850` | Engine::FRTWGameRotateList::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RotateList registration tag (name, version 0, source file and line); no arguments or state. |
| `01255860` | Engine::FRTWGameRotateList::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `0125fdb0` | Engine::FRTWGameRotateList::Vslot2 | confirmed | Reads the int argument Direction (012eb290). With the thread-owned object (0124d010) and a non-empty list at object+0x44/+0x48, finds the index of the current … |

**Engine::FRTWGameSetPhaseDialogDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125f040` | — | unknown | Reads string arguments 1 to 5 (kStringTypeID asserts) and asserts integer arguments 6 and 7; the value of argument 7 is read. When argument 1 equals CustomChoi… |

**Engine::FRTWGameSetPlaymatModeDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125e700` | Engine::FRTWGameSetPlaymatModeDisplayAction::Vslot2 | confirmed | Checks ValueData argument 1 as a string (kStringTypeID assert), then queues an Invoke ScaleformAction with MethodName SetPlaymatMode (012ec220) with the string… |

**Engine::FRTWGameSetupDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125ef40` | Engine::FRTWGameSetupDisplayAction::Vslot2 | confirmed | Checks ValueData argument 1 as a boolean (kBooleanTypeID assert); when the argument is not a true boolean it queues an Invoke ScaleformAction with MethodName C… |

**Engine::FRTWGameShowEndOfGameDialogDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125ea20` | — | unknown | Reads int argument 1 (kIntegerTypeID assert) and boolean argument 2 (kBooleanTypeID assert). Sets a winner value to 1 when argument 1 equals the result of the … |

**Engine::FRTWGameSpellCardsPopulated** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255e80` | Engine::FRTWGameSpellCardsPopulated::FRTWGameSpellCardsPopu… | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameSpellCardsPopulated::vftable (EH frame first); when the int… |
| `01255ef0` | Engine::FRTWGameSpellCardsPopulated::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SpellCardsPopulated; no arguments or state. |
| `01255f10` | Engine::FRTWGameSpellCardsPopulated::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SpellCardsPopulated registration tag (name, version 0, source file and line); no arguments or… |
| `01255f20` | Engine::FRTWGameSpellCardsPopulated::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |

**Engine::FRTWGameTieDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125f620` | Engine::FRTWGameTieDisplayAction::Vslot2 | confirmed | Gets the thread-owned object (0124d010) and, when non-null, calls 0131de20 and 0131de60; queues an Invoke ScaleformAction with MethodName DoTie (012ec220, 012e… |

**Engine::FRTWGameUnpause** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01255910` | Engine::FRTWGameUnpause::FRTWGameUnpause | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameUnpause::vftable (EH frame first); when the int argument is… |
| `01255980` | Engine::FRTWGameUnpause::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name Unpause; no arguments or state. |
| `012559a0` | Engine::FRTWGameUnpause::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the Unpause registration tag (name, version 0, source file and line); no arguments or state. |
| `012559b0` | Engine::FRTWGameUnpause::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01255a10` | Engine::FRTWGameUnpause::VTableSlot2 | likely | Vtable slot [2] body: calls FUN_012e4830() with no arguments, passes its eax result as ECX to FUN_012ee180(), then returns 1 (ret 8). |

**Engine::FRTWHoverFade** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256750` | Engine::FRTWHoverFade::FRTWHoverFade | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWHoverFade::vftable (EH frame first); when the int argument is n… |
| `012567c0` | Engine::FRTWHoverFade::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name HoverFade; no arguments or state. |
| `012567e0` | Engine::FRTWHoverFade::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the HoverFade registration tag (name, version 0, source file and line); no arguments or state. |
| `012567f0` | Engine::FRTWHoverFade::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01256850` | Engine::FRTWHoverFade::VTableSlot2 | likely | Vtable slot [2] body (security-cookie frame): calls FUN_012eb290(ElementID literal, 0) to get an ElementID value, then FUN_013276b0(0xe8) and FUN_013275f0 with… |

**Engine::FRTWHoverGlow** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256520` | Engine::FRTWHoverGlow::FRTWHoverGlow | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWHoverGlow::vftable (EH frame first); when the int argument is n… |
| `01256590` | Engine::FRTWHoverGlow::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name HoverGlow; no arguments or state. |
| `012565b0` | Engine::FRTWHoverGlow::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the HoverGlow registration tag (name, version 0, source file and line); no arguments or state. |
| `012565c0` | Engine::FRTWHoverGlow::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01256620` | — | unknown | Vtable slot [2] body (security-cookie frame): calls FUN_012eb290(ElementID literal, 0) to get an ElementID value, then FUN_013276b0(0xe5) and FUN_013275f0 with… |

**Engine::FRTWIsPlayerImageLoaded** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012579d0` | Engine::FRTWIsPlayerImageLoaded::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name IsPlayerImageLoaded; no arguments or state. |
| `01257a00` | Engine::FRTWIsPlayerImageLoaded::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `012579f0` | Engine::FRTWIsPlayerImageLoaded::GetDebugName | likely | Returns a constant string (6 bytes: mov eax, imm32; ret) with the IsPlayerImageLoaded registration tag (name, version 0, source file and line); no arguments or… |
| `01257960` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWIsPlayerImageLoaded::vftable (EH frame first); when the int arg… |
| `01260bd0` | — | unknown | Reads the string argument CharacterID, converts it with __strtoui64 base 10, and when the id is nonzero and the manager at thread-owned object +0xc0 is non-nul… |

**Engine::FRTWObjectsPicked** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256130` | Engine::FRTWObjectsPicked::FRTWObjectsPicked | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWObjectsPicked::vftable (EH frame first); when the int argument … |
| `012561a0` | Engine::FRTWObjectsPicked::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ObjectsPicked; no arguments or state. |
| `012561c0` | Engine::FRTWObjectsPicked::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ObjectsPicked registration tag (name, version 0, source file and line); no arguments or state. |
| `012561d0` | Engine::FRTWObjectsPicked::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01265d50` | Engine::FRTWObjectsPicked::VTableSlot2 | likely | Vtable slot [2] body: if the game object (0124d010) exists and ValueData Count (012eb290) is positive, appends FUN_012eb0a0(i+2, 0) for i from 0 to Count-1 int… |

**Engine::FRTWOpenStore** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012575e0` | Engine::FRTWOpenStore::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name OpenStore; no arguments or state. |
| `01257600` | Engine::FRTWOpenStore::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the OpenStore registration tag (name, version 0, source file and line); no arguments or state. |
| `01257610` | Engine::FRTWOpenStore::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01257570` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWOpenStore::vftable (EH frame first); when the int argument is n… |
| `012608e0` | — | unknown | Reads int arguments StoreLocation and CategoryID (default -1) and string argument BundleStr; when the thread-owned object (0124d010) has a non-null manager at … |

**Engine::FRTWRequestExitGameScreen** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012563d0` | Engine::FRTWRequestExitGameScreen::FRTWRequestExitGameScreen | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWRequestExitGameScreen::vftable (EH frame first); when the int a… |
| `01256440` | Engine::FRTWRequestExitGameScreen::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestExitGameScreen; no arguments or state. |
| `01256460` | Engine::FRTWRequestExitGameScreen::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestExitGameScreen registration tag (name, version 0, source file and line); no arguments … |
| `01256470` | Engine::FRTWRequestExitGameScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `012601f0` | Engine::FRTWRequestExitGameScreen::Vslot2 | confirmed | Calls virtual slot +0x20 (no arguments) on the thread-owned object from 0124d010 when it is non-null, creates a ScaleformAction named ChangeToPreviousScreen (0… |

**Engine::FRTWRequestPlayerImage** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01257880` | Engine::FRTWRequestPlayerImage::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestPlayerImage; no arguments or state. |
| `012578a0` | Engine::FRTWRequestPlayerImage::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestPlayerImage registration tag (name, version 0, source file and line); no arguments or … |
| `012578b0` | Engine::FRTWRequestPlayerImage::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01257810` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWRequestPlayerImage::vftable (EH frame first); when the int argu… |
| `01260a50` | — | unknown | Reads the string argument CharacterID (012eb250), converts it with __strtoui64 base 10, and when the id is nonzero and the thread-owned object (0124d010) has a… |

**Engine::FRTWSelectAIBaseCard** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254640` | Engine::FRTWSelectAIBaseCard::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SelectAIBaseCard; no arguments or state. |
| `01254660` | Engine::FRTWSelectAIBaseCard::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SelectAIBaseCard registration tag (name, version 0, source file and line); no arguments or st… |
| `01254670` | Engine::FRTWSelectAIBaseCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `012545d0` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWSelectAIBaseCard::vftable (EH frame first); when the int argume… |

**Engine::FRTWSelectRandomCard** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012548f0` | Engine::FRTWSelectRandomCard::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SelectRandomCard; no arguments or state. |
| `01254910` | Engine::FRTWSelectRandomCard::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SelectRandomCard registration tag (name, version 0, source file and line); no arguments or st… |
| `01254920` | Engine::FRTWSelectRandomCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01254880` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWSelectRandomCard::vftable (EH frame first); when the int argume… |

**Engine::FRTWSelectRandomHandCard** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254a40` | Engine::FRTWSelectRandomHandCard::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SelectRandomHandCard; no arguments or state. |
| `01254a60` | Engine::FRTWSelectRandomHandCard::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SelectRandomHandCard registration tag (name, version 0, source file and line); no arguments o… |
| `01254a70` | Engine::FRTWSelectRandomHandCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `012549d0` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWSelectRandomHandCard::vftable (EH frame first); when the int ar… |

**Engine::FRTWSetFocus** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012544e0` | Engine::FRTWSetFocus::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SetFocus; no arguments or state. |
| `01254500` | Engine::FRTWSetFocus::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SetFocus registration tag (name, version 0, source file and line); no arguments or state. |
| `01254510` | Engine::FRTWSetFocus::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01254470` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWSetFocus::vftable (EH frame first); when the int argument is no… |

**Engine::FRTWStopTimerCriticalAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125e970` | Engine::FRTWStopTimerCriticalAction::Vslot2 | confirmed | Creates an Invoke ScaleformAction whose MethodName is EndGameTimer (012ec220), queues it (012e5730, 012e5820) and returns 1 (ret 8). |

**Engine::FRTWUIRequestForCampaignInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01257090` | Engine::FRTWUIRequestForCampaignInfo::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestCampaignInfo; no arguments or state. |
| `012570b0` | Engine::FRTWUIRequestForCampaignInfo::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestCampaignInfo registration tag (name, version 0, source file and line); no arguments or… |
| `012570c0` | Engine::FRTWUIRequestForCampaignInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01257020` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWUIRequestForCampaignInfo::vftable (EH frame first); when the in… |
| `01260310` | — | unknown | Builds an XML text with 0130e6c0, 0130df10, 0130e7b0 (CampaignID int), 0130e750 (AI flag from a virtual call at vtable +0x1a0 on the object from 01347ad0) and … |

**Engine::GetEffectData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256c70` | Engine::GetEffectData::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name GetEffectData; no arguments or state. |
| `01256c90` | Engine::GetEffectData::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the GetEffectData registration tag (name, version 0, source file and line); no arguments or state. |
| `01256ca0` | Engine::GetEffectData::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01256c00` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::GetEffectData::vftable (EH frame first); when the int argument is n… |

**Engine::RequestAnimations** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256f30` | Engine::RequestAnimations::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestAnimations; no arguments or state. |
| `01256f50` | Engine::RequestAnimations::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestAnimations registration tag (name, version 0, source file and line); no arguments or s… |
| `01256f60` | Engine::RequestAnimations::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01256ec0` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::RequestAnimations::vftable (EH frame first); when the int argument … |

**Engine::RequestEffects** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256960` | Engine::RequestEffects::RequestEffects | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::RequestEffects::vftable (EH frame first); when the int argument is … |
| `012569d0` | Engine::RequestEffects::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestEffects; no arguments or state. |
| `012569f0` | Engine::RequestEffects::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestEffects registration tag (name, version 0, source file and line); no arguments or stat… |
| `01256a00` | Engine::RequestEffects::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01263f30` | Engine::RequestEffects::VTableSlot2 | likely | Vtable slot [2] body: iterates a vector of 4-byte entries filled by 012b9450 (from 012ae910); for each entry allocates a 0x40-byte object (00d833a6), creates a… |

**Engine::SendCampaignData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256dd0` | Engine::SendCampaignData::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SendCampaignData; no arguments or state. |
| `01256df0` | Engine::SendCampaignData::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SendCampaignData registration tag (name, version 0, source file and line); no arguments or st… |
| `01256e00` | Engine::SendCampaignData::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01256d60` | — | unknown | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::SendCampaignData::vftable (EH frame first); when the int argument i… |

**Engine::SetCampaignGameMode** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01256ab0` | Engine::SetCampaignGameMode::SetCampaignGameMode | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::SetCampaignGameMode::vftable (EH frame first); when the int argumen… |
| `01256b20` | Engine::SetCampaignGameMode::GetShortName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SetCampaignGameMode; no arguments or state. |
| `01256b40` | Engine::SetCampaignGameMode::GetDebugName | confirmed | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SetCampaignGameMode registration tag (name, version 0, source file and line); no arguments or… |
| `01256b50` | Engine::SetCampaignGameMode::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) … |
| `01264270` | Engine::SetCampaignGameMode::VTableSlot2 | likely | Vtable slot [2] body: reads the game object (0124d010) and a campaign node (0124f2e0, byte at +8 must equal 1), reads ValueData MissionID (012eb290), tests the… |

**Scaleform::IScaleformActionHandler** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01254420` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254580` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012546e0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254830` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01254980` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012554d0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255620` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255770` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012558c0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255a30` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255b80` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255cd0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255e30` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01255f90` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012560e0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256230` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256380` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012564d0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256700` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256910` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256a60` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256bb0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256d10` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256e70` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01256fd0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01257270` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012573c0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01257520` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `012577c0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |
| `01257910` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns; no derived vftable store, no calls. |

## Source shards

[[S0659 01253f30-01257d60|S0659]] (208), [[S0661 0125d190-012619f0|S0661]] (19), [[S0662|S0662]] (55)
