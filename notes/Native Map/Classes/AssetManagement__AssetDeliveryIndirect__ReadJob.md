# Class: AssetManagement::AssetDeliveryIndirect::ReadJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01099730` | `ReadJobDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, no return value. Stores the ReadJob vtable (01884dc8) at this, releases the object at this+0xb0 when set (refcount at +0x14, vtbl[1] when zero) and clears this+0xb0, calls FUN_004022b… |
| `01099da0` | `ReadJob::ReadJob` | ✅ | ctor | Constructor: calls 010995e0 (PackJob base constructor, which installs the PackJob vtable), installs the ReadJob vtable, constructs a StringFixed<64> at offset 0 (00402a10), stores param_2 at +0x58, assigns the name para… |
| `01099e30` | `ReadJob::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: calls the ReadJob body 01099730, then calls _free on the object when bit 0 of the stack argument is set; returns the object pointer. |
| `01099e50` | `ReadJob::ReadIntoBuffer ?` | 🟡 | virtual | Virtual slot [2] of ReadJob. Allocates a 0x18-byte object (00d833a6 then the 0083fe90 constructor), stores it at this+0xb0, calls 010fb8e0 with this+0x60 and the new buffer; on failure releases the buffer reference at +… |
