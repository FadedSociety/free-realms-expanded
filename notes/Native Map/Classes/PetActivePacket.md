# Class: PetActivePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50d40` | `PetActivePacket::~PetActivePacket` | ✅ | dtor | Destructor: stores PetActivePacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b50d90` | `PetActivePacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetActivePacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
| `00b53020` | `PetActivePacket::PetActivePacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 9, stores PetActivePacket::vftable, writes the sentinel 0xffffffff at +0xc and the shared default pair DAT_01be9810/DAT_01be9814 at +0x10/+0x14. |
