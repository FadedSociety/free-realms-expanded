# Class: AnnouncementDataSendPacket::AnnouncementPacketData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3eaf0` | `EmbeddedStrings3_Dtor_18_28_44` | 🟢 | dtor | Destructor of a record holding three SoeUtil::IString fields at +0x18, +0x28 and +0x44: each field gets the base IString vtable back and, when its length (+0x24, +0x34, +0x50) is positive, the shared buffer reference co… |
| `00a3ecc0` | `EmbeddedStrings3_Ctor_18_28_44` | 🟢 | ctor | Constructor of the three string fields at +0x18, +0x28 and +0x44: installs SoeUtil::IString<char>::vftable for each, points the buffers at the shared static empty buffer DAT_01be6a64 and zeroes the length/capacity pairs… |
| `00a3f0a0` | `ScalarDelete_a3f0a0` | 🟢 | dtor | Scalar deleting destructor under an SEH frame: calls the row destructor 00a3eaf0 and then calls _free(this) when bit 0 of the flag argument is set. |
