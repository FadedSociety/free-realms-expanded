# Class: LeaderBoardEntryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cdee80` | `LeaderBoardEntryDataSource::DestructorBody` | ✅ | dtor | Restores three LeaderBoardEntryDataSource vtables and destroys the embedded GUI user-data table at its adjusted subobject. |
| `00cdefb0` | `LeaderBoardEntryDataSource::Constructor` | ✅ | ctor | Initializes the leaderboard entries path, installs three vtable pointers, and stores the supplied context at +0x254. |
| `00cdf030` | `LeaderBoardEntryDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |
