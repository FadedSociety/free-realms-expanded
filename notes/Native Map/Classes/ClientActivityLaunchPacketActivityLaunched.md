# Class: ClientActivityLaunchPacketActivityLaunched

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4b300` | `ClientActivityLaunchPacketActivityLaunched::ClientActivityLaunchPacketActivityL…` | ✅ | ctor | Constructor: family base 009c0c30 with sub-opcode 5, installs its vtable, and initialises an embedded SoeUtil guid list at +0x14 (list vtable plus zeroed head at +0x18, tail at +0x1c and count at +0x20). |
| `00c4b380` | `ClientActivityLaunchPacketActivityLaunched::~ClientActivityLaunchPacketActivity…` | 🟢 | dtor | Destructor: re-installs the embedded guid-list vtable at +0x14, destroys that list through 008d3ee0 and rewrites *this to BasePacket::vftable. |
| `00c4bb50` | `ClientActivityLaunchPacketActivityLaunched::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4b380 (which destroys the embedded guid list) and frees the object with CRT _free when bit 0 of the flag is set. |
