# Class: SoccerPlayerStatsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd7620` | `SoccerPlayerStatsDataSource::~SoccerPlayerStatsDataSource` | ✅ | dtor | Rewrites the player statistics data-source vtable and destroys its embedded GuiUserDataTable base member. |
| `00cd77a0` | `SoccerPlayerStatsDataSource::SoccerPlayerStatsDataSource` | ✅ | ctor | Constructs the embedded GUI user-data table and stores the SoccerPlayerStats data-source identifier. |
| `00cd7820` | `SoccerPlayerStatsDataSource::DeletingDestructor ?` | 🟡 | virtual | Calls the player statistics data-source destructor, checks flag bit zero, conditionally frees the object, and returns its address. |
