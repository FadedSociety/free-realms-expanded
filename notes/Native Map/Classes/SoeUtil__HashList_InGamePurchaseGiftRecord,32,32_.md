# Class: SoeUtil::HashList<InGamePurchaseGiftRecord,32,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34760` | `HashListGiftRecord32Ctor` | ✅ | ctor | Constructs the underlying 32-bucket hash-list map and installs the HashList vtable. |
| `00c347c0` | `HashListGiftRecord32ScalarDtor` | ✅ | dtor | Destroys the underlying map and frees the object when the low flag bit is set. |
| `00c34820` | `HashListGiftRecord32Ctor ? ⚠rename` | 🟡 | ctor | Sets the list wrapper's scalar field to zero and constructs its underlying 32-bucket map. |
