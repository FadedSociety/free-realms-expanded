# Class: SoeUtil::HashListMap<int,ScaleformConversionLayer::GFxKeyRecord,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00840840` | `HashListMapDtorBody` | 🟢 | dtor | __fastcall destructor body (no free): stores the HashListMap vtable, then removes nodes from the head (this[2]) with 0083eff0 (BucketChainRemoveAndFree) until this[2] is null. |
| `00840870` | `HashListMapScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor: stores the HashListMap vtable, removes all nodes with 0083eff0 until this[2] is null, and calls _free(this) when bit 0 of arg1 is set. Bytes after the _free call: add esp,4; mov eax,… |
| `008414c0` | `HashListMapDtorBodyEh` | 🟢 | dtor | __fastcall, SEH frame (102 bytes): stores the HashListMap vtable, then removes nodes from the head (this[2]) with 0083eff0 until this[2] is null. Same loop as the dtor body 00840840; called from three EH unwind funclets. |
| `00842230` | `HashListMapCtor` | 🟢 | ctor | __fastcall constructor (117 bytes, SEH frame): stores the HashListMap vtable, zeroes this[2], this[3], this[4] and this[1], clears the 0x100-byte bucket array at this+0x14 with _memset, then stores the SoeUtil::HashList… |
| `00843f90` | `HashListMapDeletingDtorWrap ?` | 🟡 | dtor | thiscall (90 bytes, SEH frame): calls 008414c0 (HashListMapDtorBodyEh), which stores the HashListMap vtable and removes the nodes, then calls _free(this) when bit 0 of arg1 is set. Returns this (bytes after the _free ca… |
