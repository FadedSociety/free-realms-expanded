# Class: PetRequestNamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b543b0` | `PetRequestNamePacket::PetRequestNamePacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 6, stores PetRequestNamePacket::vftable and initialises an embedded IString at +0xc (vtable plus the empty-string data pointer DAT_01be97c4, then the zer… |
| `00b54420` | `PetRequestNamePacket::~PetRequestNamePacket` | ✅ | dtor | Destructor: stores PetRequestNamePacket::vftable, rewrites the embedded IString vtable at +0xc and, when the string owns a buffer (+0x18 positive), atomically decrements the reference counter at data-4 and calls the ISt… |
| `00b544b0` | `PetRequestNamePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor 00b54420 (IString release plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records… |
