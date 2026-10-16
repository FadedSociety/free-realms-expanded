# Class: LoadingScreenData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a5c5d0` | `LoadingScreenDataCtor` | ✅ | ctor | Initializes the delivery-handler base, installs LoadingScreenData vtables, sets default flags, initializes a string, and stores a field from a global manager. |
| `00a5c680` | `LoadingScreenDataDtorAdjustorThunk` | ✅ | function | Subtracts 8 from ECX, then jumps to 00a5c730; bytes are 83 e9 08 followed by a relative jump to that target. |
| `00a5c690` | `LoadingScreenDataBaseDtor → LoadingScreenData::DtorBody` | 🟢 | dtor | Releases its reference-counted string when retained, runs another member cleanup, then installs the RefCounted base vtable. |
| `00a5c730` | `LoadingScreenData::ScalarDeletingDtor` | ✅ | dtor | Calls the LoadingScreenData cleanup body, frees the object when flag bit 0 is set, otherwise returns its pointer. |
| `00ca8050` | `LoadingScreenData::ResetState ?` | 🟡 | virtual | Virtual slot 4 of LoadingScreenData: stores 0xffffffff at this+0x10 and returns with ret 0xc (pops 12 bytes of arguments). |
| `00ca99e0` | `LoadingScreenData::VirtualLoadOrRelease` | 🟢 | virtual | Virtual slot 3 of LoadingScreenData: when the mode argument is 0 it calls 00ca97e0 on the argument; when it is 1 it decrements the counter at this+0x24 and, when that reaches zero and the state at +0x10 is not -1, sets … |
