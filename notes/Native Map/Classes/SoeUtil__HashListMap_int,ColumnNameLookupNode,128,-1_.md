# Class: SoeUtil::HashListMap<int,ColumnNameLookupNode,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbde0` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x24 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bbe20` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `00919570` | `SoeUtil::HashListMapColumnNameLookupNode128DtorBody` | ✅ | dtor | Destructor body: stores the HashListMap<int,ColumnNameLookupNode,128,-1> vtable, then loops over the node chain at +8 calling FUN_008fb8d0 on each node until the chain is empty. |
| `0091a240` | `SoeUtil::HashListMapColumnNameLookupNode128ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fb8d0 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
| `0091bef0` | `HashListMap_ColumnNameLookup_Dtor` | ✅ | dtor | Destructor of the SoeUtil::HashListMap<int,ColumnNameLookupNode,128,-1> hash container: stores the class vftable (so later base destructors can override it) and walks the node list at +8 draining entries through FUN_008… |
| `0091e7c0` | `HashListMap_ColumnNameLookup_Ctor` | ✅ | ctor | Constructor of the column-name lookup hash container: writes the HashListMap vtable, zeroes +4/+8/+0xc/+0x10, memsets the 128-entry bucket array at +0x14 for 0x200 bytes and finally switches the vtable to the SoeUtil::H… |
