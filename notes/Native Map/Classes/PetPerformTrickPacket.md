# Class: PetPerformTrickPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50a10` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 7, then stores PetPerformTrickPacket::vftable at *this. |
| `00b50a70` | `PetPerformTrickPacket::~PetPerformTrickPacket` | ✅ | dtor | Destructor: stores PetPerformTrickPacket::vftable then BasePacket::vftable at *this, the full two-stage MSVC vtable reset; one unwind funclet calls it. |
| `00b50ac0` | `PetPerformTrickPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetPerformTrickPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. |
