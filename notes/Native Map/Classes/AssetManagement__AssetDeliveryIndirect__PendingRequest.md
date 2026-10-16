# Class: AssetManagement::AssetDeliveryIndirect::PendingRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01099b00` | `AssetDeliveryIndirect::PendingRequestCtor` | 🟢 | ctor | Thiscall (ECX=this), three stack args (param_2 C string, param_3 stored at +8, param_4 stored at +0x60), returns this. Stores the PendingRequest vtable (01884e28) at this, constructs a StringFixed<64> member through FUN… |
| `01099b80` | `PendingRequestDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, no return value. Stores the PendingRequest vtable (01884e28) at this, sets a local pointer to this+0xc, and calls FUN_004022b0 (IStringFixed64 destructor body, label confirmed) on the… |
| `01099be0` | `PendingRequest::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: calls the PendingRequest body 01099b80 unconditionally, then calls _free on the object when bit 0 of the stack argument is set; returns the object pointer. |
