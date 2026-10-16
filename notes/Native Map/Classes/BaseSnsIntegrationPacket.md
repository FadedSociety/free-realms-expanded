# Class: BaseSnsIntegrationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb5640` | `— ⚠reject` | ⚪ | ctor | Constructor body: installs BasePacket::vftable, stores the base opcode 0xc5 (197) at +4, zeroes the sub-opcode at +8 and the payload slot at +0xc, then installs BaseSnsIntegrationPacket::vftable. |
| `00bb56a0` | `— ⚠reject` | ⚪ | method | Initialisation used by every SNS sub-packet constructor: installs BasePacket::vftable, stores the base opcode 0xc5 at +4, installs BaseSnsIntegrationPacket::vftable and stores the sub-opcode argument at +8 while zeroing… |
| `00bb5710` | `BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket` | ✅ | dtor | Destructor body: installs BaseSnsIntegrationPacket::vftable and then BasePacket::vftable into this+0 and returns; it is called from fifteen MSVC unwind funclets, so it is the base cleanup for the SNS packet classes. |
| `00bb5760` | `BaseSnsIntegrationPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0: installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this. |
| `00bb5830` | `BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket_2` | 🟢 | dtor | Second base-destructor body of the SNS family (same shape as 00bb5710, 76 bytes): installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable into this+0; invoked from the unwind funclet 015e8c10. |
| `00bb58e0` | `BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket_3` | 🟢 | dtor | Third base-destructor body of the SNS family (same shape): installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable into this+0; invoked from the unwind funclet 015e8c70. |
