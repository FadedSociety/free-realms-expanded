# Class: Streaming::SOEAssetLoaderHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123bbc0` | `Streaming::SOEAssetLoaderHandler::Ctor` | ✅ | ctor | Stores AssetManagement::AssetDeliveryHandlerInterface::vftable into the first word, stores the argument at +4, then overwrites the first word with Streaming::SOEAssetLoaderHandler::vftable. |
| `0123bd80` | `Streaming::SOEAssetLoaderHandler::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_010935e0 (base teardown) with the stack-cookie value, then frees the object through _free when bit 0 of the flag argument is set; returns the object pointer. |
