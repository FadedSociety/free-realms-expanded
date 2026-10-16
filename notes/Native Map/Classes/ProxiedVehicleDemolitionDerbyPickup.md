# Class: ProxiedVehicleDemolitionDerbyPickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009eb600` | `ProxiedVehicleDemolitionDerbyPickup_DtorWrapper` | 🟢 | function | SEH-framed wrapper preserves incoming ECX as the receiver for destructor body 00c65a10, restores the prior exception chain, and returns; the stack-cookie push is frame bookkeeping, not a destructor argument. |
| `009eb960` | `ProxiedVehicleDemolitionDerbyPickup_ConstructWrapper` | 🟢 | function | SEH-framed placement-construction wrapper: calls FUN_00c65aa0(*param_2, *param_3) and returns param_1. |
| `009ecdd0` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: calls FUN_00c65a10 (the ProxiedVehicleDemolitionDerbyPickup destructor) and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
