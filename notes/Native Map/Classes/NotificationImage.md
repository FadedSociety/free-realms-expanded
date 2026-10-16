# Class: NotificationImage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bf6310` | `NotificationImage::Ctor` | ✅ | ctor | Constructor body. Stores AssetManagement::AssetRequestInterface::AssetRequester::vftable, then sets fields +0x4 = param5+0x1c4, +0x8 = arg2, +0xc = param5+0x268, +0x10 = arg3, +0x14 = arg4, zeroes +0x18..+0x24 and bytes… |
| `00bf63d0` | `NotificationImage::DtorBody` | ✅ | dtor | Destructor body. Stores NotificationImage::vftable. When the byte at +0x29 is set, calls 00769710 and 00984960 on the pair at +0x18/+0x1c and, when the returned object's vcall +0x30 returns nonzero, calls it again and F… |
| `00bf6570` | `NotificationImage::VirtualRegisterChild ? ⚠rename` | 🟡 | virtual | NotificationImage::vftable[1]. When arg2 is nonzero: LOCK-increments the refcount at arg2+0x4, appends arg2 to the array at +0x88 (count +0x8c; grows through FUN_00bf4d70 when count+1 exceeds the capacity at +0x90), the… |
| `00bf65f0` | `NotificationImage::VirtualDecAndRelease ?` | 🟡 | virtual | NotificationImage::vftable[2]. Decrements the dword at this+0x24, calls FUN_00bf5680, and returns with ret 8. |
| `00bf6680` | `NotificationImage::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls 00bf63d0 (dtor body) on ECX, frees the object when bit 0 of the stack argument is set, and returns this (ret 4). |
