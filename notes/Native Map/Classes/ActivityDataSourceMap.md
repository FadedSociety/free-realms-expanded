# Class: ActivityDataSourceMap

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cea3a0` | `ActivityDataSourceMap::FindOrCreate ?` | 🟡 | method | Looks up a datasource by hash and key; returns it if matched, returns null when creation is disabled, otherwise allocates a datasource, inserts it, and acquires a reference. |
| `00cea4c0` | `ScheduledActivityDataSourceMap::FindOrCreate ?` | 🟡 | method | Looks up scheduled datasource by hash and key; returns it if found, otherwise optionally allocates, inserts, and references a new scheduled datasource. |
