# Class: SnsIntegrationGuildCreated

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb62b0` | `— ⚠reject` | ⚪ | ctor | Constructor of the SNS guild-created packet: runs the family initialiser 00bb56a0 with sub-opcode 3, installs SnsIntegrationGuildCreated::vftable and initialises the embedded IString at +0x10 (IString vtable, static emp… |
| `00bb6320` | `SnsIntegrationGuildCreated::~SnsIntegrationGuildCreated` | ✅ | dtor | Destructor: rewrites the IString vtable at +0x10 and, when its capacity word is positive, decrements the shared buffer refcount and releases it through IString vtable slot 2, then installs BaseSnsIntegrationPacket::vfta… |
| `00bb64b0` | `SnsIntegrationGuildCreated::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6320 and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this. |
