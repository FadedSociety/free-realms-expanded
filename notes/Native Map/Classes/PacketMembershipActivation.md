# Class: PacketMembershipActivation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089bab0` | `PacketMembershipActivationCtor` | ✅ | ctor | Constructor of PacketMembershipActivation: installs BasePacket::vftable, stores opcode 100 at +4, installs the derived vtable and clears the payload word at +8. |
| `008b15b0` | `PacketMembershipActivation::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketMembershipActivation::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
