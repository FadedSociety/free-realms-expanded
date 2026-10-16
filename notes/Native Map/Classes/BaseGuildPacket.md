# Class: BaseGuildPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09b70` | `— ⚠reject` | ⚪ | ctor | Base constructor of every guild packet: installs BasePacket::vftable, stores the family opcode 0x81 (129) in the packet id slot param_1[1], installs BaseGuildPacket::vftable and stores the sub-opcode argument at param_1… |
| `00a09be0` | `BaseGuildPacket::~BaseGuildPacket` | ✅ | dtor | Destructor body: rewrites the vtable to BaseGuildPacket::vftable, then to BasePacket::vftable; no members released. |
| `00a09c30` | `BaseGuildPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the guild base packet: resets the two vtables and frees this when the delete flag is set. |
| `00a09ca0` | `BaseGuildPacket::~BaseGuildPacket_B` | ✅ | dtor | Second emission of the guild base destructor body (BaseGuildPacket then BasePacket vtable reset). |
| `00a09cf0` | `BaseGuildPacket::~BaseGuildPacket_C` | ✅ | dtor | Third emission of the guild base destructor body (BaseGuildPacket then BasePacket vtable reset). |
| `00a09d40` | `BaseGuildPacket::~BaseGuildPacket_D` | ✅ | dtor | Fourth emission of the guild base destructor body (BaseGuildPacket then BasePacket vtable reset). |
| `00a151d0` | `— ⚠reject` | ⚪ | dtor | Destructor body of a guild packet: resets the IString at +0x18, releases its shared buffer when the refcount at buffer-4 drops to zero via virtual slot 2, then rewrites the vtable to BaseGuildPacket and BasePacket. |
| `00b17220` | `BaseGuildPacket::BaseGuildPacket` | ✅ | ctor | Base constructor of the guild packet family: stores BasePacket::vftable into *this, writes 0x81 (129) into the packet id slot at +4, installs BaseGuildPacket::vftable and zeroes the sub-opcode slot at +8. |
