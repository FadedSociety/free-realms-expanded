# Class: FixtureDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb420` | `FixtureDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of FixtureDataSource (thiscall, index argument): index 0 returns Fixture Guid, 1 returns Texture Alias, 2 returns Tint Alias; any other index returns <error>. |
| `00cbb460` | `FixtureDataSource::GetSub238Field7c ?` | 🟡 | virtual | Virtual slot 2 of FixtureDataSource (thiscall, no arguments): when the pointer at receiver+0x238 is nonzero returns the dword at that pointer plus 0x7c, otherwise returns 0. Bytes 8b 81 38 02 00 00 85 c0 74 04 8b 40 7c … |
| `00cbe150` | `FixtureDataSource::GetValue` | ✅ | virtual | Checks that fixture data exists, refreshes a cached field when its flag is set, bounds-checks the row index, then formats the selected fixture's 64-bit identifier for column zero. |
| `00cbf2a0` | `FixtureDataSource::FixtureDataSource ⚠downgrade` | 🟢 | ctor | Registers BaseClient.Housing.Instance.Fixtures, installs the class and guid-array vtables, initializes array counts and pointers, stores one argument, and clears two flags. |
| `00cbf3f0` | `FixtureDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the fixture-data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
