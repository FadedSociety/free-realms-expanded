# Class: PetBuyRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51a70` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x19 (25), then stores PetBuyRequestPacket::vftable at *this. |
| `00b51ad0` | `PetBuyRequestPacket::~PetBuyRequestPacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b531b0` | `PetBuyRequestPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
