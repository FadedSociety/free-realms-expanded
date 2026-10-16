# Class: PetUiModePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b519c0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x18 (24), then stores PetUiModePacket::vftable at *this. |
| `00b53150` | `PetUiModePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
