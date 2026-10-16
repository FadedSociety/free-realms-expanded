# Class: SoeUtil::List<SoeUtil::Math::Vector<float,4>,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009cc1c0` | `Vector4List32Ctor` | ✅ | ctor | Installs base and 32-node list vtables, zeros list fields, initializes the embedded node pool, and returns the list pointer. |
| `009ccc60` | `Vector4List32::AdjustorThunkPop` | ✅ | virtual | Adds 0x10 to ECX and jumps to 009ccb00; raw bytes show an adjustor thunk to the free-chain pop method. |
| `009ccc70` | `Vector4List32::AdjustorThunkPush` | ✅ | virtual | Adds 0x10 to ECX and jumps to 009ccb50; raw bytes show an adjustor thunk to the free-chain push method. |
| `009ccc80` | `Vector4List32::ScalarDeletingDtor` | ✅ | dtor | Calls the Vector4 list destructor, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
