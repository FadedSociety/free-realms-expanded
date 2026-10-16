# Class: AssetManagement::AssetDeliveryIndirect::DecompressJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010998b0` | `DecompressJobDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, no return value. Stores the DecompressJob vtable (01884e08) at this, releases the object at this+0xb0 when set (refcount at +0x14, vtbl[1] when zero) and clears this+0xb0, calls FUN_0… |
| `01099fb0` | `DecompressJob::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: calls the DecompressJob body 010998b0, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |
| `01099fd0` | `DecompressJob::Decompress` | ✅ | virtual | Virtual slot [2] of DecompressJob. Reads the input buffer at this+0xb0; 010ebd70 returns the output length from the envelope header, the output RefArray is grown to that length plus 4 when short (00701cc0), and 010ebee0… |
| `0109aa80` | `— ⚠reject` | ⚪ | ctor | Constructor: calls 010995e0 (PackJob base constructor), installs the DecompressJob vtable, constructs a StringFixed<64> at offset 0 (00402a10), assigns the name param_2 (00402ce0), stores the input buffer param_3 at +0x… |
