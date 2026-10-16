# Class: PetPreviewRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51ea0` | `PetPreviewRequestPacket::PetPreviewRequestPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x26 (38), stores PetPreviewRequestPacket::vftable and zeroes the four dwords at +0xc through +0x18. |
| `00b51f10` | `PetPreviewRequestPacket::~PetPreviewRequestPacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; the pet dispatcher 00b5ce60 and two unwind funclets call it. |
| `00b533f0` | `PetPreviewRequestPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
