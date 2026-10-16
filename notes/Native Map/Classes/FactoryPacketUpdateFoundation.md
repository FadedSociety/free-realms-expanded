# Class: FactoryPacketUpdateFoundation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d92e0` | `FactoryPacketUpdateFoundation::FactoryPacketUpdateFoundation` | ✅ | ctor | Constructor for FactoryPacketUpdateFoundation: calls FUN_009d90d0(0x10), installs FactoryPacketUpdateFoundation::vftable, runs the member initialiser FUN_010589d0 and clears the word at +0x110. |
| `009d9350` | `FactoryPacketUpdateFoundation::~FactoryPacketUpdateFoundation` | 🟢 | dtor | Destructor body: destroys the member via FUN_01058960, then rewrites the vtables to BaseFactoryPacket::vftable and BasePacket::vftable. |
| `009db2b0` | `FactoryPacketUpdateFoundation::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009d9350, then frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateFoundation::vftable (dword at 0x0181049c is this address). |
