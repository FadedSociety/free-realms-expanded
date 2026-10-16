# Class: HouseCustomNameDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb480` | `HouseCustomNameDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of HouseCustomNameDataSource (thiscall, index argument): 0 returns String, 1 returns Id, 2 returns Part, 3 returns Selected; any other index returns <error>. |
| `00cbc730` | `HouseCustomNameDataSource::GetValueAfterRebuildType123 ?` | 🟡 | virtual | Virtual slot 2 of HouseCustomNameDataSource (vtable 018378c0; thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbc400 with ECX = receiver-0x30; then returns the dword at receiver+0x230. Bytes 56 … |
| `00cbe200` | `HouseCustomNameDataSource::GetValue` | ✅ | virtual | Resolves a row into the house custom-name table, returns fields by column, and on the custom lookup column forms a player/house key and queries the Houses table; missing rows return the literal error values. |
| `00cbedd0` | `HouseCustomNameDataSource::Destroy` | ✅ | dtor | Restores HouseCustomNameDataSource vtables at the primary and two embedded bases, then destroys the embedded GUI user-data holder. |
| `00cbf410` | `HouseCustomNameDataSource::HouseCustomNameDataSource` | ✅ | ctor | Registers BaseClient.HouseCustomNameList, installs the class's three vtables, stores two constructor arguments, and clears the byte flag at offset 0x25c. |
| `00cbf4a0` | `HouseCustomNameDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the custom-name data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
