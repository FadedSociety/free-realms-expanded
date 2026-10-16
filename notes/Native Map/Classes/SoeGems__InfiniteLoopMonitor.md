# Class: SoeGems::InfiniteLoopMonitor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d04710` | `InfiniteLoopMonitorPoll` | ✅ | virtual | Virtual slot [2] of SoeGems::InfiniteLoopMonitor: loops until 00fc4ac0 returns non-zero; each pass, when the threshold at this+0xc4 is positive, takes a 64-bit time sample with 007027f0, compares it with the stored samp… |
| `00d04830` | `InfiniteLoopMonitorCtor` | ✅ | ctor | Stores the InfiniteLoopMonitor vtable at offset 0, param_2 at +0xc4 (threshold) and param_3 at +0xec, installs the SoeUtil::IString<char> vtable at +0xd0 with an empty buffer, clears the flag at +0xe8, and stores a 64-b… |
| `00d048d0` | `InfiniteLoopMonitorDtorBody` | ✅ | dtor | Non-deleting destructor body: resets the InfiniteLoopMonitor vtable and the IString vtable, releases the string buffer at +0xd4 when its count at +0xdc is positive (LOCK decrement, destroy through IString vtable slot 2 … |
| `00d04920` | `InfiniteLoopMonitorScalarDtor` | ✅ | dtor | Scalar deleting destructor: resets the InfiniteLoopMonitor and IString vtables, releases the string buffer as in 00d048d0, calls 00fc4c30, and calls _free on the object when bit 0 of the flag argument is set. |
| `00d049c0` | `InfiniteLoopMonitorCtorNamed` | ✅ | ctor | Calls 00fc4cc0 with the literal "InfiniteLoopMonitor" (argument form 0,2,name), installs the InfiniteLoopMonitor and IString vtables, sets the threshold at +0xc4 to 0 (polling off), assigns param_2 to the string at +0xd… |
| `00d04a90` | `InfiniteLoopMonitorCtorAssign` | ✅ | ctor | Calls 00fc4cc0(0,2,0), installs the InfiniteLoopMonitor and IString vtables, stores param_2 at +0xc4, assigns param_3 to the string at +0xd4 through 00402ce0, stores param_4 at +0xec, stores a 64-bit tick from 007027f0 … |
