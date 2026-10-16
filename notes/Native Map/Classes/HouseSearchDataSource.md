# Class: HouseSearchDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb520` | `HouseSearchDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of HouseSearchDataSource (thiscall, index argument): 0 returns HouseGuid, 1 returns OwnerName, 2 returns HouseName, 3 returns OwnerGuid, 4 returns Snapshot, 5 returns Rating, 6 returns NumVotes; any other… |
| `00cbc4a0` | `HouseSearchDataSource::GetValueAfterRebuildSlot4 ?` | 🟡 | virtual | Virtual slot 2 of HouseSearchDataSource (thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbbcb0 with ECX = receiver-0x30; then returns the dword at receiver+0x230. Bytes 56 8b f1 80 be 2c 02 00 … |
| `00cbe570` | `HouseSearchDataSource::GetValue` | ✅ | virtual | Looks up a search-result row and maps columns 0 through 6 to its integer id, string fields and numeric fields, formatting the 64-bit key for column zero. |
| `00cbeed0` | `HouseSearchDataSource::Destroy` | ✅ | dtor | Restores HouseSearchDataSource vtables at the primary and two embedded bases, then destroys the embedded GUI user-data holder. |
| `00cbf570` | `HouseSearchDataSource::HouseSearchDataSource` | ✅ | ctor | Registers BaseClient.HouseSearchList, installs the class's three vtables, stores two constructor arguments, and clears the byte flag at offset 0x25c. |
| `00cbf600` | `HouseSearchDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the search data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
