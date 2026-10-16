# Class: RepeatingActivityCountRemainingInfoPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad9c60` | `RepeatingActivityCountRemainingInfoPacket::~RepeatingActivityCountRemainingInfo…` | 🟢 | dtor | Resets the vtable to BasePacket::vftable (67-byte body); dispatcher 00adbf00 calls it on the info packet it built via 00ada590 and one EH unwind funclet also calls it. |
| `00ada590` | `— ⚠reject` | ⚪ | ctor | Runs admin base constructor 00ada120 with activity type 10 and two arguments, then installs RepeatingActivityCountRemainingInfoPacket::vftable; the dispatcher calls it with -1 placeholders before parsing. |
| `00ada610` | `RepeatingActivityCountRemainingInfoPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivityCountRemainingInfoPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adb820` | `RepeatingActivityCountRemainingInfoPacket_Unserialize` | 🟢 | function | Builds a local stream over the buffer/length arguments, parses the admin header (00adb160: id, two dwords, 64-bit Guid) and returns 1 only when no error was flagged and (unless partial input is allowed) the cursor consu… |
