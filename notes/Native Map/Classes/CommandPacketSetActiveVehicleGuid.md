# Class: CommandPacketSetActiveVehicleGuid

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a10340` | `CommandPacketSetActiveVehicleGuid_Ctor` | 🟢 | ctor | Constructor: calls FUN_008a2820(0x1f,0) to set sub-opcode 31 on a command packet, installs CommandPacketSetActiveVehicleGuid::vftable, and calls FUN_0101cd40 which zeroes the member field it is given (*ptr = 0). |
