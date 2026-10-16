# Class: AssetManagement::AssetRequestInterface::AssetRequester

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071af10` | `AssetRequester::Destructor` | 🟢 | dtor | Installs AssetRequester vtable and calls member virtual slot +0x18 with this and zero when member exists. |
| `0071b5b0` | `AssetRequesterScalarDeletingDtor` | ✅ | dtor | Calls AssetRequester cleanup body and frees object when deleting flag bit is set. |
| `00989810` | `AssetRequester::Destructor` | ✅ | dtor | Releases the IString member at +0xc when its length is positive, installs the AssetRequester vtable, then calls the nonnull member at +4 through slot +0x18 with this and zero. |
