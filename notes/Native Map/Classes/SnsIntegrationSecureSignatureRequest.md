# Class: SnsIntegrationSecureSignatureRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb61b0` | `SnsIntegrationSecureSignatureRequest::SnsIntegrationSecureSignatureRequest` | ✅ | ctor | Constructor of the SNS secure-signature-request packet: runs the family initialiser 00bb56a0 with sub-opcode 6, installs SnsIntegrationSecureSignatureRequest::vftable and initialises the embedded IString at +0x10 (IStri… |
| `00bb6220` | `SnsIntegrationSecureSignatureRequest::~SnsIntegrationSecureSignatureRequest` | ✅ | dtor | Destructor: rewrites the IString vtable at +0x10 and, when its capacity word is positive, decrements the shared buffer refcount and releases it through IString vtable slot 2, then installs BaseSnsIntegrationPacket::vfta… |
| `00bb6490` | `SnsIntegrationSecureSignatureRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6220 and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this. |
