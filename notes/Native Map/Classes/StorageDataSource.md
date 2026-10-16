# Class: StorageDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6db90` | `StorageDataSource::RowPassesFilter` | 🟢 | virtual | Virtual slot 15 override for storage lists: resolves the row id through 00b0d810 (must be nonzero), requires 008a6920 on this+0x2e0 with the id at +0x70 to return nonzero and the count at param_2+0x1c to be positive. Wh… |
| `00a6e000` | `StorageDataSource::StorageDataSource` | ✅ | ctor | Constructor: calls the InventoryDataSource constructor 00a6deb0 with the forwarded arguments, then installs StorageDataSource vtables at offsets 0x0, 0x30, 0x34 and 0x254. |
| `00a6e090` | `StorageDataSource::SecondaryAdjustorToDtor ?` | 🟡 | virtual | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6e110 (StorageDataSource scalar deleting destructor). |
| `00a6e110` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: calls 00a6df70 (text member release at +0x2ec) with the frame cookie, then frees this through _free when bit 0 of the flag argument is set; returns this. |
