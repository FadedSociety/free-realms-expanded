# Class: FactoryPacketUpdateFactoryInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d9200` | `FactoryPacketUpdateFactoryInfo::FactoryPacketUpdateFactoryInfo` | ✅ | ctor | Constructor for FactoryPacketUpdateFactoryInfo: calls the shared initialiser FUN_009d90d0(1), installs FactoryPacketUpdateFactoryInfo::vftable, initialises the trailing member pair at +0x2270/+0x2274 through FUN_0105886… |
| `009d9280` | `FactoryPacketUpdateFactoryInfo::~FactoryPacketUpdateFactoryInfo` | 🟢 | dtor | Destructor body: destroys the member map via FUN_01058700 (HashListMap keyed by StrongType<u64,CharacterGuid> in the FactoryMisfortune family), then rewrites the vtables to BaseFactoryPacket::vftable and BasePacket::vft… |
| `009db290` | `FactoryPacketUpdateFactoryInfo::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009d9280, then frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateFactoryInfo::vftable (dword at 0x01810494 is this address). |
