# Class: SoeUtil::HashList<ClientLuaMetrics::Entry,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01044db0` | `HashList_ClientLuaMetricsEntry::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores SoeUtil::HashListMap<int,ClientLuaMetrics::Entry,20,-1>::vftable at *this, zeroes this+0x10, +8, +0xc and +4, memsets 0x50 bytes at this+0x14, then overwrites *this with SoeUtil::HashList<Cl… |
| `01044e30` | `HashList_ClientLuaMetricsEntry::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Calls 0091d980 (the ClientLuaMetrics hash container destructor body), then if bit 0 of the flag is set calls _free on the object. Returns the object pointer. |
