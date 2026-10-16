# Class: TintGroupDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a72780` | `TintGroupDataSource::ColumnTitle` | ✅ | virtual | Virtual slot [1]. Switch on the int argument: 0 returns the literal Tint Group ID, 1 returns the literal Data (<ID> - <Value>[,...]), any other value returns the literal <error>. Pure lookup; no state is read. |
| `00a733e0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. If the byte at this+0x228 is zero it calls 00a732e0 with ECX = this-0x30. Then it sets ECX = this+4 and tail-jumps to 010c8f30 (jmp rel32), so the callee result is returned to the caller. Does not set … |
| `00a73400` | `— ⚠reject` | ⚪ | virtual | Virtual slot [4] (row, column, out). When the byte at this+0x228 is set it calls 00a732e0 first. It resolves the row key through vtable slot +0x10 of the sub-object at this+4 (argument row), finds the node with that key… |
| `00a73610` | `TintGroupDataSource::DtorBody` | ✅ | dtor | Stores the TintGroupDataSource vtables at this, +0x30 and +0x34, sets a local cleanup flag, stores the HashMap<int,IString<unsigned_short>> vtable at +0x2a4 and calls 00a73160 (clear all nodes) on that map, then stores … |
| `00a736c0` | `TintGroupDataSource::Ctor` | ✅ | ctor | Calls the base constructor 00922890 with the name literal BaseClient.TintGroups, stores its stack argument at this+0x254, installs the TintGroupDataSource vtables at +0, +0x30 and +0x34, sets the dirty byte at +0x258 to… |
| `00a73760` | `TintGroupDataSource::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] of the secondary vtable. Calls the dtor body 00a73610 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
