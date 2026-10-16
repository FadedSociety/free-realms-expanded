# Class: SoeUtil::List<ProxiedVehicleProximityArrow,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ba07e0` | `— ⚠reject` | ⚪ | dtor | Fastcall destructor body (this in ECX): stores the list vtable 01824614 at +0, then calls the node remover 00b9ec50 on the head node (+4) for as long as the head is non-null. Does not free the object. |
| `00ba0810` | `SoeUtil::List_ProxiedVehicleProximityArrow::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the list vtable 01824614 at +0, drains the head nodes through 00b9ec50 until +4 is null, then calls _free(this) only when bit 0 of the byte argument is set. Returns this. |
