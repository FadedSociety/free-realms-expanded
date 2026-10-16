# Class: SoeUtil::HashListMap<int,ClientQuestData,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79fc0` | `SoeUtil::HashListMap_int_ClientQuestData_256_neg1::Ctor` | ✅ | ctor | Constructor: stores the SoeUtil::HashListMap<int,ClientQuestData,256,-1> vtable, zeroes this+8, +0xc, +0x10 and +4, and memsets 0x400 bytes (256 bucket heads) at this+0x14. |
| `00c7c470` | `ClientQuestMap::Destruct → SoeUtil::HashListMap<int,ClientQuestData,256,-1>::De…` | ✅ | dtor | Installs the typed hash-map vtable and repeatedly destroys the first map node through FUN_0092ac70 until the bucket list is empty. |
| `00c7c4e0` | `ClientQuestMap::DeletingDtor → SoeUtil::HashListMap<int,ClientQuestData,256,-1>…` | ✅ | dtor | Installs the typed map vtable, drains and destroys stored nodes through FUN_0092ac70, and frees the map only when flag bit 0 is set. |
