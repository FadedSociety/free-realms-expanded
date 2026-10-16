# Class: SoeUtil::Array<WhatsNewInfo,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e7f0` | `Array_WhatsNewInfo8_FreeBuffer` | ✅ | virtual | Compares the pointer argument with (this+0x13 rounded down to a multiple of 4); if different, calls FUN_00401c00(pointer,4), otherwise skips freeing inline storage. It ignores the other stack argument and returns with r… |
| `00a3fb40` | `SoeUtil::Array::~Array` | ✅ | dtor | Destructor of the 8-inline-slot Array<WhatsNewInfo,8,1>: installs SoeUtil::Array<WhatsNewInfo,8,1>::vftable, destroys live elements through FUN_00a3ef10(this+8), frees the buffer only when it is not the inline storage (… |
| `00a3fc10` | `SoeUtil::Array::Reserve` | ✅ | virtual | Virtual slot 1 of the Array<WhatsNewInfo,8,1> vtable: when the requested element count param_2 is at most 8 it reports the inline capacity through *param_3 = 8 and returns the inline storage address (this+0x13 rounded d… |
| `00a3ff00` | `SoeUtil::Array::Array` | ✅ | ctor | Constructor of the 8-inline-slot announcement array: installs SoeUtil::Array<WhatsNewInfo,0,1>::vftable, zeroes the buffer pointer, count and capacity at +4/+8/+0xc, then overwrites the vtable with SoeUtil::Array<WhatsN… |
| `00a3ff60` | `SoeUtil::Array::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of Array<WhatsNewInfo,8,1>: runs the full destructor 00a3fb40 and frees the object with _free when bit 0 of the delete flag is set. |
| `00a40220` | `SoeUtil::Array::AppendAndClearSource` | 🟢 | virtual | Virtual slot 3 of the Array<WhatsNewInfo,8,1> vtable: builds a temporary Array<WhatsNewInfo,0,1> on the stack, appends param_2's elements into the temporary through FUN_00a3fd80, truncates param_2 to empty through FUN_0… |
