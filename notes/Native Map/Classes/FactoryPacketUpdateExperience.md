# Class: FactoryPacketUpdateExperience

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009de350` | `FactoryPacketUpdateExperience::FactoryPacketUpdateExperience` | ✅ | ctor | Constructor for FactoryPacketUpdateExperience: calls FUN_009d90d0(0x1d), installs FactoryPacketUpdateExperience::vftable, initialises the embedded IString member at +0x0c and clears the words at +0x1c/+0x20 plus the byt… |
| `009de3d0` | `FactoryPacketUpdateExperience::~FactoryPacketUpdateExperience` | 🟢 | dtor | Destructor body identical in shape to the other IString-carrying factory packets: restores the embedded IString member vtable, releases its buffer when non-empty and rewrites the object vtables to BaseFactoryPacket::vft… |
| `009df9a0` | `FactoryPacketUpdateExperience::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009de3d0 and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateExperience::vftable (dword at 0x018105a4 is this address). |
