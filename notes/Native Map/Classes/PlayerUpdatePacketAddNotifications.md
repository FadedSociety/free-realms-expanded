# Class: PlayerUpdatePacketAddNotifications

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d06d0` | `PlayerUpdatePacketAddNotifications::PlayerUpdatePacketAddNotifications` | ✅ | ctor | Constructor of the AddNotifications player-update packet: runs the family constructor FUN_0089fbb0 with subtype 10, installs PlayerUpdatePacketAddNotifications::vftable, then installs the SoeUtil::Array<NotificationInfo… |
| `008d0740` | `PlayerUpdatePacketAddNotifications::~PlayerUpdatePacketAddNotifications` | ✅ | dtor | Destructor of the AddNotifications packet: zeroes the array count at +0x14, rewrites the SoeUtil::Array<NotificationInfo,0,1> vtable at +0xc, frees the array buffer at +0x10 through FUN_00401c00 with alignment 8 and nul… |
| `008dcbb0` | `PlayerUpdatePacketAddNotifications::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d0740 (labelled PlayerUpdatePacketAddNotifications destructor) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
