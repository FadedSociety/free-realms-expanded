# Class: SoeUtil::List<ProxiedVehicleDemolitionDerby,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009eca20` | `— ⚠reject` | ⚪ | virtual | Allocation virtual (vtable slot 2) of the list: calls the TLS initialiser FUN_006fb9c0, then runs the thread memory manager from FUN_006fb860's vtable slot 1 when present, otherwise the global manager DAT_01bdf280's slo… |
| `009eca60` | `SoeUtil::List::FreeNode` | ✅ | virtual | Freeing virtual (vtable slot 3) of the list: 18-byte body that forwards its pointer argument to FUN_00401c00 with alignment 16 and returns. |
| `009ecc20` | `DemolitionDerbyList_AppendNew` | 🟢 | method | Appends a new actor to the ProxiedVehicleDemolitionDerby list (head +4, tail +8, count +0xc): the list's vtable slot 2 allocates the node, 009eb8e0 constructs the actor in it, the node's +0x1bf4 takes the old tail, +0x1… |
| `009ed6e0` | `DemolitionDerbyList_RemoveNode` | 🟢 | method | Unlinks the node param_2 from the list (head +4, tail +8, count +0xc; node next +0x1bf0, prev +0x1bf4), decrements the count, destroys the actor with FUN_00c656b0 and frees the node through the list's vtable slot 3. |
| `009ee010` | `DemolitionDerbyList_ClearNodes` | 🟢 | function | Loops on the head at this+4 calling 009ed6e0 (unlink, destroy actor, free node) until the list is empty; returns nothing. |
| `009ee510` | `SoeUtil::List::dtor → SoeUtil::List<ProxiedVehicleDemolitionDerby,-1>::dtor` | ✅ | dtor | Destructor body: stores the list vtable at this+0 and then removes every node through 009ed6e0 (unlink, destroy actor, free) until the head at +4 is null. |
| `009ee580` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: stores the list vtable, removes every node through 009ed6e0 and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
