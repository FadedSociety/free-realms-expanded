# Class: SoccerTeamStatsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd75b0` | `SoccerTeamStatsDataSource::~SoccerTeamStatsDataSource` | ✅ | dtor | Rewrites the team statistics data-source vtable and destroys its embedded GuiUserDataTable base member. |
| `00cd7700` | `SoccerTeamStatsDataSource::SoccerTeamStatsDataSource` | ✅ | ctor | Constructs the embedded GUI user-data table and stores the SoccerTeamStats data-source identifier. |
| `00cd7780` | `SoccerTeamStatsDataSource::DeletingDestructor ?` | 🟡 | virtual | Calls the team statistics data-source destructor, checks flag bit zero, conditionally frees the object, and returns its address. |
