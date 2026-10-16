# Class: T4BaseStream

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00e73ac0` | `T4BaseStream::T4BaseStream` | ✅ | ctor | thiscall, one stack argument: stores T4BaseStream::vftable at [this], zeroes the dwords at +4, +8 and +0xc, stores the argument at +0x10, and zeroes the bytes at +0x14, +0x15 and +0x16. Returns with ret 4. |
| `00e73bc0` | `T4BaseStream::T4BaseStream ?` | 🟡 | ctor | thiscall, three stack arguments (stored at +4, +8 and the text pointer at +0xc): stores T4BaseStream::vftable, stores arg 1 at +4 and arg 2 at +8; when the third argument is -1 it takes the text pointer from 00e706b0 (o… |
| `00e73cd0` | `T4BaseStream::~T4BaseStream` | ✅ | dtor | fastcall (ECX = this): stores T4BaseStream::vftable; when the owned sub-object pointer at +4 is non-NULL and the bytes at +0x14 and +0x15 are zero, it calls virtual slot [0] of that sub-object with argument 1 (its scala… |
| `00e748c0` | `T4BaseStream::~T4BaseStream_Deleting` | ✅ | dtor | thiscall, byte flag in stack arg 1: stores T4BaseStream::vftable; when the owned buffer (+4) is non-NULL and bytes +0x15 and +0x14 are both zero it calls the buffer's virtual slot [0] with argument 1; when bit 0 of the … |
