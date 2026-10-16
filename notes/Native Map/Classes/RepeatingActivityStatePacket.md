# Class: RepeatingActivityStatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad9740` | `— ⚠reject` | ⚪ | ctor | Runs base constructor 00ad9620 with activity type 2, installs RepeatingActivityStatePacket::vftable, stores the activity id parameter at +0xc and zeroes the following five dwords (+0x10..+0x1c). |
| `00ad97b0` | `RepeatingActivityStatePacket::~RepeatingActivityStatePacket` | 🟢 | dtor | Resets the object's vtable to BasePacket::vftable (67-byte body, no member cleanup); it is the cleanup the dispatcher 00adbf00 runs on the state packet it built and is also reached from an EH unwind funclet. |
| `00ad9f10` | `RepeatingActivityStatePacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivityStatePacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adb6d0` | `RepeatingActivityStatePacket_Unserialize` | 🟢 | function | Builds a local stream over the buffer/length arguments, parses the common state fields with 00adaed0 and returns 1 only when no error was flagged and (unless partial input is allowed) the cursor reached the end of the b… |
