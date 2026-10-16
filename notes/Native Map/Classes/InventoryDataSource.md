# Class: InventoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6db10` | `InventoryDataSource::ColumnLabel_Kinds1dTo1c` | ✅ | virtual | Virtual slot 1 override: returns IsRental for argument 0x1d, RentalSecondsRemaining for 0x1e, Price for 0x1b and NoSale for 0x1c. Other arguments are forwarded to 00cb09d0 with the same argument and its result is return… |
| `00a6db60` | `InventoryDataSource::ForwardToSlot14 ?` | 🟡 | virtual | Virtual slot 17 (+0x44) thunk, 7 bytes: loads the vtable pointer from ECX, reads the entry at +0x38 (virtual slot 14) and jumps to it with the caller's arguments unchanged (indirect jmp, not a call). |
| `00a6deb0` | `InventoryDataSource::InventoryDataSource` | ✅ | ctor | Constructor body: calls the base constructor 00a6d870, installs InventoryDataSource vtables at offsets 0x0, 0x30, 0x34 and 0x254, embeds an SoeUtil::IString<char> at +0x2ec with an empty buffer (DAT_01be6e88), clears th… |
| `00a6df60` | `InventoryDataSource::SecondaryAdjustorToDtor ?` | 🟡 | virtual | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6e0f0 (InventoryDataSource scalar deleting destructor). |
| `00a6df70` | `InventoryDataSource::ReleaseTextMember2ec → InventoryDataSource::~InventoryData…` | ✅ | function | Releases the IString<char> member at +0x2ec: stores the SoeUtil::IString<char> vtable there and, when the length at +0x2f8 is positive, drops one reference on the buffer at +0x2f0 (lock-prefixed decrement of the count s… |
| `00a6e0f0` | `InventoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls 00a6df70 (text member release at +0x2ec), then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6e6b0` | `InventoryDataSource::GetRowValueKind1dOr1e` | 🟢 | virtual | Virtual slot 4 override: returns 0 when the row index is at or beyond the count at +0x24c. Kind 0x1d looks the row up through 008becc0 and writes DAT_0180c498 or DAT_017f7d9c (chosen by 0098f7b0) through 00703620, retur… |
| `00a6e910` | `— ⚠reject` | ⚪ | virtual | Virtual slot 14 (+0x38) of InventoryDataSource and StorageDataSource. When the byte at +0x2e8 is clear it calls 00cb1340 (SEH-prolog-style helper) and returns. Otherwise it loads the stored text at +0x2f0 through 0083d2… |
