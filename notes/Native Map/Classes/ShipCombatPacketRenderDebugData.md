# Class: ShipCombatPacketRenderDebugData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a65460` | `ShipCombatPacketRenderDebugData::ShipCombatPacketRenderDebugData` | ✅ | ctor | Constructor of the RenderDebugData packet: base-initialises through 00a4ec00(0x23) (base opcode 0x5e plus sub-opcode 0x23), installs ShipCombatPacketRenderDebugData::vftable and initialises three empty debug lists by st… |
| `00a654f0` | `ShipCombatPacketRenderDebugData_Dtor` | ✅ | dtor | Destructor body of the RenderDebugData packet: resets the member lists to their list vtables and drains them (text list at +0x2c through 00a5d0f0, line list at +0x1c through 00a58180, sphere list at +0xc through 00a5812… |
| `00a670f0` | `ShipCombatPacketRenderDebugData_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the RenderDebugData packet: runs the packet destructor 00a654f0 and, when bit 0 of param_2 is set, releases the object through the _free thunk 00d82463. |
