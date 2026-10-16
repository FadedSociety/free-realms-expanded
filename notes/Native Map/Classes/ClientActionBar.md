# Class: ClientActionBar

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c44600` | `— ⚠reject` | ⚪ | virtual | Client override of the slot setter: keeps a copy of the previous slot payload (FUN_0101cd80), applies the base set (00c445a0), then compares old and new fields and pushes targeted UI notifications - code 6 when the +0x3… |
| `00c45510` | `ClientActionBar_Construct` | ✅ | ctor | Constructor: runs the BaseActionBar constructor with the owner argument, installs ClientActionBar::vftable, zeroes the extra fields at +0x398..+0x3b4, stores the game/minigame pointer at +0x3a8, sets the selected slot a… |
| `00c455f0` | `ClientActionBar_Destruct` | ✅ | dtor | Destructor: reinstalls ClientActionBar::vftable, releases the data source pointer at +0x3b0 (virtual slot 10 result handed to FUN_0081ad80 while the engine global is alive) and nulls it, then runs the BaseActionBar dest… |
| `00c45680` | `ClientActionBar_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the ClientActionBar destructor 00c455f0 and frees the object with _free when the low bit of the flag argument is set. |
