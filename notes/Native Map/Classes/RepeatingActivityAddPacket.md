# Class: RepeatingActivityAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ada9b0` | `RepeatingActivityAddPacket::RepeatingActivityAddPacket` | ✅ | ctor | Runs state-packet constructor 00ad9740, installs RepeatingActivityAddPacket::vftable, then default-constructs two IString members at +0x20 and +0x30 (vtable plus the static empty string at 0x1be7f84) and sets the activi… |
| `00adaa50` | `RepeatingActivityAddPacket::~RepeatingActivityAddPacket` | ✅ | dtor | Releases the two reference-counted IString members at +0x30 and +0x20 (atomic decrement of the refcount in front of the buffer, virtual release when it hits zero) and then resets the vtable to BasePacket::vftable. |
| `00adad40` | `RepeatingActivityAddPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: calls the non-deleting destructor 00adaa50 and then _free(this) when bit 0 of the flag argument is set; it is vtable slot 0 of RepeatingActivityAddPacket. |
| `00adb740` | `RepeatingActivityAddPacket_Unserialize` | 🟢 | function | Builds a local stream over the buffer/length arguments, parses the add body with 00adafa0 and returns 1 only when no error was flagged and (unless partial input is allowed) the cursor consumed the whole buffer. |
