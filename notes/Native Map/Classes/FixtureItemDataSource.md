# Class: FixtureItemDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb160` | `FixtureItemDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of FixtureItemDataSource (thiscall, index argument): for index below 0x29 returns the result of 00c92c10 on the index; index 0x29 returns FixtureGuid, 0x2a returns Location; any other index returns <error… |
| `00cbbac0` | `FixtureItemDataSource::Ctor` | ✅ | ctor | Thiscall with three stack arguments: calls 00c96a30 on the receiver with param_4 and param_2, stores FixtureItemDataSource::vftable at p+0, p+0x30 and p+0x34, stores param_2 at p+0x52f4 and zeroes p+0x52f8, p+0x52fc and… |
| `00cbbba0` | `FixtureItemDataSource::GetField52d0 ?` | 🟡 | virtual | Virtual slot 2 of FixtureItemDataSource: bytes 8b 81 d0 52 00 00 c3 decode to mov eax,[ecx+0x52d0]; ret, returning the dword at receiver+0x52d0. |
| `00cbc260` | `FixtureItemDataSource::DeletingDtor` | ✅ | dtor | Virtual slot 0 of FixtureItemDataSource (thiscall, one byte flag): calls 00c968b0 on the receiver (dtor body), then when flag bit 0 is set calls _free (thunk 00d82463) on it; returns the receiver. |
| `00cbd390` | `FixtureItemDataSource::GetValue` | ✅ | virtual | Returns values for an indexed fixture row; handles column 0x29 by formatting an integer and column 0x2a by building a player-location string from actor coordinates, with other columns delegated to lookup and formatting … |
