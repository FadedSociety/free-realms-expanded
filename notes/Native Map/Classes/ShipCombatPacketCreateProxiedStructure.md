# Class: ShipCombatPacketCreateProxiedStructure

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a620d0` | `— ⚠reject` | ⚪ | dtor | Destroys the packet's member state, calls the packet base destructor, then resets the object to BaseShipCombatPacket and BasePacket vtables. |
| `00a653c0` | `ShipCombatPacketCreateProxiedStructure::ShipCombatPacketCreateProxiedStructure` | ✅ | ctor | Constructor of the CreateProxiedStructure packet: base-initialises through 00a4ec00(0x1f) (which stores the BasePacket and BaseShipCombatPacket vtables, the base opcode 0x5e and the sub-opcode 0x1f), installs ShipCombat… |
| `00a65440` | `ShipCombatPacketCreateProxiedStructure_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the CreateProxiedStructure packet: runs the packet destructor body 00a620d0 and, when bit 0 of param_2 is set, releases the object through the _free thunk 00d82463. |
