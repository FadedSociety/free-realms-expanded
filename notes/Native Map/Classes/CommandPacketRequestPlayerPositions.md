# Class: CommandPacketRequestPlayerPositions

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14bf0` | `CommandPacketRequestPlayerPositions::~CommandPacketRequestPlayerPositions` | ✅ | dtor | Destructor body: stores CommandPacketRequestPlayerPositions::vftable, then BaseCommandPacket::vftable and finally BasePacket::vftable at *this. |
| `00c14c50` | `CommandPacketRequestPlayerPositions::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of CommandPacketRequestPlayerPositions): resets the three vtables (own, BaseCommandPacket, BasePacket) and frees the object through _free when bit 0 of param_2 is set. |
| `00c1ab00` | `— ⚠reject` | ⚪ | ctor | Calls the command packet base constructor with sub-opcode 0x22 and zero, installs its vtable, copies two global words, and clears two bytes. |
