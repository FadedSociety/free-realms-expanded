# Class: ClientRepeatingActivity

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c06d0` | `ClientRepeatingActivity::ScalarDeletingDestructor` | 🟢 | dtor | Calls 0x00adac80 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00adab00` | `ClientRepeatingActivity::ClientRepeatingActivity ⚠downgrade` | 🟢 | ctor | Default constructor of an activity entry: installs RepeatingActivityPlayer::vftable then ClientRepeatingActivity::vftable, sets id at +8 to -1, count at +0xc to 1, consecutive at +0x10 to 0, copies the default 64-bit pa… |
| `00adabc0` | `ClientRepeatingActivity::ClientRepeatingActivity` | ✅ | ctor | Constructor overload taking an activity id: stores the id at +8, installs RepeatingActivityPlayer::vftable then ClientRepeatingActivity::vftable, initialises count to 1 at +0xc, consecutive to 0 at +0x10, the default 64… |
| `00adac80` | `ClientRepeatingActivity::~ClientRepeatingActivity` | ✅ | dtor | Restores ClientRepeatingActivity::vftable, releases the two reference-counted IString members at +0x38 and +0x28, then resets the vtable to RepeatingActivityPlayer::vftable; called from EH unwind funclets and by the add… |
| `00adad60` | `ClientRepeatingActivity::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: calls the non-deleting destructor 00adac80 and then _free(this) when bit 0 of the flag argument is set; it is vtable slot 0 of ClientRepeatingActivity. |
| `00adad80` | `ClientRepeatingActivity::ClientRepeatingActivity` | ✅ | ctor | Copy constructor copies source dwords at +8,+0xc,+0x10,+0x18,+0x1c,+0x20,+0x24 and +0x48 through +0x58; it skips source +0x14. It installs the vtables, initializes destination string members, then calls 006f9e60 for sou… |
| `00adb530` | `ClientRepeatingActivity_CopyCtorWrapper` | 🟢 | ctor | SEH-framed wrapper that forwards to the copy constructor 00adad80 with the source entry pointer, returning this. |
