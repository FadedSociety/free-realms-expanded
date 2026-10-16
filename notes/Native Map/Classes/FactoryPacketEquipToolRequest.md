# Class: FactoryPacketEquipToolRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d96e0` | `FactoryPacketEquipToolRequest::FactoryPacketEquipToolRequest` | ✅ | ctor | Constructor for FactoryPacketEquipToolRequest: calls FUN_009d90d0(7), installs FactoryPacketEquipToolRequest::vftable and clears +0x0c. |
| `009db640` | `FactoryPacketEquipToolRequest::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketEquipToolRe… |
