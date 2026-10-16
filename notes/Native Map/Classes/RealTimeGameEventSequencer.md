# Class: RealTimeGameEventSequencer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c4730` | `RealTimeGameEventSequencer::Constructor` | ✅ | ctor | Calls base constructor 0x008b3040 with this in ECX and no stack arguments, installs RealTimeGameEventSequencer vtable, allocates 0x548 bytes, calls 0x00872850 on the allocated object in ECX with 0.0, and stores its retu… |
| `008c47c0` | `RealTimeGameEventSequencer::InvokeMemberSlot1` | ✅ | virtual | Loads the child pointer at this+4 and invokes its virtual slot 1 with the supplied argument and zero. |
| `008c47e0` | `RealTimeGameEventSequencer::Destructor` | ✅ | dtor | Restores RealTimeGameEventSequencer vtable, then calls base cleanup 0x008c4680 with this in ECX and no stack arguments. |
| `008c4830` | `RealTimeGameEventSequencer::ScalarDeletingDestructor` | ✅ | dtor | Installs RealTimeGameEventSequencer vtable, calls base cleanup 0x008c4680 with this in ECX and no stack arguments, then frees this when deleting-flag bit 0 is set; otherwise returns this. |
