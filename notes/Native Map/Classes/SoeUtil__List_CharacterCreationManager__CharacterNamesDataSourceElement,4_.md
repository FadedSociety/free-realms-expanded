# Class: SoeUtil::List<CharacterCreationManager::CharacterNamesDataSourceElement,4>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00981300` | `List_CharacterNamesElement4_Ctor` | ✅ | ctor | Constructor of the pooled list variant: installs the List<...,-1> vtable, zeroes head/tail/count, installs the List<...,4> vtable, zeroes the pool fields (+0x10..+0x20) and builds the inline free list of four 0xc-byte n… |
| `009813c0` | `List_CharacterNamesElement4_Dtor` | 🟢 | dtor | Destructor of the pooled name-element list: installs the List<...,4> vtable, drains the list nodes with 00980f80, frees the external pool block tree (root at +0x10: children freed with FUN_00767c60 then FUN_00401c00(roo… |
| `009814d0` | `List_CharacterNamesElement4_ScalarDeletingDtor` | 🟢 | dtor | Deleting destructor wrapper: runs the pooled-list destructor 009813c0 and frees the object with _free when the delete flag is set. |
