# Class: GameEventSequencer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b3040` | `GameEventSequencer::GameEventSequencer` | ✅ | ctor | Installs GameEventSequencer::vftable, zeroes +4, +8, +0xc, +0x10, +0x14 and the 0x80-byte block starting at +0x18 via _memset, then zeroes +0x98 and +0x9c. Returns this. |
| `008c4680` | `GameEventSequencer::Destructor` | ✅ | dtor | Restores GameEventSequencer vtable, walks the event chain at +0x24 and calls 0x007e7ea0(event,1,1) for each. Then invokes the owned member at +4 through virtual slot 0 with argument 1 and clears +4. |
| `008c4710` | `GameEventSequencer::ScalarDeletingDestructor` | ✅ | dtor | Calls GameEventSequencer cleanup, then frees this when deleting-flag bit 0 is set; otherwise returns this. |
