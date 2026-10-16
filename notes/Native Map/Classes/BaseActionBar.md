# Class: BaseActionBar

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c445a0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 setter: finds the slot node for id param_2 (bucket id & 7 at this+0x1c, key at +0x60); when no node exists and the create flag is set it allocates one through FUN_008f65a0(&id), then copy-assigns the inco… |
| `00c453d0` | `BaseActionBar_Destruct` | ✅ | dtor | Base destructor: reinstalls BaseActionBar::vftable and destroys the embedded ActionBarSlot map at this+8 by calling the map destructor 00c44770 (the destructor is also reached from EH unwind funclets of the derived clas… |
| `00c45490` | `BaseActionBar_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the base destructor 00c453d0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c454b0` | `BaseActionBar_Construct` | ✅ | ctor | Constructor: installs BaseActionBar::vftable, stores the owner argument at +4 and constructs the embedded slot map at +8 through 00c45310. |
