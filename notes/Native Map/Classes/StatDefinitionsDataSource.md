# Class: StatDefinitionsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a73af0` | `StatDefinitionsDataSource::ColumnName` | ✅ | virtual | Virtual slot [1]. Switch on the int argument: 0 returns Stat ID, 1 Icon ID, 2 Name, 3 Description, any other value returns <error>. Pure lookup; no state is read. |
| `00a73c10` | `StatDefinitionsDataSource::GetCell` | 🟢 | virtual | Virtual slot [4] (key, column, out). Gets the object through 00769710 and its +4 sub-object, then the table at +0x1d8 (returns 0 when that is null). Looks up the node with key % 0x47 in the 71-bucket table at +0x14 (key… |
| `00a73cf0` | `StatDefinitionsDataSource::DtorBody` | ✅ | dtor | Stores the StatDefinitionsDataSource vtables at this, +0x30 and +0x34, then calls 0091bf60 (accepted label Dtor_GuiUserDataTableAt34) with ECX = this. Does not free this. |
| `00a73d60` | `StatDefinitionsDataSource::Ctor` | ✅ | ctor | Calls the base constructor 00922890 with the name string BaseClient.StatDefinitions, then installs the StatDefinitionsDataSource vtables at +0, +0x30 and +0x34. Returns this. |
| `00a73dd0` | `StatDefinitionsDataSource::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a73cf0 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
