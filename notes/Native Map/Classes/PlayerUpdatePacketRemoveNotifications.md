# Class: PlayerUpdatePacketRemoveNotifications

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d07c0` | `PlayerUpdatePacketRemoveNotifications::PlayerUpdatePacketRemoveNotifications` | ✅ | ctor | Constructor of the RemoveNotifications player-update packet: runs the family constructor FUN_0089fbb0 with subtype 0xb (11), installs PlayerUpdatePacketRemoveNotifications::vftable, then installs the SoeUtil::Array<Play… |
| `008d0830` | `PlayerUpdatePacketRemoveNotifications::~PlayerUpdatePacketRemoveNotifications` | ✅ | dtor | Destructor of the RemoveNotifications packet: zeroes the array count at +0x14, rewrites the SoeUtil::Array<PlayerUpdatePacketRemoveNotifications::Entry,0,1> vtable at +0xc, frees the entry buffer through FUN_00401c00 wi… |
| `008dcbd0` | `PlayerUpdatePacketRemoveNotifications::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d0830 (labelled PlayerUpdatePacketRemoveNotifications destructor) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
