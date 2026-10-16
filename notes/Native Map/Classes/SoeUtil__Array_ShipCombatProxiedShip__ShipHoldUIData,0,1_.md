# Class: SoeUtil::Array<ShipCombatProxiedShip::ShipHoldUIData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c9b3a0` | `ShipHoldUIArrayReallocate` | ✅ | virtual | Reuses existing storage under capacity conditions; otherwise computes a capacity, updates the output count, and allocates capacity times 0x1c bytes aligned to four. |
| `00c9b440` | `ShipHoldUIArrayFreeStorage` | ✅ | virtual | Frees the supplied storage pointer with four-byte alignment through the memory manager. |
| `00c9b460` | `— ⚠reject` | ⚪ | virtual | Calls the element destructor when requested; otherwise swaps the array storage pointer and uses XOR swaps to exchange two size fields. |
