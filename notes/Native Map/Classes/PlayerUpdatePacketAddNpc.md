# Class: PlayerUpdatePacketAddNpc

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091cac0` | `PlayerUpdatePacketAddNpc::Ctor` | ✅ | ctor | Constructor of the AddNpc packet: runs BasePlayerUpdatePacket::BasePlayerUpdatePacket (FUN_0089fbb0, opcode 0x23 stored by the base), installs PlayerUpdatePacketAddNpc::vftable, and initializes the packet fields (two 8-… |
| `0091cdd0` | `PlayerUpdatePacketAddNpc::~PlayerUpdatePacketAddNpc` | 🟢 | dtor | Destructor of the AddNpc packet: releases the SoeUtil::IString fields at +0x230/+0x220/+0x210, tears down the CharacterVariables subobject at +0x188 (vtable restore plus FUN_0101b370 and FUN_00777340), drains the Effect… |
| `00921a10` | `PlayerUpdatePacketAddNpc_ReadBody` | ✅ | method | Parses the AddNpc body from the bounded reader param_2 into the record at param_1: first reads two dwords into +0x10/+0x14, then dwords, bytes and floats in order (booleans at +0x20, +0x9c, +0xbc, +0xd4; the NaN check s… |
| `00922960` | `PlayerUpdatePacketAddNpc_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091cdd0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
