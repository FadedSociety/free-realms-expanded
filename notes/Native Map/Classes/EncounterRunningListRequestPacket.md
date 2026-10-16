# Class: EncounterRunningListRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a22300` | `— ⚠reject` | ⚪ | ctor | Constructs the packet in ecx by calling the encounter family constructor 008a2fd0 with sub-opcode 0x81 (129) - which stores opcode 0x29 (41) and the sub-opcode - and then stores EncounterRunningListRequestPacket::vftabl… |
| `00a22ec0` | `EncounterRunningListRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseEncounterPacket::vftable and BasePacket::vftable and calls _free on the object when bit 0 of the flag argument is set, otherwise returns the object pointer. |
