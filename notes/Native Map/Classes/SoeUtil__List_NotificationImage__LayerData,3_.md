# Class: SoeUtil::List<NotificationImage::LayerData,3>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bf6220` | `— ⚠reject` | ⚪ | dtor | Destructor body. Stores the List<NotificationImage::LayerData,3> vftable at [ECX], calls FUN_00bf54e0 (EH bookkeeping), zeroes this+0x18 and +0x1c, and if the pointer at this+0x10 is nonzero calls FUN_00bf53a0 on its fi… |
| `00bf62d0` | `ListLayerData3ThunkPop` | 🟢 | virtual | Adjustor thunk: add ecx,0x10 then jmp 00bf6170 (ListNodePoolPop). It has no body of its own (bytes 83 c1 10 e9 ...). |
| `00bf62e0` | `ListLayerData3ThunkPush` | 🟢 | virtual | Adjustor thunk: add ecx,0x10 then jmp 00bf61c0 (ListNodePoolPush). It has no body of its own (bytes 83 c1 10 e9 ...). |
| `00bf62f0` | `ListLayerData3ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls 00bf6220 (the dtor body) on ECX, then if bit 0 of the stack argument is set calls _free on the object. Returns this (mov eax,esi; ret 4). |
