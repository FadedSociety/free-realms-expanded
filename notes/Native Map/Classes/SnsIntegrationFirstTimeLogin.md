# Class: SnsIntegrationFirstTimeLogin

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb57d0` | `— ⚠reject` | ⚪ | ctor | Constructor of the SNS first-time-login packet: runs the family initialiser 00bb56a0 with sub-opcode 2 and installs SnsIntegrationFirstTimeLogin::vftable. |
| `00bb5be0` | `SnsIntegrationFirstTimeLogin::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0: installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this. |
