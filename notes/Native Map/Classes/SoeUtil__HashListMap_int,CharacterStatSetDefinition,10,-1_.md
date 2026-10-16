# Class: SoeUtil::HashListMap<int,CharacterStatSetDefinition,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102cbf0` | `HashListMap::RemoveNode` | ✅ | method | Thiscall (ECX = table, param_2 = node): removes node from its bucket chain (0102bea0), unlinks it from the order list (table+0x8 first, table+0xc last, node+0x14/+0x18), decrements count at table+0x10, clears node+0x4 (… |
| `0102cce0` | `HashListMap::DtorBody → SoeUtil::HashListMap<int,CharacterStatSetDefinition,10,…` | ✅ | dtor | Fastcall destructor body (ECX = table): stores the HashListMap<...,10> vftable at this and repeatedly removes the first node via 0102cbf0 until table+0x8 is 0; no free. |
| `0102cd10` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (thiscall, flag byte): stores the HashListMap<...,10> vftable, removes nodes via 0102cbf0 until table+0x8 is 0, and calls _free(this) when bit 0 of the flag is set; returns this. |
| `0102cd50` | `HashListMap::DtorBodyEH` | ✅ | dtor | EH-framed destructor body (ECX = table): stores the HashListMap<...,10> vftable and removes nodes via 0102cbf0 while table+0x8 is non-null; no free. |
| `0102cdc0` | `HashListMap::DtorWrapper` | 🟢 | dtor | EH-framed wrapper (push -1, SEH frame, saves ECX, then call 0102cd50 with the same this) that runs the destructor body and returns. |
| `0102ce10` | `HashListMap::Ctor → SoeUtil::HashListMap<int,CharacterStatSetDefinition,10,-1>:…` | ✅ | ctor | Fastcall constructor body (ECX = table): stores the HashListMap<...,10> vftable, zeroes table+0x4, +0x8, +0xc, +0x10 and the ten bucket heads at +0x14..+0x38, then stores the HashList<CharacterStatSetDefinition,10,-1> v… |
