# Class: InspectDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbfd20` | `InspectDataSource::GetColumnName` | ✅ | virtual | Maps integer selectors 0 through 0x1d to inspection column-name strings such as player identity, jobs, stats, profile and social fields, with an error fallback. |
| `00cc0140` | `InspectDataSource::GetRowCount` | ✅ | virtual | Reads the count field at inspect-object offset 0x174 and returns at least one, even when the stored count is below two. |
| `00cc0760` | `InspectDataSource::GetValue` | ✅ | virtual | Maps selectors 0 through 0x1d to the inspect target's identity, numeric stats, job values, strings and profile fields; computes age from the birthday string and returns a boolean for populated extra data. |
| `00cc0db0` | `InspectDataSource::InspectDataSource` | ✅ | ctor | Registers BaseClient.Inspect, installs the class's primary and secondary vtables, and stores the supplied inspect-object pointer. |
| `00cc0e90` | `InspectDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the embedded GUI user-data holder and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
