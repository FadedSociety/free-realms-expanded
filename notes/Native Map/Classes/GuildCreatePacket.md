# Class: GuildCreatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0ea50` | `GuildCreatePacket::GuildCreatePacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 1, stores GuildCreatePacket::vftable at *this, and initialises three SoeUtil::IString members: the vtable pointers at +0xc, +0x1c and +0x… |
| `00c0eae0` | `— ⚠reject` | ⚪ | dtor | Destructor body: releases the three embedded SoeUtil::IString members at +0xc, +0x1c and +0x2c, each by decrementing an interlocked reference count stored just before the string buffer and invoking the IString virtual s… |
| `00c0ee30` | `GuildCreatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildCreatePacket): calls the destructor body FUN_00c0eae0 (which releases the three embedded strings and resets the vtables) and frees the object through _free when bit 0 of… |
