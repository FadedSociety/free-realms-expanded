# Class: LobbyGame

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00912310` | `— ⚠reject` | ⚪ | method | __fastcall (this=ECX). Takes head node [this+4]; if null returns. Computes bucket = node[3] % 100 with buckets at this+0x10; walks that bucket chain to the node and unlinks it (the head/tail links at this+4, this+8 and … |
| `01048860` | `LobbyGame::RequeueId64` | 🟢 | virtual | Thiscall, one stack arg (param_2, a record with a 64-bit id at +0x10/+0x14). If param_2 is non-null: walks the list at this+0x2c (next at node+8) and unlinks the first node whose two dwords equal param_2's +0x10/+0x14 (… |
| `010488d0` | `LobbyGame::RemoveId64` | ✅ | virtual | Thiscall, two stack args (id low dword, id high dword). Walks the list at this+0x2c for the node whose two dwords equal the arguments. On a match unlinks it (head at this+0x2c, tail at this+0x30, count at this+0x34 decr… |
| `01048940` | `LobbyGame::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores LobbyGame::vftable (01803f28) at *this, zeroes the dwords at +0x10..+0x1c, stores the SoeUtil::List<StrongType<uint64,...>,-1> vtable (018037fc) at +0x28, zeroes +0x34, +0x2c..., sets the by… |
| `01048a00` | `LobbyGame::CtorWithArgs` | ✅ | ctor | Thiscall with four stack args (param_2..param_5; Ghidra). Same base initialisation as LobbyGame::Ctor (01048940): stores LobbyGame::vftable at *this, the list vtable 018037fc at +0x28, zeroes state fields, sets +0x44 = … |
