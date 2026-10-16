# Class: ClientItemFilterManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb2350` | `— ⚠reject` | ⚪ | ctor | Constructor: installs BaseItemFilterManager<ClientItemDefinition>::vftable and then ClientItemFilterManager::vftable at +0, stores the argument at +4, runs the base filter-manager setup 0107f420(param_3,param_4), zeroes… |
| `00bb2420` | `ClientItemFilterManager::~ClientItemFilterManager` | ✅ | dtor | Destructor body: rewrites ClientItemFilterManager::vftable into this+0, releases the refcounted data source at +0xb1c (asks it for its registry id through vtable slot 0x28 and calls the registry release 0081ad80 when th… |
| `00bb24c0` | `ClientItemFilterManager::FilterMatch ⚠downgrade` | 🟢 | virtual | Vtable slot 0: returns false unless the client interface [[this+4]+0x3e0] and the filter entry for param_3 (bucket this+0xc4+(param_3 & 0x1f)*4, key +0x130, chain +0x134) exist; then resolves the option entry at entry+0… |
| `00bb2620` | `ClientItemFilterManager::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 1: runs the destructor body 00bb2420 and, when bit 0 of the flag argument is set, releases this through the CRT free thunk before returning it. |
| `0107f4e0` | `— ⚠reject` | ⚪ | method | Clears the list at this+0xb0 through 0107dce0, then loads three text tables with the string formatter and the row reader: ItemFilters.txt (rows split on caret, inserted through 0107f060 into the bucket table at this+0x2… |
