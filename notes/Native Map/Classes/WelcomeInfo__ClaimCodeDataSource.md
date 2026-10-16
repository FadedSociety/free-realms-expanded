# Class: WelcomeInfo::ClaimCodeDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ace4f0` | `GetClaimCodeColumnName` | ✅ | virtual | Maps indices 0 through 4 to Code, Title, Description, ImageSetId, and TintId; returns <error> for other indices. |
| `00ace930` | `GetField230` | ✅ | virtual | Returns the dword stored at this+0x230. |
| `00acfcf0` | `— ⚠reject` | ⚪ | virtual | Selects one 0x2c-byte row and handles columns 0 through 4 by clearing a string, resolving two string ids, formatting an integer, or looking up a final id. |
| `00ad0d20` | `WelcomeInfo::ClaimCodeDataSource::Ctor` | ✅ | ctor | Installs ClaimCodeDataSource and entry-array vtables, stores the owner pointer, and zeros the array count, capacity, and pointer. |
| `00ad0db0` | `WelcomeInfo::ClaimCodeDataSource::Dtor` | ✅ | dtor | Restores the entry-array vtable, destroys mixed-string rows, frees array storage, clears the pointer, and calls the embedded base destructor. |
| `00ad0e40` | `WelcomeInfo::ClaimCodeDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the data-source destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
