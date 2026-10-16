# Class: SoeUtil::Array<GuildMemberLocationUpdatePacket::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b18570` | `— ⚠reject` | ⚪ | virtual | Slot [1] of the entry-array vtable: decides the new capacity (growing to requested*5/4, shrinking to current*6/5 when the request is far below the current capacity), returns the inline buffer at +4 when no reallocation … |
| `00b18610` | `SoeUtil::Array::FreeStore → SoeUtil::Array<GuildMemberLocationUpdatePacket::Ent…` | ✅ | virtual | Slot [2] of the entry-array vtable: frees the storage pointer passed in with alignment 8 through 00401c00 and returns; the raw bytes show a pure forwarder that pops one stack argument. |
| `00b18630` | `— ⚠reject` | ⚪ | virtual | Slot [3] of the entry-array vtable: when param_3 is non-zero it tail-jumps to virtual slot 0xc of the other array (delegating the swap), otherwise it exchanges the storage pointer, element size and capacity fields at +4… |
| `00b19050` | `SoeUtil::Array::Destruct → SoeUtil::Array<GuildMemberLocationUpdatePacket::Entr…` | ✅ | dtor | Destructor body of the 0x18-byte entry array: installs the Array vtable, zeroes the element count at +8, frees the storage pointer at +4 with alignment 8 through 00401c00 and nulls it; both callers are unwind funclets 0… |
| `00b19110` | `SoeUtil::Array::ScalarDeletingDtor → SoeUtil::Array<GuildMemberLocationUpdatePa…` | ✅ | dtor | Slot [0] scalar deleting destructor of the entry-array vtable: installs the Array vtable, zeroes the element count, frees the storage at +4 with alignment 8 and nulls it, then frees the object itself with _free when bit… |
