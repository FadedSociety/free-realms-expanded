# Class: FactoryPacketOpenBlueprintGenerator

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009de250` | `FactoryPacketOpenBlueprintGenerator::FactoryPacketOpenBlueprintGenerator` | ✅ | ctor | Constructor for FactoryPacketOpenBlueprintGenerator: calls FUN_009d90d0(0x1b), installs FactoryPacketOpenBlueprintGenerator::vftable and initialises the same embedded SoeUtil::IString<char> member at +0x0c. |
| `009de2c0` | `FactoryPacketOpenBlueprintGenerator::~FactoryPacketOpenBlueprintGenerator` | 🟢 | dtor | Destructor body identical in shape to 009de1c0: restores the embedded IString member vtable, releases its reference-counted buffer when non-empty, then rewrites the object vtables to BaseFactoryPacket::vftable and BaseP… |
| `009df980` | `FactoryPacketOpenBlueprintGenerator::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009de2c0 and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketOpenBlueprintGenerator::vftable (dword at 0x0181059c is this address). |
