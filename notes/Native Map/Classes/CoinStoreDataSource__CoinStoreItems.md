# Class: CoinStoreDataSource::CoinStoreItems

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf0700` | `CoinStoreDataSource::CoinStoreItems::Ctor` | ✅ | ctor | Calls the base data-source initializer with its two arguments and installs the CoinStoreItems vtable at three object offsets. |
| `00cf0790` | `CoinStoreDataSource::CoinStoreItems::Dtor` | ✅ | dtor | Restores the CoinStoreItems vtables at three object offsets and invokes the shared base-data-source cleanup routine. |
| `00cf07f0` | `CoinStoreDataSource::CoinStoreItems::FieldName` | ✅ | virtual | Returns field labels for coin purchase price, sell price, item GUID list, no-sale, preview eligibility, tint group, and member purchase price; delegates unknown indices. |
| `00cf1130` | `CoinStoreDataSource::CoinStoreItems::ScalarDeletingDtor` | ✅ | dtor | Calls CoinStoreItems cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
| `00cf1b60` | `CoinStoreDataSource::CoinStoreItems::GetFieldValue` | ✅ | virtual | Switches on the requested field id, returns or writes values for cases 6, 0xd, 0x29, 0x2a, 0x2c, 0x2d, 0x2e and 0x2f, and delegates all other cases to 00c956c0. |
