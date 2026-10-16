# Class: AssetManagement::AssetDeliveryIndirect::WriteJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01098f80` | `— ⚠reject` | ⚪ | virtual | Fastcall (ECX=write job), no stack args, returns a pointer in EAX. Reads the payload pointer and size from the object at job+0xb4 (+4 and +8); calls FUN_010fd740 with job+0x64 and that payload. On failure logs 'Error, a… |
| `010997f0` | `WriteJobDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, no return value. Stores the WriteJob vtable (01884de8) at this, releases the object at this+0xb4 when set (refcount at +0x14, vtbl[1] when zero) and clears this+0xb4, calls FUN_004022… |
| `01099ef0` | `— ⚠reject` | ⚪ | ctor | Constructor: installs the WriteJob vtable, constructs a StringFixed<64> at offset 0 (00402a10), stores param_2 at +0x5c and param_5 at +0x58, assigns the name param_3 (00402ce0), stores the data buffer param_4 at +0xb4 … |
| `01099f90` | `WriteJob::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: calls the WriteJob body 010997f0, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |
