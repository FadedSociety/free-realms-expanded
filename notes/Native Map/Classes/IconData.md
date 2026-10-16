# Class: IconData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00720bf0` | `IconData::IconData` | ✅ | ctor | Installs Loggable::vftable and then IconData::vftable into param_1 and stores param_2 at +4 and param_3 at +8. |
| `00720cd0` | `IconData::Print` | ✅ | method | Logs the header --- IconData --- and then ImageSetId=%d with the dword at +4 and TintId=%d with the dword at +8 through 00720c60. |
| `008b2bd0` | `IconData::CopyConstructor` | ✅ | ctor | Installs Loggable::vftable, then IconData::vftable over it, then copies the two dwords at param_2+4 and param_2+8 into this+4 and this+8. Returns nothing. |
