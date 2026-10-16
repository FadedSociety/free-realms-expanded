# Class: ControllerDraw

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00afcb90` | `ControllerDraw::UpdateFlag3b0FromCounters ?` | 🟡 | virtual | Thiscall, one stack arg (ret 4). Calls 00afe5d0(param_2) first (thiscall with ECX=this). Then: if byte +0x218 is 0 and flag +0x3b0 is 1 and dword +0x204 is above 14, sets +0x3b0 to 0; else if +0x218 is 1 and +0x3b0 is 0… |
| `00afcc80` | `ControllerDraw::BuildPoseOutputs ?` | 🟡 | virtual | Thiscall with three stack args (ret 0xc), vtable slot 67 (+0x10c). Reads 4 floats from the object at 0094cf60() (+0x610..+0x61c, scaled by DAT_01762a34) and transforms this+0x16c with 00770e70; computes a normalized qua… |
| `00afcff0` | `ControllerDraw::ControllerDraw` | ✅ | ctor | Thiscall ctor, 3 stack args forwarded to base 00afe470, returns this. Stores ControllerDraw::vftable at +0x00, clears the byte at +0x3b0, installs SoeUtil::Array<float,0,1>::vftable at +0x3b4 with data +0x3b8, size +0x3… |
| `00afd200` | `ControllerDraw::~ControllerDraw` | ✅ | dtor | Fastcall dtor body (ECX=this): restores ControllerDraw::vftable at +0x00 and Array vftable at +0x3b4, zeroes the size at +0x3bc, calls 00401c00(data, 4) to free the data buffer, clears data at +0x3b8, then calls base te… |
| `00afd290` | `ControllerDraw::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting dtor shape (ret 4, flag byte arg): calls dtor body 00afd200 on this, then calls _free(this) (00d82463) when flag bit 0 is set. Returns this. |
| `00afd470` | `— ⚠reject` | ⚪ | virtual | Fastcall with ECX=this, vtable slot 53 (+0xd4). Calls virtuals on this: +0x5c(DAT_0176b1b0), +0x68(_DAT_0181c7f4), +0xf0(0), +0xdc(1), +0xe0(1), +0xe4(1). Reads the owner object [[this+0x74]+0xc8], takes floats from it … |
