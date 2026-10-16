# Class: SoeUtil::List<GameAccountManagement::AccountInfoCharacterDetails,10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009569f0` | `AccountInfoCharacterDetailsFixedListCtor` | 🟢 | ctor | Installs the dynamic and ten-capacity AccountInfoCharacterDetails list vtables, clears its link/count fields, then calls 00952d10 with embedded storage at this+0x10 in ECX and no stack arguments. |
| `00957420` | `AccountInfoCharacterDetailsFixedList::Pop ?` | 🟡 | virtual | Adds 0x10 to this pointer and jumps to 00957340; byte sequence confirms an adjustor thunk. |
| `00957430` | `AccountInfoCharacterDetailsFixedList::Push ?` | 🟡 | virtual | Adds 0x10 to this pointer and jumps to 009573b0; byte sequence confirms an adjustor thunk. |
| `00957440` | `AccountInfoCharacterDetailsFixedListDtor` | 🟢 | dtor | Destroys linked nodes, clears fields, frees external backing storage, then restores the dynamic-list base vtable. |
| `00957510` | `AccountInfoCharacterDetailsFixedList::ScalarDeletingDtor` | 🟢 | dtor | Calls the fixed-list destructor and frees the object only when deletion flag bit 0 is set. |
