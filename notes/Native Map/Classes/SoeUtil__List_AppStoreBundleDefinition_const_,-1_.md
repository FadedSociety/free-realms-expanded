# Class: SoeUtil::List<AppStoreBundleDefinition_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099bd00` | `ListAppStoreBundleDefPtrSlot2Alloc12` | 🟢 | virtual | Virtual slot [2] of SoeUtil::List<AppStoreBundleDefinition_const*,-1>, no stack args, 57 bytes. Calls 006fb9c0, then 006fb860: if nonzero calls its slot +4 and returns; else if DAT_01bdf280 is nonzero calls its slot +4 … |
| `0099bd40` | `ListAppStoreBundleDefPtrSlot3FreeArg` | 🟢 | virtual | Virtual slot [3] of SoeUtil::List<AppStoreBundleDefinition_const*,-1>, one stack arg, 18 bytes. Calls 00401c00(arg,4) and returns void. |
| `0099f8a0` | `— ⚠reject` | ⚪ | method | Repeatedly removes the head node, repairs its predecessor/successor links and the tail when needed, decrements count at +0xC, and invokes cleanup through virtual slot +0xC. |
| `009a0300` | `ListAppStoreBundleDtor` | ✅ | dtor | Installs the typed list vtable and delegates to FUN_0099F8A0 to remove each linked-list node. |
| `009a0440` | `ListAppStoreBundleScalarDtor` | ✅ | virtual | Clears the typed bundle list, conditionally frees the list object, and returns its pointer. |
