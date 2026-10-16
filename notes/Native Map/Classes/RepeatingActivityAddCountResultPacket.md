# Class: RepeatingActivityAddCountResultPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad9b70` | `RepeatingActivityAddCountResultPacket::~RepeatingActivityAddCountResultPacket` | 🟢 | dtor | Resets the vtable to BasePacket::vftable (67-byte body); dispatcher 00adbf00 calls it on the result packet it built via 00ada2e0 and one EH unwind funclet also calls it. |
| `00ada2e0` | `RepeatingActivityAddCountResultPacket::RepeatingActivityAddCountResultPacket` | ✅ | ctor | Runs admin base constructor 00ada120 with activity type 7, installs RepeatingActivityAddCountResultPacket::vftable and stores a success byte at +0x20; the dispatcher calls it with -1 placeholders before parsing. |
| `00ada370` | `RepeatingActivityAddCountResultPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivityAddCountResultPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adb890` | `RepeatingActivityAddCountResultPacket_Unserialize` | 🟢 | function | Builds a local stream over the buffer/length arguments, parses the admin header plus trailing bool with 00adb5d0 and returns 1 only when no error was flagged and (unless partial input is allowed) the cursor consumed the… |
