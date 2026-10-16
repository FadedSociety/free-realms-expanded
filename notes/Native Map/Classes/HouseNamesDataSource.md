# Class: HouseNamesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb1b0` | `HouseNamesDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of HouseNamesDataSource (stdcall-style, one stack argument, ret 4): index 0 returns Name Id, index 1 returns Name, any other index returns <error>. Bytes decode as sub eax,0 / je, sub eax,1 / je, then mov… |
| `00cbbbe0` | `HouseNamesDataSource::GetViaSub228Dec0 ?` | 🟡 | virtual | Virtual slot 2 of HouseNamesDataSource: bytes 8b 81 28 02 00 00 8b 80 c0 de 00 00 c3 decode to load the pointer at receiver+0x228, load the dword at that pointer plus 0xdec0 and return it. |
| `00cbd530` | `HouseNamesDataSource::GetValue` | ✅ | virtual | Reads the indexed house-name id; column 0 formats that id, while column 1 resolves a wide-string name and builds a player-specific key before returning the resulting value. |
| `00cbef50` | `HouseNamesDataSource::HouseNamesDataSource` | ✅ | ctor | Registers the BaseClient.Housing.AvailableNames data source, installs its vtables, and stores two constructor arguments in fields 0x254 and 0x258. |
| `00cbf030` | `HouseNamesDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the class destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
