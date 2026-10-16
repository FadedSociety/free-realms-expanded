# Class: PetPlayWithToyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50b30` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x21 (33), stores PetPlayWithToyPacket::vftable, then initialises an embedded Target sub-object at +0xc: zero at +0xc, Target::vftable at +0x10 and zero … |
| `00b50ba0` | `PetPlayWithToyPacket::~PetPlayWithToyPacket` | ✅ | dtor | Destructor: stores PetPlayWithToyPacket::vftable, tears down the embedded Target sub-object through 0101b370, then stores BasePacket::vftable. Three unwind funclets and the senders 00b5fa50/00b5fbc0 also call it. |
| `00b50c00` | `PetPlayWithToyPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor 00b50ba0 (Target teardown plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. |
