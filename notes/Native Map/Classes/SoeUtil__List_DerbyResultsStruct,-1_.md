# Class: SoeUtil::List<DerbyResultsStruct,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ecb70` | `SoeUtil::List::AllocNode` | ✅ | virtual | Allocation virtual (vtable slot 2) of the DerbyResultsStruct list: same manager probe, falling back to FUN_006fb950(0x20, 8). |
| `009ecbb0` | `SoeUtil::List::FreeNode` | ✅ | virtual | Freeing virtual (vtable slot 3) of the DerbyResultsStruct list: 18-byte body forwarding its argument to FUN_00401c00 with alignment 8. |
| `009ed810` | `DerbyResultsList_AppendNew` | 🟢 | method | Allocates a fresh node through the list's vtable slot 2 and appends it at the tail (head +4, tail +8, count +0xc): the node's next at +0x18 is zeroed, its prev at +0x1c takes the old tail, the old tail's +0x18 is set, t… |
| `009ed890` | `DerbyResultsList_Sort` | 🟢 | method | In-place bottom-up merge sort of a doubly linked list (head +4, tail +8, count +0xc; node next +0x18, prev +0x1c): a 32-slot stack array (memset 0x80) drives 1,2,4,... element merges, the comparison comes from the callb… |
| `009ee050` | `DerbyResultsList_Clear` | 🟢 | method | Empties the results list (head +4, tail +8, count +0xc; node next +0x18, prev +0x1c) by taking the head, unlinking it with both neighbour links repaired, decrementing the count and freeing the node through the list's vt… |
| `009ee570` | `SoeUtil::List::dtor → SoeUtil::List<DerbyResultsStruct,-1>::dtor` | ✅ | dtor | Eleven-byte destructor body: stores the list vtable at *param_1 (01810b38) and tail-jumps into the clear routine 009ee050 (bytes: c7 01 38 0b 81 01; e9 d5 fa ff ff). |
| `009ee600` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: stores the results-list vtable, calls the clear routine 009ee050 and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
