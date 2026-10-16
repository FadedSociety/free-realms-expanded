# Class: MemberPanelDataSendPacket::MemberPanelData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3ebd0` | `EmbeddedStrings2_Dtor_8_24` | 🟢 | dtor | Destructor of a record with two SoeUtil::IString fields at +8 and +0x24: each field gets the base IString vtable back and, when its length (+0x14, +0x30) is positive, the buffer reference count at buffer-4 is decremente… |
| `00a3ed40` | `EmbeddedStrings2_Ctor_8_24` | 🟢 | ctor | Constructor of the two string fields at +8 and +0x24: installs SoeUtil::IString<char>::vftable for each, points the buffers at the shared static empty buffer DAT_01be6a64 and zeroes the length/capacity pairs (+0x10/+0x1… |
| `00a3f100` | `ScalarDelete_a3f100` | 🟢 | dtor | Scalar deleting destructor under an SEH frame: calls the member-panel row destructor 00a3ebd0 and then calls _free(this) when bit 0 of the flag argument is set. |
| `00a3f560` | `MemberPanelData_ReadFields` | 🟢 | method | Decodes one fixed-size record from the raw reader at param_2 into param_1: reads a dword into param_1[0] and param_1[1], then a length-prefixed string block assigned through the IString helper FUN_00894b10 (kept as FUN_… |
