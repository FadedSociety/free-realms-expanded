# Class: SoeUtil::HashListMap<int,FotomatPortraitData,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bd4620` | `— ⚠reject` | ⚪ | method | Thiscall (ECX = map at manager+0x1bc; layout: head +8, tail +0xc, count +0x10, eight bucket heads at +0x14): calls map vtable slot [1] (no real argument) and, when it returns nonzero, creates a node with 00bd4100 from i… |
| `00bd46f0` | `HashListMap8_RemoveNode` | 🟢 | method | Thiscall (ECX = map, arg = node): finds the node in its bucket (key at node+0x2118 & 7) and unlinks it; relinks the list neighbours (next +0x2110, prev +0x2114), updates head, tail and count (-1); calls 00bd3dc0 on the … |
| `00bd4d10` | `HashListMap8_ClearHead` | 🟢 | method | Thiscall (ECX = map): loops while the list head (+8) is non-null, calling 00bd46f0 with the head node each time; returns nothing (plain ret). |
| `00bd4df0` | `HashListMap8_DtorBody` | 🟢 | dtor | Thiscall (ECX = map): stores the map vftable at +0, then loops while the list head (+8) is non-null, calling 00bd46f0 to remove the head node each time. |
| `00bd4e20` | `HashListMap8_ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor: stores the map vftable, drains the list through 00bd46f0 while the head (+8) is non-null, then calls _free(this) (thunk 00d82463) when bit 0 of the flag byte argument is set; returns… |
| `00bd4e60` | `HashListMap8_DtorBodyEH` | 🟢 | dtor | Thiscall (ECX = map): stores the map vftable at +0, then loops while the list head (+8) is non-null, calling 00bd46f0 on the head node; the same body as the drain in 00bd4df0, with its own EH frame. |
| `00bd50f0` | `HashListMap8_Ctor` | ✅ | ctor | Thiscall ctor (ECX = map): stores the HashListMap<int,FotomatPortraitData,8,-1> vftable at +0, zeroes dwords +4 through +0x30 (head, tail, count and the eight bucket heads), then stores the SoeUtil::HashList<FotomatPort… |
