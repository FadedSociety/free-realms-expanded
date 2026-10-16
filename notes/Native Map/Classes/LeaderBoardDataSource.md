# Class: LeaderBoardDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cdeef0` | `LeaderBoardDataSource::DestructorBody` | ✅ | dtor | Restores three LeaderBoardDataSource vtables, conditionally releases the stored reference at +0x258, then destroys the embedded GUI table. |
| `00cdf050` | `LeaderBoardDataSource::Constructor` | ✅ | ctor | Initializes the BaseClient.LeaderBoard path, installs three vtables, stores its context, allocates a 600-byte entries object, and installs that object through the reference helper. |
| `00cdf110` | `LeaderBoardDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |
