# Class: SoeUtil::HashList<ActionBarSlot,8,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45310` | `HashList8x8_Construct` | ✅ | ctor | Constructor of the derived list: first runs the pooled map constructor 00c45230 (which installs the map vtable and inits the pool) and then installs the HashList<ActionBarSlot,8,8> vftable at +0. |
| `00c45370` | `HashList8x8_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor for the derived list: runs the map destructor 00c44770 through an SEH frame and frees the object with _free when the low bit of the flag argument is set. |
