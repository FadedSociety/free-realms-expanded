# Class: CommandPacketPlaySoundAtLocation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9b650` | `CommandPacketPlaySoundAtLocation::CommandPacketPlaySoundAtLocation` | ✅ | ctor | Calls 008a2820 with sub-opcode 7 and zero extra word, stores CommandPacketPlaySoundAtLocation::vftable, stores an empty SoeUtil::IString<char> at +0x10 (vtable, buffer DAT_01be7abc), and zeroes the words at +0x18 and +0… |
| `00a9b6d0` | `CommandPacketPlaySoundAtLocation::~CommandPacketPlaySoundAtLocation` | 🟢 | dtor | Stores the IString<char> vtable at +0x10, releases the string buffer at +0x14 when the count at +0x1c is positive (interlocked decrement and release through vtable slot +8 at the last reference), then stores BaseCommand… |
| `00a9c1a0` | `CommandPacketPlaySoundAtLocation::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9b6d0 (the PlaySoundAtLocation destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
