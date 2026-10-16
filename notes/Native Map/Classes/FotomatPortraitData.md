# Class: FotomatPortraitData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bd3dc0` | `FotomatPortraitData::ReleaseImageArrays` | 🟢 | dtor | Thiscall (ECX = record): for the three Array<unsigned_char,0,1> members at +0x2100, +0x20f0 and +0x20e0 it sets the array vftable, frees the buffer through 00401c00 (alignment 1), and zeroes buffer and size; ends by cal… |
| `00bd3f10` | `FotomatPortraitData::FotomatPortraitData` | 🟢 | ctor | Thiscall ctor with two stack dwords: stores arg1 at +0 and arg2 at +4, constructs an embedded object at +0x10 via 00ad5f60, and sets three Array<unsigned_char,0,1> vftables at +0x20e0, +0x20f0 and +0x2100 with zeroed bu… |
| `00bd4100` | `— ⚠reject` | ⚪ | ctor | Thiscall with two stack args: stores the dword at *arg1 into this+0x2118 (the hash key), calls 00bd3f10 on this with the two dwords at *arg2 (the id halves), and returns this (ret 8). |
| `00bd4250` | `FotomatPortraitData::ScalarDeletingDtor` | 🟢 | dtor | Thiscall scalar deleting destructor: calls 00bd3dc0 (release of the three image arrays) on this, then calls _free(this) (thunk 00d82463) when bit 0 of the flag byte argument is set; returns this. |
