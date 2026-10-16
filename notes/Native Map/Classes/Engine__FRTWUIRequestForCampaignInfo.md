# Class: Engine::FRTWUIRequestForCampaignInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01257020` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWUIRequestForCampaignInfo::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01257090` | `Engine::FRTWUIRequestForCampaignInfo::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestCampaignInfo; no arguments or state. |
| `012570b0` | `Engine::FRTWUIRequestForCampaignInfo::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestCampaignInfo registration tag (name, version 0, source file and line); no arguments or state. |
| `012570c0` | `Engine::FRTWUIRequestForCampaignInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01260310` | `— ⚠reject` | ⚪ | virtual | Builds an XML text with 0130e6c0, 0130df10, 0130e7b0 (CampaignID int), 0130e750 (AI flag from a virtual call at vtable +0x1a0 on the object from 01347ad0) and 0130e380 (EnemyName and EnemyIMG strings from the record fro… |
