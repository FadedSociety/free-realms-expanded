# Class: GuildMemberLocationRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19760` | `GuildMemberLocationRequest::GuildMemberLocationRequest` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0xf (15), installs GuildMemberLocationRequest::vftable, stores the two header arguments at +0x10 and +0x14, and initializes an embedded SoeUtil::Arr… |
| `00b197e0` | `GuildMemberLocationRequest::Destruct` | 🟢 | dtor | Destructor body of the member location request: rewrites the embedded guid-array vtable at +0x18, zeroes the array count, frees the array storage at +0x1c with alignment 8 through 00401c00 and nulls the pointer, then re… |
| `00b1a430` | `GuildMemberLocationRequest::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildMemberLocationRequest::vftable: calls the destructor body 00b197e0 and frees the object with _free when bit 0 of the flag argument is set. |
