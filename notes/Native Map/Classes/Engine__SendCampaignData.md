# Class: Engine::SendCampaignData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256d60` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::SendCampaignData::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256dd0` | `Engine::SendCampaignData::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SendCampaignData; no arguments or state. |
| `01256df0` | `Engine::SendCampaignData::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SendCampaignData registration tag (name, version 0, source file and line); no arguments or state. |
| `01256e00` | `Engine::SendCampaignData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
