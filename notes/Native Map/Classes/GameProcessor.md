# Class: GameProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9efd0` | `GameProcessor::GameProcessor` | ✅ | ctor | Constructs the GameProcessor base object: stores param_2 at +4, stores GameProcessor::vftable at *this, clears the byte at +8, sets the dwords at +0xc and +0x10 to -1, clears +0x14, stores SoeUtil::Array<SoeUtil::IStrin… |
| `00b9f050` | `— ⚠reject` | ⚪ | dtor | Destroys the GameProcessor base object: stores GameProcessor::vftable, calls FUN_00ce1e10 (teardown of the +0x14 member), resets the embedded string array at +0x18 to the Array vtable, calls FUN_0071b470 on the pointer … |
| `00b9f340` | `GameProcessor::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls 00b9f050(this) (the GameProcessor destructor) and, when the low byte of param_2 is set, calls _free(this); returns this. |
| `00ce1cb0` | `GameProcessor::GetByte8` | 🟢 | virtual | Virtual slot [4] (no stack arguments): returns the byte at this+8 (mov al,[ecx+8]; ret). |
| `00ce1e10` | `GameProcessor::ReleaseOwnedMember` | 🟢 | virtual | Virtual slot [3] (no stack arguments): if vtbl+0x10 of this returns nonzero, calls FUN_010c2170, calls FUN_010c2610 and _free on the object at [this+0x14] when nonnull and nulls [this+0x14], then clears byte this+8, cal… |
| `00ce2810` | `GameProcessor::LoadPreloadForKey` | 🟢 | virtual | Virtual slot [2] (two stack arguments): when the virtual slot-4 flag is clear it stores param_2 at this+0xc and param_3 at this+0x10, looks up a node by param_2 with FUN_009ad170, and when that node string at +0xbc has … |
