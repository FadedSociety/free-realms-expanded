# Class: BattleMagesCommandProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd5b90` | `BattleMagesCommandProcessor::BattleMagesCommandProcessor` | ✅ | ctor | Stores CommandProcessor::CommandEventHandler::vftable and then BattleMagesCommandProcessor::vftable at this; stores arg2 at +0x4; calls 00987000 with bm_reloaddata (id 1) and bm_camera (id 2); returns this. |
| `00cd5c20` | `BattleMagesCommandProcessor::~BattleMagesCommandProcessor` | ✅ | dtor | Stores BattleMagesCommandProcessor::vftable at this, calls 00986c10 with this, then stores CommandProcessor::CommandEventHandler::vftable at this; returns void. |
| `00cd5ca0` | `BattleMagesCommandProcessor::CallDtorBodyThenFreeIfFlag` | 🟢 | virtual | Virtual slot 1 of BattleMagesCommandProcessor: calls destructor body 00cd5c20 on this, frees this when bit 0 of the flag argument is set, and returns this. |
| `00cd69e0` | `BattleMagesCommandProcessor::ProcessCommand` | ✅ | virtual | Branches on command type values 1 and 2 and dispatches the matching command processing path. |
