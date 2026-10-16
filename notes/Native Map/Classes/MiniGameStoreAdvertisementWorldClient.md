# Class: MiniGameStoreAdvertisementWorldClient

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a9f80` | `MiniGameStoreAdvertisementWorldClient::MiniGameStoreAdvertisementWorldClient` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x2c and three -1 header words, zeroes this[6], installs MiniGameStoreAdvertisementWorldClient::vftable and stores param_2 at this[7]. |
| `009ac810` | `DeletingDtor_009ac810` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
