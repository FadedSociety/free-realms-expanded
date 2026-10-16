# Class: SoccerPlayerActionsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd7690` | `SoccerPlayerActionsDataSource::~SoccerPlayerActionsDataSource` | ✅ | dtor | Rewrites the player actions data-source vtable and destroys its embedded GuiUserDataTable base member. |
| `00cd7840` | `— ⚠reject` | ⚪ | ctor | Constructs the embedded GUI user-data table, stores the SoccerPlayerActions identifier, and clears the eight action cells at offset 0x254. |
| `00cd78e0` | `SoccerPlayerActionsDataSource::DeletingDestructor ?` | 🟡 | virtual | Calls the player actions data-source destructor, checks flag bit zero, conditionally frees the object, and returns its address. |
