# Class: PetTrickNotifyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b513b0` | `PetTrickNotifyPacket::PetTrickNotifyPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0xf (15), stores PetTrickNotifyPacket::vftable and zeroes the five dwords at +0xc through +0x1c. |
| `00b51420` | `PetTrickNotifyPacket::~PetTrickNotifyPacket` | ✅ | dtor | Destructor: stores PetTrickNotifyPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b51470` | `PetTrickNotifyPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickNotifyPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
