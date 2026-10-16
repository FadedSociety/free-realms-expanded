# Class: AdminGuildServerVersionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19d70` | `AdminGuildServerVersionPacket::AdminGuildServerVersionPacket` | ✅ | ctor | Constructor: calls the admin guild base constructor 00b174c0 with sub-opcode 0x29 (41), installs AdminGuildServerVersionPacket::vftable, initializes the embedded IString at +0xc and zeroes its length fields. |
| `00b19de0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the admin guild server version packet: rewrites the IString vtable at +0xc, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseAdminGuildP… |
| `00b1a4f0` | `AdminGuildServerVersionPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of AdminGuildServerVersionPacket::vftable: calls the destructor body 00b19de0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00b1abb0` | `— ⚠reject` | ⚪ | method | Reads the body of an admin guild version packet: two 16-bit header values through 00b19f00 and then a length-prefixed block skipped through 00894b10, flagging the reader when the block would run past the end. |
