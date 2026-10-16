# Class: Engine::FRTWCampaignScreenUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01250c20` | `Engine::FRTWCampaignScreenUpdate::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWCampaignScreenUpdate::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01250c90` | `Engine::FRTWCampaignScreenUpdate::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::FRTWCampaignScreenUpdate::vftable. Returns the constant C string 'CampaignScreenUpdate'. |
| `01250cb0` | `Engine::FRTWCampaignScreenUpdate::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'CampaignScreenUpdate : 0 : .\engine\campaign\frtwcampaignhandlers.cc:52'. |
| `01250cc0` | `Engine::FRTWCampaignScreenUpdate::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01250f80` | `Engine::FRTWCampaignScreenUpdate::RefreshIfActive ? ⚠downgrade` | 🟡 | virtual | Slot [2] of Engine::FRTWCampaignScreenUpdate::vftable, 31 bytes. Calls FUN_012e0ba0, then FUN_0124d010; if that returns nonzero it calls FUN_0131faa0; returns 1. |
