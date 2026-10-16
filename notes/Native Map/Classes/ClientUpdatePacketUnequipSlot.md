# Class: ClientUpdatePacketUnequipSlot

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d7a0` | `ClientUpdatePacketUnequipSlotCtor` | ✅ | ctor | Constructor of ClientUpdatePacketUnequipSlot: runs the base constructor FUN_0089d180 with sub-opcode 6, then installs ClientUpdatePacketUnequipSlot::vftable. |
| `0089d800` | `ClientUpdatePacketUnequipSlotDtor` | ✅ | dtor | Destructor body: installs ClientUpdatePacketUnequipSlot::vftable, then BaseClientUpdatePacket::vftable and BasePacket::vftable; no member cleanup. |
| `0089d860` | `ClientUpdatePacketUnequipSlotScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the three vtables (derived, family, base) and calls _free when bit 0 of param_2 is set. |
