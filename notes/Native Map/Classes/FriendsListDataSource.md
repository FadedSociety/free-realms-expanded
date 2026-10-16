# Class: FriendsListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb2db0` | `FriendsListDataSource::ColumnName` | ✅ | virtual | Maps column numbers to friend name, coordinates, GUID, online state, profile values, encounter state, status, score and current minigame fields. |
| `00cb44c0` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the row, then maps column numbers to friend identity, profile, online status, location, score and minigame values, formatting each to the requested output type. |
| `00cb5140` | `FriendsListDataSource::Dtor` | ✅ | dtor | Restores data-source vtables, clears the row count, frees the pointer array with alignment 4, destroys the embedded hash list and GUI holder. |
| `00cb5210` | `FriendsListDataSource::GetRowCount` | ✅ | virtual | Returns the 32-bit value stored at this+0x238. |
| `00cb52c0` | `FriendsListDataSource::Ctor` | ✅ | ctor | Initializes the BaseClient.FriendsList holder, stores its source, installs class vtables, initializes the embedded hash list and row array, and clears count/dirty state. |
| `00cb5370` | `FriendsListDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the friends data-source destructor, then frees the object when the low bit of the deleting flag is set. |
