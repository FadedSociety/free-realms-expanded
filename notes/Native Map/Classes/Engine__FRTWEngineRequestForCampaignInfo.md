# Class: Engine::FRTWEngineRequestForCampaignInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01257410` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWEngineRequestForCampaignInfo::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01257480` | `Engine::FRTWEngineRequestForCampaignInfo::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestCampaignInfo; no arguments or state. |
| `012574a0` | `Engine::FRTWEngineRequestForCampaignInfo::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestCampaignInfo registration tag (name, version 0, source file and line); no arguments or state. |
| `012574b0` | `Engine::FRTWEngineRequestForCampaignInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
