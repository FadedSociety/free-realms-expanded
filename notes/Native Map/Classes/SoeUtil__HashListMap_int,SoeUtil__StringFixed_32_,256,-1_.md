# Class: SoeUtil::HashListMap<int,SoeUtil::StringFixed<32>,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097d010` | `HashListMap_StringFixed32_Ctor` | ✅ | ctor | Constructor for the 256-bucket hash list: stores the HashListMap<int,StringFixed<32>,256,-1> vtable at offset 0, zeroes the head/tail/count fields at +4..+0x10, memsets the 256 bucket heads at +0x14 to zero (0x400 bytes… |
| `0097dc20` | `HashListMap_intSoeUtil_StringFixed_32256m1_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `009801d0` | `HashList256_RemoveNode` | 🟢 | method | Removes node param_2 from the 256-bucket container: unlinks it from bucket (key at +0x3c, &0xff, chain +0x40), unlinks the list links (node+0x38 previous, node+0x34 next, header +8/+0xc), decrements the count, destroys … |
| `009811f0` | `HashList256_Clear` | 🟢 | method | Clear helper: repeatedly removes the head node (header +8) with 009801d0 until the 256-bucket container is empty. |
| `00981550` | `HashListMap_StringFixed32_Dtor` | ✅ | dtor | Destructor body: installs the StringFixed<32> hash-list vtable and removes every node through 009801d0. |
| `00981630` | `HashListMap_StringFixed32_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor: installs the StringFixed<32> hash-list vtable, removes every node with 009801d0 and frees the object when the delete flag is set. |
| `00981710` | `HashListMap_StringFixed32_Dtor_EH` | 🟢 | dtor | Destructor body with SEH frame: installs the StringFixed<32> hash-list vtable and removes every node through 009801d0. |
| `00981800` | `HashListMap_StringFixed32_ScalarDeletingDtor2 → HashList_StringFixed32_ScalarDe…` | 🟢 | dtor | Deleting destructor wrapper: runs 00981710 and frees the object with _free when the delete flag is set. |
