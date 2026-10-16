# Class: PetMoodListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b561d0` | `PetMoodListPacket::PetMoodListPacket` | ✅ | ctor | Calls the PetBasePacket constructor with subtype 0x22, installs PetMoodListPacket and list vtables, and zeros the list fields. |
| `00b56240` | `PetMoodListPacket::~PetMoodListPacket` | ✅ | dtor | Replaces the derived packet vtable with PetMoodListPacket, clears its list, restores the list base vtable, clears that list, then restores BasePacket. |
| `00b562c0` | `PetMoodListPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor and frees the object only when the low bit of param_2 is set. |
