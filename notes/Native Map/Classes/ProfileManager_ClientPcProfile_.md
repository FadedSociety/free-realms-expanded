# Class: ProfileManager<ClientPcProfile>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a197e0` | `ClientPcProfileManager_Ctor` | 🟢 | ctor | Installs the ProfileManager vftable, zeros its linked-list and cache fields, constructs the ProfileTypeEntry hash list, and clears two trailing fields. |
| `00a19870` | `ClientPcProfileManager_Dtor` | 🟢 | dtor | Restores the manager vftable, removes and frees list-head nodes, clears a field, and destroys its ProfileTypeEntry map. |
| `00a19bf0` | `ClientPcProfileManager_ScalarDeletingDtor` | 🟢 | dtor | Calls the ProfileManager teardown body, conditionally frees this, and returns this when the deleting flag is clear. |
