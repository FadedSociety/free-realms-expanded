# Class: Engine::FRTWCampaignCampaignCompletionUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01250e80` | `Engine::FRTWCampaignCampaignCompletionUpdate::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWCampaignCampaignCompletionUpdate::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01250f10` | `Engine::FRTWCampaignCampaignCompletionUpdate::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kCampaignCompletionUpdate : 0 : ..\engine\campaign\frtwcampaignhandlers.cc:64'. |
| `01250f20` | `Engine::FRTWCampaignCampaignCompletionUpdate::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01250fa0` | `— ⚠reject` | ⚪ | virtual | Slot [2] of its vftable, 296 bytes. If FUN_0124d010 returns nonzero it reads ValueData entries 1 and 2 as integers (type tag 2, else assert kIntegerTypeID at ValueData.h:0x129) and entry 3 must be non-null; it then call… |
