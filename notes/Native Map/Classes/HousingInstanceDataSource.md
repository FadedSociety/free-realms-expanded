# Class: HousingInstanceDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb210` | `HousingInstanceDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of the housing instance data sources (shared implementation; thiscall, index argument): switches on indices 0 to 0x11 and returns the header text: 0 Instance Guid, 1 Owner Guid, 2 Owner Name, 3 House Name… |
| `00cbbbf0` | `HousingInstanceDataSource::GetField234 ?` | 🟡 | virtual | Virtual slot 2 of the housing instance data sources (shared implementation): bytes 8b 81 34 02 00 00 c3 decode to mov eax,[ecx+0x234]; ret, returning the dword at receiver+0x234. |
| `00cbd760` | `HousingInstanceDataSource::GetValue` | ✅ | virtual | Selects a housing-instance row and returns its identifier, strings, booleans, numeric properties, player name, or formatted custom values according to the requested column index. |
| `00cbf1d0` | `HousingInstanceDataSource::HousingInstanceDataSource` | ✅ | ctor | Registers the supplied data-source key, installs HousingInstanceDataSource vtables, stores two arguments, and initializes the PlayerHousingInstanceInfo array vtable and size/capacity fields to zero. |
| `00cbf280` | `HousingInstanceDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00cbed40 to release array and holder state, then tail-jumps to the allocator thunk when the low flag bit requests object deletion; otherwise returns this. |
