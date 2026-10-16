# Class: BroadcastPacketWorld

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dd480` | `BroadcastPacketWorld::BroadcastPacketWorld` | ✅ | ctor | Packet constructor: calls the base broadcast constructor 008dd400 with sub-opcode 3, installs BroadcastPacketWorld::vftable, zeroes +0x1ac/+0x1b0, sets the byte at +0x1b4 to 1 and clears the byte at +0x1b5. |
| `008dd4f0` | `BroadcastPacketWorld::ScalarDeletingDtor` | ✅ | dtor | SEH-framed scalar deleting destructor: runs the destructor body 008d1b30 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
