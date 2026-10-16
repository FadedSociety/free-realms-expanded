# Class: WhatsNewInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e990` | `EmbeddedStrings3_Dtor_18_34_44` | ✅ | dtor | Destructor of a record holding three SoeUtil::IString fields at +0x18, +0x34 and +0x44: for each field it installs the base IString vtable, and when the length field (+0x24, +0x40, +0x50) is positive it decrements the r… |
| `00a3ea70` | `EmbeddedStrings3_Ctor_18_34_44` | ✅ | ctor | Constructor of the three string fields at +0x18, +0x34 and +0x44: for each it installs SoeUtil::IString<char>::vftable, points the buffer at the shared static empty buffer DAT_01be6a64 and zeroes the length and capacity… |
| `00a3edb0` | `ScalarDelete_a3edb0` | 🟢 | dtor | Scalar deleting destructor: calls the three-string destructor body 00a3e990 and then calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
| `00a3efb0` | `EmbeddedStrings3_CopyCtor_18_34_44` | ✅ | ctor | Copy-constructs a row: copies six leading dwords, initializes and copies the IString at +0x18, copies three dwords at +0x28..+0x30, then initializes and copies the IStrings at +0x34 and +0x44; returns the destination. |
