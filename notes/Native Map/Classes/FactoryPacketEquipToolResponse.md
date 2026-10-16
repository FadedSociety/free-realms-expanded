# Class: FactoryPacketEquipToolResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d97f0` | `FactoryPacketEquipToolResponse::FactoryPacketEquipToolResponse` | 🟢 | ctor | Constructor for FactoryPacketEquipToolResponse: calls FUN_009d90d0(0x17), installs FactoryPacketEquipToolResponse::vftable, clears the byte at +0x0c and the word at +0x10. |
| `009db6b0` | `FactoryPacketEquipToolResponse::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketEquipToolRe… |
