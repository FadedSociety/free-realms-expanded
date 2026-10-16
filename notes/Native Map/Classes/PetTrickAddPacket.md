# Class: PetTrickAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51150` | `PetTrickAddPacket::PetTrickAddPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0xd (13), stores PetTrickAddPacket::vftable, then zeroes nine dwords at +0xc through +0x2c and the byte at +0x30. |
| `00b511d0` | `PetTrickAddPacket::~PetTrickAddPacket` | ✅ | dtor | Destructor: stores PetTrickAddPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b51220` | `PetTrickAddPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickAddPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
