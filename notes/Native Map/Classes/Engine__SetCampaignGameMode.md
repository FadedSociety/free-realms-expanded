# Class: Engine::SetCampaignGameMode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256ab0` | `Engine::SetCampaignGameMode::SetCampaignGameMode` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::SetCampaignGameMode::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256b20` | `Engine::SetCampaignGameMode::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SetCampaignGameMode; no arguments or state. |
| `01256b40` | `Engine::SetCampaignGameMode::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SetCampaignGameMode registration tag (name, version 0, source file and line); no arguments or state. |
| `01256b50` | `Engine::SetCampaignGameMode::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
