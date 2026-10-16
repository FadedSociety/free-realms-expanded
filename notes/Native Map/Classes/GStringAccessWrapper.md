# Class: GStringAccessWrapper

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00409165` | `— ⚠reject` | ⚪ | method | Virtual method: returns 1 when the type dword at +0x10 is null or when the 16-bit key cached at +0x0C equals param_2; otherwise tail-jumps to the object's virtual slot +4 (jmp dword ptr [eax+4], arguments preserved) - t… |
| `00409250` | `InvokeVirtual50` | ✅ | method | Calls the receiver's virtual slot +0x50 with (param_2, 0, -1). |
| `00415038` | `GStringAccessWrapper::ForwardAccessSlot6C` | 🟢 | virtual | When the data access object at +8 is present, forwards (param_2, param_3, param_4) to that object's vtable slot +0x6C and returns its result; returns 0 when no access is held. |
| `0041505a` | `GStringAccessWrapper::ForwardAccessSlot68` | 🟢 | virtual | When the data access object at +8 is present, forwards param_2 to that object's vtable slot +0x68 and returns its result; returns 0 when no access is held. |
| `004184f4` | `— ⚠reject` | ⚪ | ctor | Constructor of the string data-access wrapper: runs the GObjectBase constructor FUN_0040541a, stores the two-vtable pair 0x175DEA8/0x175DEA0, runs the GDataAccessWrapper base constructor FUN_0041493a, overwrites the pri… |
| `0041b41f` | `GStringAccessWrapper::ApplySlotValue` | 🟢 | virtual | When holder count at +0x10 is zero, clears the short at +0x0C, Releases the current interface at +8, then stores an AddRef'd incoming interface when the source short at +0x0C is zero, or stores that short otherwise. Whe… |
| `0041d2e3` | `GStringAccessWrapper::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_004149C8 on this, frees this through _free when param_2 bit 0 is set, and returns this otherwise. |
| `00420db2` | `GStringAccessWrapper::Ops40` | 🟢 | function | Dispatches construction and destruction operations for 0x28-byte GStringAccessWrapper objects. Ops 0 and 2 call default constructor FUN_0041493a then store the GStringAccessWrapper vtable; ops 1 and 3 instead call copy … |
| `0042ae63` | `GStringAccessWrapper::GStringAccessWrapper` | ✅ | ctor | Constructor of the GStringAccessWrapper class: runs the base initializer 0041493a, installs the GBitmap-free GStringAccessWrapper vtable at +0, allocates 0x34 bytes through 004d9dbd and constructs a GStringObject in the… |
