# Class: SoeUtil::HashListMap<int,ColorDefinition,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a395e0` | `HashListMapClearNodes` | 🟢 | method | Thiscall; repeatedly removes the head node (this+8) until the list is empty: removes it from its bucket chain (bucket = node key at +0x18 modulo 10, buckets at this+0x14 onward, chain link +0x1c), unlinks it from the li… |
| `00a396d0` | `HashListMapDtorBody` | ✅ | dtor | Bytes: mov dword [ecx], vftable(SoeUtil::HashListMap<int,ColorDefinition,10,-1>) then jmp 00a395e0. It installs the map vtable and tail-jumps to HashListMapClearNodes. |
| `00a39760` | `HashListMapScalarDtor` | ✅ | dtor | Virtual slot [0] of SoeUtil::HashListMap<int,ColorDefinition,10,-1>::vftable (binary vtable read). Body shape of a scalar deleting destructor: stores the map vtable, calls 00a395e0 (HashListMapClearNodes), then tests bi… |
| `00a39790` | `HashListMapEhDtorBody` | ✅ | dtor | Called from two EH unwind funclets. Under a small SEH frame it stores the map vtable into *this and calls 00a395e0 (HashListMapClearNodes). No free. |
| `00a397e0` | `HashListMapSehDtorBody` | ✅ | dtor | Stores the map vtable into *this under an SEH frame, calls 00a395e0 (HashListMapClearNodes) and returns. No free. The scalar deleting wrapper 008a6160 calls it before freeing. |
| `00a39a80` | `HashListMapCtor` | ✅ | ctor | Stores the HashListMap<int,ColorDefinition,10,-1> vtable into *this, zeroes this+4 through +0x38 (list head at +8, tail at +0xc, count at +0x10, the 10 bucket heads at +0x14..+0x38), then stores SoeUtil::HashList<ColorD… |
| `00a39b00` | `HashListMap_ColorDefinition10_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: under an SEH frame it installs SoeUtil::HashListMap<int,ColorDefinition,10,-1>::vftable at +0, calls the base teardown FUN_00a395e0, and calls _free(this) only when bit 0 of the flag argument… |
