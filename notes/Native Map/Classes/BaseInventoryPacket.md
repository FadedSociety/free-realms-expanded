# Class: BaseInventoryPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098ea30` | `BaseInventoryPacket::Constructor` | ✅ | ctor | Installs BasePacket, stores packet type 0x2a, installs BaseInventoryPacket, clears field +8, and returns. |
| `0098ea90` | `BaseInventoryPacket::Destructor` | ✅ | dtor | Restores BaseInventoryPacket then BasePacket vtables at object offset zero and returns. |
| `0098eae0` | `BaseInventoryPacket::ScalarDeletingDestructor` | ✅ | dtor | Restores BaseInventoryPacket and BasePacket vtables, frees this only when deleting flag bit 0 is set, and otherwise returns this. |
| `00a10400` | `BaseInventoryPacket_Ctor` | ✅ | ctor | Constructor: stores BasePacket::vftable, writes the family opcode 0x2a (42) into the packet opcode field at +4, installs BaseInventoryPacket::vftable and stores the caller's sub-opcode param_2 into the field at +8. |
| `00a10600` | `BaseInventoryPacket_Dtor` | 🟢 | dtor | Destructor body: stores BaseInventoryPacket::vftable then BasePacket::vftable; no fields released. |
| `00a12380` | `BaseInventoryPacket_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: stores BaseInventoryPacket::vftable then BasePacket::vftable and frees the object when bit 0 of param_2 is set. |
| `00bd5920` | `BaseInventoryPacket::~BaseInventoryPacket` | ✅ | dtor | Destructor body: stores BaseInventoryPacket::vftable then BasePacket::vftable at +0; no other work; returns nothing. |
| `00c09000` | `BaseInventoryPacket_DtorBody` | ✅ | dtor | Base-class destructor body: rewrites the vtable to BaseInventoryPacket::vftable then BasePacket::vftable; called from the inventory request's deleting destructor 00c096b0 (via the funclet at 015f44d0) during the item-re… |
