# Class: ClientUpdatePacketUpdateActionBarSlot

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e7f0` | `ClientUpdatePacketUpdateActionBarSlot::ClientUpdatePacketUpdateActionBarSlot` | ✅ | ctor | Packet constructor that also builds an embedded ActionBarSlot: calls the client-update base constructor FUN_0089d180(0x19) (family 0x26, sub-opcode 25), stores 0 in param_1[3] and -1 in param_1[4], installs ClientUpdate… |
| `0089e860` | `ClientUpdatePacketUpdateActionBarSlot::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: destroys the embedded ActionBarSlot member at +0x18 by calling FUN_0101cd70 with that member as this (it rewrites ActionBarSlot::vftable), then writes BaseClientUpdatePacket::vftable and BaseP… |
| `008b2920` | `ClientUpdatePacketUpdateActionBarSlot::ScalarDeletingDestructor` | 🟢 | dtor | Calls the packet destructor at 0089e860 (which destroys the packet member through 0101cd70 and then resets the BaseClientUpdatePacket and BasePacket vtables) and, when bit 0 of the flag argument is set, calls operator d… |
