# Class: RepeatingActivityConsecutiveMsgPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad98d0` | `RepeatingActivityConsecutiveMsgPacket::RepeatingActivityConsecutiveMsgPacket` | ✅ | ctor | Runs base constructor 00ad9620 with activity type 3, installs RepeatingActivityConsecutiveMsgPacket::vftable, stores the activity id at +0xc and zeroes six dwords (+0x10..+0x24). |
| `00ad9940` | `RepeatingActivityConsecutiveMsgPacket::~RepeatingActivityConsecutiveMsgPacket` | 🟢 | dtor | Resets the object's vtable to BasePacket::vftable (67-byte body); dispatcher 00adbf00 calls it on the consecutive-message packet it built and one EH unwind funclet also calls it. |
| `00ad9f70` | `RepeatingActivityConsecutiveMsgPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivityConsecutiveMsgPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adb7b0` | `RepeatingActivityConsecutiveMsgPacket_Unserialize` | 🟢 | function | Builds a local stream over the buffer/length arguments, parses the six dwords with 00adb070 and returns 1 only when no error was flagged and (unless partial input is allowed) the cursor consumed the whole buffer. |
