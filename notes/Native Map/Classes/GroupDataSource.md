# Class: GroupDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7c960` | `GroupDataSource::GetColumnName` | ✅ | virtual | Column-name table of the group data source: returns the literal MinigameType when the index is 0 and <error> for every other index; it never touches this. |
| `00a7c980` | `GroupDataSource::GetRowCount` | 🟢 | virtual | Reads the group pointer at this+0x224 (primary object +0x254), then returns 1 when that object's field +0x6ec is non-null and 0 otherwise (the compiled body is cmp/setne on the single field), i.e. the list has one group… |
| `00a7cd10` | `GroupDataSource::GetColumnValue ⚠downgrade` | 🟢 | virtual | Row-value getter of the group data source. When the group pointer (this+0x224)->+0x6ec is null it returns 0; otherwise it returns 0 for any column other than 0 and for column 0 writes FUN_009ad290(0) into the output cel… |
| `00a7db70` | `GroupDataSource::~GroupDataSource` | ✅ | dtor | Destructor body: rewrites the three GroupDataSource vtable pointers (at +0, +0x30 and +0x34) and runs the shared base destructor FUN_0091bf60. |
| `00a7dc80` | `GroupDataSource::GroupDataSource` | ✅ | ctor | Constructor: registers the name BaseClient.GroupData through FUN_00922890, installs the three GroupDataSource vtables (+0, +0x30, +0x34) and stores the owner argument at +0x254. |
| `00a7dd00` | `GroupDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the destructor body 00a7db70 and frees the object with _free when bit 0 of param_2 is set. |
