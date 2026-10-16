# Class: SelectedFixtureDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb1e0` | `SelectedFixtureDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of SelectedFixtureDataSource (one stack argument, ret 4): index 0 returns Fixture Guid, any other index returns <error>. Bytes: cmp dword [esp+4],0; mov eax,<error>; jne; mov eax,Fixture Guid; ret 4. |
| `00cbd700` | `SelectedFixtureDataSource::GetValue` | ✅ | virtual | Reads the selected fixture pointer; returns false if it is absent or the requested column is nonzero, otherwise formats the fixture's 64-bit value into the output. |
| `00cbf090` | `SelectedFixtureDataSource::SelectedFixtureDataSource` | ✅ | ctor | Registers BaseClient.Housing.Instance.SelectedFixture, installs three class vtables, and stores the two supplied arguments in fields 0x254 and 0x258. |
| `00cbf170` | `SelectedFixtureDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the embedded-holder destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
