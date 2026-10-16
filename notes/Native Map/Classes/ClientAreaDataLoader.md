# Class: ClientAreaDataLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1970` | `ClientAreaDataLoader::ClientAreaDataLoader` | ✅ | ctor | Constructor of the client area-data loader: installs the base interface vtables ClientAreaDataManagerInterface (primary slot) and AssetManagement::AssetDeliveryHandlerInterface (secondary slot), stores the two arguments… |
| `008d1a10` | `ClientAreaDataLoader::ScalarDeletingDtorThunk` | ✅ | dtor | Eight-byte adjustor thunk decoded from the raw bytes (83 e9 04 / e9 a8 00 00 00): subtracts 4 from this (converting the secondary interface subobject pointer at +4 back to the object start) and jumps to the real scalar … |
| `008d1a20` | `ClientAreaDataLoader::~ClientAreaDataLoader` | ✅ | dtor | Destructor body of the area-data loader: rewrites both vtable slots to ClientAreaDataLoader::vftable, releases the embedded SoeUtil::IString<char> at +0x14 (param_1[5], buffer +0x18, capacity +0x20) through the LOCK ref… |
| `008d1ac0` | `ClientAreaDataLoader::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the primary vtable): runs the destructor body FUN_008d1a20 and frees the object through the CRT _free thunk when bit 0 of the flag argument is set. |
| `00b09ec0` | `ClearLoaderStateFields ?` | 🟡 | virtual | Clears object byte at offset 0x24 and dword at offset 0x28. |
| `00b0c010` | `ClientAreaDataLoader::OnData` | ✅ | virtual | Copies a received byte buffer into a null-terminated array, conditionally calls the area shape loader, then releases temporary storage. |
