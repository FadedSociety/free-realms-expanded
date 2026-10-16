# Class: PetSetNamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b544d0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 10, stores PetSetNamePacket::vftable and initialises an embedded IString at +0x18 (vtable plus empty-string data pointer DAT_01be97c4, zeroed cursor and … |
| `00b54540` | `PetSetNamePacket::~PetSetNamePacket` | ✅ | dtor | Destructor: stores PetSetNamePacket::vftable, rewrites the embedded IString vtable at +0x18 and, when the string owns a buffer (+0x24 positive), atomically decrements the counter at data-4, calling the IString virtual a… |
| `00b545d0` | `PetSetNamePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor 00b54540 (IString release plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records… |
