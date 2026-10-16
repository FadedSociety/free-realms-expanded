# Class: SocialShareDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a714e0` | `SocialShareDataSource::FieldName` | ✅ | virtual | Switch on the int argument: returns the C string literal 'Provider' for 0, 'Status' for 1, 'Enabled' for 2, the address PTR_LAB_01815d30 for 3 and DAT_0180c430 otherwise. Returns a pointer in EAX; no state is read. |
| `00a71c60` | `SocialShareDataSource::RebuildRows` | 🟢 | method | Calls 00769710 and returns 0 when the object at (result+4)+0x234 is zero. Otherwise it builds a header row from the literal 01813268 into a local 40-byte record (two IString<char> assigns), appends it to the array at th… |
| `00a71e00` | `SocialShareDataSource::DtorBody` | ✅ | dtor | Stores the SocialShareDataSource vtables at this+0, +0x30 and +0x34, then stores the SocialShareData array vtable at +0x254, truncates that array through 00a718a0 with its count at +0x25c, frees its buffer at +0x258 thr… |
| `00a71eb0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [4] (row, column[, out]). Calls vtable slot +8 of this for the row count and returns 0 when row >= count. Otherwise it switches on the column (third argument): column 0 gets the row element (00a71dd0), form… |
| `00a720f0` | `SocialShareDataSource::Ctor` | ✅ | ctor | Calls the base constructor 00922890 with the name string BBaseClient.SocialShare, installs the SocialShareDataSource vtables at +0, +0x30 and +0x34, installs the empty SocialShareData array vtable at +0x254 and zeroes i… |
| `00a721a0` | `SocialShareDataSource::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a71e00 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
