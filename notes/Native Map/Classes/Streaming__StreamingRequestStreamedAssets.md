# Class: Streaming::StreamingRequestStreamedAssets

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123bde0` | `Streaming::StreamingRequestStreamedAssets::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable and then Streaming::StreamingRequestStreamedAssets::vftable into the first word of the object, and when the flag argument is nonzero calls FUN_012ea4f0 on the object. |
| `0123be50` | `Streaming::StreamingRequestStreamedAssets::GetActionName` | 🟢 | virtual | Virtual slot [1] of Streaming::StreamingRequestStreamedAssets::vftable: returns the address of the literal RequestStreamedAssets. |
| `0123be70` | `Streaming::StreamingRequestStreamedAssets::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of Streaming::StreamingRequestStreamedAssets::vftable: returns the address of its diagnostic literal. |
| `0123be80` | `Streaming::StreamingRequestStreamedAssets::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
| `0123bf70` | `Streaming::StreamingRequestStreamedAssets::BuildRequestFields ? ⚠rename` | 🟡 | virtual | Virtual slot [2] of Streaming::StreamingRequestStreamedAssets::vftable (904 bytes): assembles several named string fields (msgName, msgData, imgData, locale, defaults, initial) through FUN_012eb250 and FUN_012eb980/FUN_… |
