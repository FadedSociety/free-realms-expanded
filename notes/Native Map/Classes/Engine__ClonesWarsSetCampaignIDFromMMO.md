# Class: Engine::ClonesWarsSetCampaignIDFromMMO

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01257170` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::ClonesWarsSetCampaignIDFromMMO::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01257200` | `Engine::ClonesWarsSetCampaignIDFromMMO::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kSetCampaignIdDisplayAction registration tag; no arguments or state. |
| `01257210` | `Engine::ClonesWarsSetCampaignIDFromMMO::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01260690` | `Engine::ClonesWarsSetCampaignIDFromMMO::Vslot2` | ✅ | virtual | Reads int argument 0 (kIntegerTypeID assert) and calls 01327790 with it when 012583b0 succeeds, then queues a StartCampaignGame ScaleformAction (012ec110, 012e5730, 012e5820); returns 1. |
