# Class: ProxiedVehicleDemolitionDerby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009eb5b0` | `ProxiedVehicleDemolitionDerby_DtorWrapper` | 🟢 | function | SEH-framed wrapper preserves incoming ECX as the receiver for destructor body 00c656b0, restores the prior exception chain, and returns; the stack-cookie push is frame bookkeeping, not a destructor argument. |
| `009eb8e0` | `— ⚠reject` | ⚪ | function | SEH-framed placement-construction wrapper: calls FUN_00c657a0(*param_2, *param_3), i.e. the actor constructor receives the two dwords its pointer arguments point to, and returns param_1. |
| `009ecd70` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: calls FUN_00c656b0 (the ProxiedVehicleDemolitionDerby destructor) and frees this through _free when bit 0 of param_2 is set; returns this otherwise (87 bytes with SEH frame). |
