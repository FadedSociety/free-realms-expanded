# Class: SoeUtil::List<ProxiedVehicleDemolitionDerbyPickup,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009eca90` | `SoeUtil::List::AllocNode` | ✅ | virtual | Allocation virtual (vtable slot 2) of the pickup list: same TLS/thread/global manager probe as 009eca20 but falls back to FUN_006fb950(0x2c, 4). |
| `009ecad0` | `SoeUtil::List::FreeNode → SoeUtil::List<ProxiedVehicleDemolitionDerbyPickup,-1>…` | ✅ | virtual | Freeing virtual (vtable slot 3) of the pickup list: 18-byte body forwarding its argument to FUN_00401c00 with alignment 4. |
| `009eccd0` | `DemolitionDerbyPickupList_AppendNew` | 🟢 | method | Appends a new pickup to the ProxiedVehicleDemolitionDerbyPickup list (head +4, tail +8, count +0xc): the list's vtable slot 2 allocates the node, 009eb960 constructs the pickup, the node's +0x28 takes the old tail, +0x2… |
| `009ed780` | `DemolitionDerbyPickupList_RemoveNode` | 🟢 | method | Unlinks the node param_2 from the pickup list (head +4, tail +8, count +0xc; node next +0x24, prev +0x28), decrements the count, destroys the pickup with FUN_00c65a10 and frees the node through the list's vtable slot 3. |
| `009ee030` | `DemolitionDerbyPickupList_ClearNodes` | 🟢 | function | Loops on the head at this+4 calling 009ed780 (unlink, destroy pickup, free node) until the pickup list is empty; returns nothing. |
| `009ee540` | `SoeUtil::List::dtor → SoeUtil::List<ProxiedVehicleDemolitionDerbyPickup,-1>::dt…` | ✅ | dtor | Destructor body: stores the pickup-list vtable at this+0 and then removes every node through 009ed780 until the head at +4 is null. |
| `009ee5c0` | `— → SoeUtil::List<ProxiedVehicleDemolitionDerbyPickup,-1>_scalarDeletingDtor ⚠r…` | ⚪ | dtor | Scalar deleting destructor: stores the pickup-list vtable, removes every node through 009ed780 and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
