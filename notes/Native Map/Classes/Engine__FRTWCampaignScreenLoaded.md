# Class: Engine::FRTWCampaignScreenLoaded

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01250d80` | `Engine::FRTWCampaignScreenLoaded::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWCampaignScreenLoaded::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01250df0` | `Engine::FRTWCampaignScreenLoaded::TypeName` | ✅ | virtual | Slot [1] of Engine::FRTWCampaignScreenLoaded::vftable. Returns the constant C string 'CampaignScreenLoaded'. |
| `01250e10` | `Engine::FRTWCampaignScreenLoaded::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'CampaignScreenLoaded : 0 : .\engine\campaign\frtwcampaignhandlers.cc:24'. |
| `01250e20` | `Engine::FRTWCampaignScreenLoaded::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012510d0` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::FRTWCampaignScreenLoaded::vftable, 1426 bytes. Creates an action named 'InitializeCampaigns' (FUN_012ec220) and then, in a loop of 40 iterations, adds a 'Campaign' child (FUN_0130df10) and sets 'Camp… |
