# Class: SoeUtil::List<AssetManagement::AssetRequestInternal::DeliverableAsset,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd310` | `DeliverableAssetList::AllocateNode` | ✅ | virtual | Initializes allocator state, tries thread and global allocation, then allocates 0x20 bytes aligned to 4. |
| `008bd350` | `DeliverableAssetList::FreeNode` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 4-byte alignment, then returns. |
| `00919870` | `SoeUtil::ListDeliverableAssetDtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the List vtable, then loops over the chain at +4 calling FUN_008faa30 as a thiscall on the container with the node pushed. |
| `0091a4c0` | `SoeUtil::ListDeliverableAssetScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the List vtable, loops over the +4 chain calling FUN_008faa30 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
