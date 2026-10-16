# Class: ServerListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c29c80` | `ServerListDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): returns Name for column 0, Online for column 1 (a pointer three bytes into the literal 0x0182f830), Locked for column 2 and <error> otherwise. |
| `00c29cc0` | `ServerListDataSource::Refresh` | 🟢 | virtual | Virtual override (framework vtable slot [4], whose default implementation is 010caf20): reads the session state field at *(*(this+0x25c)+0x53d9c) into this+0x258 and sets the data-source state at this+0x254 to 0, or to … |
| `00c29d00` | `ServerListDataSource::GetStatusText` | ✅ | method | Writes the current status dword (this+0x258) through the out pointer and switches on it: state 0 returns Invalid session token, state 2 returns No servers available, state -1 (0xffffffff) returns Unable to get list of s… |
| `00c29e00` | `ServerListDataSource::GetRowCount` | 🟢 | virtual | Virtual method (vtable slot [2]): returns the dword at offset +0x22044 of the object referenced by this+0x22c - the row count of the server list. |
| `00c29e10` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [4]) with signature (row, column, outString): walks the server row list from this+0x22c (root at +0x2203c, next node at node+0x2c) to the requested row; column 0 copies the wide server name a… |
| `00c29fb0` | `— ⚠reject` | ⚪ | dtor | Destructor body: resets the three vtable pointers (this+0, +0x30, +0x34) to ServerListDataSource::vftable and calls the shared base teardown 0091bf60 with the stack cookie, then returns. |
| `00c2a020` | `ServerListDataSource::ServerListDataSource` | ✅ | ctor | Constructor: registers the data-source name 'BaseClient.ServerList' through 00922890, installs the three vtable pointers (this+0, +0x30, +0x34), zeroes the state fields at +0x254 and +0x258 and stores the argument at +0… |
| `00c2a0a0` | `ServerListDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c29fb0 and frees the object with _free when the low bit of the flag argument is set. |
