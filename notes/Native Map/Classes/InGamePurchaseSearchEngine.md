# Class: InGamePurchaseSearchEngine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc4c80` | `InGamePurchaseSearchEngine::InGamePurchaseSearchEngine` | ✅ | ctor | Calls 00c30040 with this as receiver, installs the InGamePurchaseSearchEngine vtable, and stores param_2 at this+0x1080. |
| `00cc4cf0` | `InGamePurchaseSearchEngine::~InGamePurchaseSearchEngine` | ✅ | dtor | Restores the InGamePurchaseSearchEngine vtable and calls 00c2fa70 with this as receiver to destroy the owned member. |
| `00cc4d40` | `InGamePurchaseSearchEngine::ScalarDeletingDestructor` | ✅ | dtor | Restores the class vtable, destroys the object member through 00c2fa70 with this as receiver, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |
| `00cc4da0` | `InGamePurchaseSearchEngine::FindBundleItem` | ✅ | virtual | Looks up the bundle by param_1, walks its linked item entries, searches the item-definition hash table by each entry key, writes the first matching definition field at +0x28 to param_2, and returns a success byte or zer… |
| `00cc4e40` | `InGamePurchaseSearchEngine::ForwardBundleFieldA0` | ✅ | virtual | Looks up a bundle by param_1; if found, obtains the object at manager+0x10 and invokes its vtable slot +8 with bundle field +0xa0 and param_2. |
| `00cc50f0` | `InGamePurchaseSearchEngine::SearchBundles` | ✅ | virtual | Resolves the requested bundle, checks the active store context, queries localized strings and associated item maps, then appends matching identifiers and display strings to result vectors at param_1 offsets +4, +8, +0xc… |
