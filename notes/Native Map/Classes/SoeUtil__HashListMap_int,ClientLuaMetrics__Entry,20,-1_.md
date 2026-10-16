# Class: SoeUtil::HashListMap<int,ClientLuaMetrics::Entry,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009194e0` | `SoeUtil::HashListMapClientLuaMetricsEntry20DtorBody` | ✅ | dtor | Destructor body: stores the HashListMap<int,ClientLuaMetrics::Entry,20,-1> vtable and walks the node chain at +8 calling FUN_008fb730 on each node until the chain is empty. |
| `00919510` | `SoeUtil::HashListMapClientLuaMetricsEntry20AllocNode` | 🟢 | virtual | Virtual slot [1] (no this use). Calls FUN_006fb9c0 and FUN_006fb860 to get the current allocator; if one exists it calls its vtable slot +4, else if the global allocator DAT_01bdf280 is set it calls that slot +4, else a… |
| `00919550` | `SoeUtil::HashListMapClientLuaMetricsEntry20FreeNode` | 🟢 | virtual | Virtual slot [2] (decoded from bytes: mov eax,[esp+4]; push 4; push eax; call FreeMemory; add esp,8; ret 4). Frees its pointer argument with alignment 4 through FreeMemory (00401c00). |
| `0091a200` | `SoeUtil::HashListMapClientLuaMetricsEntry20ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fb730 (thiscall on the container with the node pushed), then if the flag argument bit 0 is set free… |
| `0091d980` | `HashListMap_ClientLuaMetrics_Dtor` | ✅ | dtor | Destructor of the metrics hash container: stores the class vftable and drains the node list at +8 through FUN_008fb730. |
| `01044bb0` | `HashListMap_ClientLuaMetricsEntry::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores SoeUtil::HashListMap<int,ClientLuaMetrics::Entry,20,-1>::vftable at *this, zeroes this+0x10, +0x8, +0xc and +0x4, memsets 0x50 bytes at this+0x14 to zero, and returns this. |
