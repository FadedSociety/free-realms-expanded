# Class: Engine::FRTWEngineStartQACampaignGame

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012572c0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWEngineStartQACampaignGame::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01257330` | `Engine::FRTWEngineStartQACampaignGame::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name StartQAGame; no arguments or state. |
| `01257350` | `Engine::FRTWEngineStartQACampaignGame::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the StartQAGame registration tag (name, version 0, source file and line); no arguments or state. |
| `01257360` | `Engine::FRTWEngineStartQACampaignGame::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `012607d0` | `Engine::FRTWEngineStartQACampaignGame::Vslot2` | ✅ | virtual | Reads the int argument CampaignID (012eb290, default 0x65), passes it to 012583b0 and 01327790, creates a ScaleformAction named StartCampaignGame (012ec110), queues it (012e5730, 012e5820) and returns 1. |
