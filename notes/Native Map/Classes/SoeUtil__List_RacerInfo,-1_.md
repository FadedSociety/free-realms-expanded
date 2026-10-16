# Class: SoeUtil::List<RacerInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ba0fe0` | `SoeUtil::List_RacerInfo::DrainNodes` | 🟢 | method | Fastcall loop (this in ECX): while the head pointer at +4 is non-null, removes that head node with 00b9fa20 (SoeUtil::List_RacerInfo::Remove). Does not touch the vtable and does not free the list object. |
| `00ba2830` | `SoeUtil::List_RacerInfo::~List_RacerInfo` | ✅ | dtor | Fastcall destructor body (this in ECX): stores the list vtable 018245b0 at +0, then calls the racer-info unlinker 00b9fa20 on the head node (+4) for as long as the head is non-null. Does not free the object. |
| `00ba2a20` | `SoeUtil::List_RacerInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, byte flag argument): stores the racer-info list vtable 018245b0 at +0, drains the head nodes through 00b9fa20 until +4 is null, then calls _free(this) only when bit 0 of the flag is… |
