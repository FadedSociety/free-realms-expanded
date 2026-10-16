# Class: AchievementInitializePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bad510` | `AchievementInitializePacket_Ctor` | ✅ | ctor | Constructor: calls 00ba97d0 with argument 4 on ECX (base constructor, not resolved here), stores the AchievementInitializePacket vtable (01824bf4) and constructs the SoeUtil::List<AchievementData,32> member at +0xc thro… |
| `00bad570` | `AchievementInitializePacket_DtorBody` | 🟢 | dtor | Destructor body: calls 00bad2d0 (List<AchievementData,32> destructor body) on the member at this+0xc, then stores the BasePacket vtable (018011ec) into the object. |
| `00bad8d0` | `AchievementInitializePacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00bad570 on ECX, then _free when the delete flag bit 0 is set, and returns the object pointer. |
