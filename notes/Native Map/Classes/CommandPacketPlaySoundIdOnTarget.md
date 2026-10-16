# Class: CommandPacketPlaySoundIdOnTarget

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98680` | `CommandPacketPlaySoundIdOnTarget::CommandPacketPlaySoundIdOnTarget` | ✅ | ctor | Calls 008a2820 with sub-opcode 0x27 and a zero extra word, then stores CommandPacketPlaySoundIdOnTarget::vftable, zeroes +0x10, stores Target::vftable at +0x14 and zeroes +0x18. |
| `00a986f0` | `CommandPacketPlaySoundIdOnTarget::~CommandPacketPlaySoundIdOnTarget` | 🟢 | dtor | Calls the Target sub-object destructor 0101b370 (whose body writes Target::vftable and releases the pointer at Target+4) on the embedded Target member, then stores BaseCommandPacket::vftable and BasePacket::vftable into… |
| `00a99cb0` | `CommandPacketPlaySoundIdOnTarget::ScalarDeletingDtor` | ✅ | dtor | Calls 00a986f0 (the BaseCommandPacket destructor body) on this, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of CommandPacketPlaySoundIdOnT… |
| `00a9d360` | `CommandPacketPlaySoundIdOnTarget::ReadBody ?` | 🟡 | method | Thiscall with a reader object. Calls 00a9cb50 to read two 16-bit words, reads a dword into this+0x10 with the bounded-cursor rule, then calls vtable slot +4 of the object at this+0x14 with the reader as the argument. |
