# Class: PacketPlayerImageData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bd3b40` | `PacketPlayerImageData::PacketPlayerImageData` | ✅ | ctor | Thiscall ctor: calls the BaseFotomatPacket ctor 008a23f0 on this with stack arg 3, stores PacketPlayerImageData::vftable at +0, the SoeUtil::IString<char> vftable at +0x18 with the shared empty buffer at +0x1c and zero … |
| `00bd3be0` | `— ⚠reject` | ⚪ | dtor | Destructor body: sets the Array<unsigned_char> vftable at +0x2100, frees its buffer at +0x2104 through 00401c00 (alignment 1) and zeroes it; releases the IString at +0x18 by decrementing the refcount header at buffer-4 … |
| `00bd3fd0` | `PacketPlayerImageData::ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor: calls the PacketPlayerImageData dtor body 00bd3be0 on this, then calls _free(this) (thunk 00d82463) when bit 0 of the flag byte argument is set; returns this. |
| `00bd4180` | `PacketPlayerImageData::Read` | 🟢 | method | Parses the PacketPlayerImageData body from a reader (reader cursor +8, end +0xc, error flag +0x10): 00bd31d0 reads two 16-bit header values into this+4 and this+8; a 64-bit id is read into this+0x10/+0x14; one byte beco… |
