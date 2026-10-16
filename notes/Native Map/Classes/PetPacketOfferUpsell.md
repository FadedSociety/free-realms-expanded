# Class: PetPacketOfferUpsell

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51df0` | `PetPacketOfferUpsell::PetPacketOfferUpsell ⚠downgrade` | 🟢 | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x24 (36), then stores PetPacketOfferUpsell::vftable at *this. |
| `00b51e50` | `PetPacketOfferUpsell::~PetPacketOfferUpsell` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b53390` | `PetPacketOfferUpsell::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
