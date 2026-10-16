# Class: ItemRecord

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b1e00` | `ItemRecord::Field4PositiveAndField8NonNegative` | ✅ | virtual | Returns 1 when the signed dword at this+4 is greater than zero and the signed dword at this+8 is >= 0, and 0 otherwise. It reads only those two fields, calls nothing and writes nothing. |
| `009ff220` | `ItemRecord::~ItemRecord` | ✅ | dtor | Stores ItemRecord::vftable at *this and returns; it performs no field cleanup or allocation. |
| `00a00100` | `ItemRecord::ItemRecordB ? ⚠rename` | 🟡 | ctor | Second copy of the ItemRecord constructor body: writes ItemRecord::vftable into this and returns. |
| `00a00330` | `ItemRecord::ScalarDeletingDestructor ⚠downgrade` | 🟢 | dtor | Scalar deleting destructor: rewrites this->vtable to ItemRecord::vftable and calls _free(this) when bit 0 of the flag is set. |
| `00a00ca0` | `ItemRecord::ItemRecordCopy` | ✅ | ctor | Copy constructor: writes ItemRecord::vftable into this and copies the three dwords at source+4, +8 and +0xc into this+4, +8 and +0xc. |
| `00a00ea0` | `ItemRecord::ScalarDeletingDestructorB` | ✅ | dtor | Scalar deleting destructor in an SEH frame: rewrites this->vtable to ItemRecord::vftable and frees this when bit 0 of the flag is set (second copy of 00a00330). |
| `00a84370` | `ItemRecordCtor` | ✅ | ctor | Constructor of an ItemRecord: stores ItemRecord::vftable and copies two dword values supplied through param_2 and param_3 into +4 and +8, then zeroes +0xc. |
| `00b2a4d0` | `CheckItemRecordConditions ?` | 🟡 | method | Checks several runtime prerequisites and item flags, consults item definitions for linked records, and returns 0, 1, or 2 according to the tested conditions. |
| `00bda810` | `ItemRecord::ItemRecord` | ✅ | ctor | Constructor: installs ItemRecord::vftable at object+0 and copies the dwords pointed to by the three pointer arguments into +4, +8 and +0xc (the vftable plus three dwords make a 16-byte record). |
| `00bffd20` | `ItemRecord::Ctor` | ✅ | ctor | Installs ItemRecord::vftable and clears fields at offsets 4, 8, 16, and 20. |
| `00c00490` | `ItemRecord::DtorBody` | ✅ | dtor | Restores ItemRecord::vftable at the start of the record and returns. |
| `00c00520` | `ItemRecord::DtorBody` | ✅ | dtor | Restores ItemRecord::vftable at the start of the record and returns. |
| `00c00680` | `CoinStoreItemRecord::Ctor ? ⚠rename` | 🟡 | ctor | Calls 00bffd20 on a temporary record and returns the supplied pointer. |
| `00c006d0` | `ItemRecord::ScalarDeletingDtor` | ✅ | dtor | Restores ItemRecord::vftable and frees this only when flag bit 0 is set; otherwise returns this. |
| `00c00840` | `ItemRecordList::RemoveNode` | ✅ | function | Unlinks the supplied node using its previous pointer at +0x1c and next pointer at +0x18, updates head or tail, decrements count, restores ItemRecord vtable, and frees the node. |
| `00c707e0` | `ItemRecord::CopyCtor` | ✅ | ctor | Stores ItemRecord::vftable, then copies the five words at source +4..+0x14 into words [1]..[5] of the new object (six words total). Returns nothing. |
| `01045c10` | `ItemRecord::FormatDebugString` | ✅ | virtual | Thiscall, one stack arg (param_2, an IString-like object). Calls 00402fd0 with param_2, the format string "ItemRecord: {Definition: %d, Tint: %d}" and the dwords at this+4 and this+8, then returns the dword at param_2+4. |
