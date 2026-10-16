# Class: SoeUtil::Job

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083f180` | `— ⚠reject` | ⚪ | dtor | __fastcall destructor body (no free): SEH frame, calls 00fc5140 on a stack-cookie-derived argument, then stores the SoeUtil::Job vtable and the SoeUtil::RefCounted vtable into *this. Seven EH unwind funclets call it. |
| `0083f1e0` | `JobCtor` | 🟢 | ctor | __thiscall constructor: SEH frame, calls SoeUtil::RefCountedImplicit ctor 0071b950 first, then stores the SoeUtil::Job vtable, zeroes this+0x14..+0x20 and +0x8/+0xc, sets this+0x28 = 0xffffffff, stores param_2 at this+0… |
| `0083f280` | `JobScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor: sets the SoeUtil::Job vtable then the SoeUtil::RefCounted vtable on this; when bit 0 of the flag argument is set, calls _free(this) (00d82463 thunk). Returns this in EAX (bytes: add … |
| `01098f30` | `JobBaseDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, no return value. Stores the SoeUtil::Job vtable at this+0 and then overwrites it with the SoeUtil::RefCounted vtable (the final vtable is RefCounted); no other work. |
| `01099640` | `JobScalarDeletingDtor` | ✅ | dtor | Thiscall with one stack arg (flags byte), returns this. Stores the SoeUtil::Job vtable at this and then the SoeUtil::RefCounted vtable; when flags bit 0 is set calls _free(this). This body is shared: it is the slot [0] … |
