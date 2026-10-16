# Class: FriendAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7f8e0` | `FriendAddPacket::FriendAddPacket` | ✅ | ctor | Calls the packet base initializer with 6, installs FriendAddPacket::vftable, then initializes the shared embedded fields through FUN_00c7f750. |
| `00c7f940` | `FriendAddPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the FriendAddPacket destructor and frees the object only when flag bit 0 is set; returns the object pointer otherwise. |
