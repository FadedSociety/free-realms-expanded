# Class: SnsIntegrationSecureSignatureResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb60b0` | `— ⚠reject` | ⚪ | ctor | Constructor of the SNS secure-signature-response packet: runs the family initialiser 00bb56a0 with sub-opcode 1, installs SnsIntegrationSecureSignatureResponse::vftable and initialises the embedded IString at +0x10 (ISt… |
| `00bb6120` | `SnsIntegrationSecureSignatureResponse::~SnsIntegrationSecureSignatureResponse` | ✅ | dtor | Destructor: rewrites the IString vtable at +0x10 and, when its capacity word is positive, decrements the shared buffer refcount and releases the buffer through IString vtable slot 2, then installs BaseSnsIntegrationPack… |
| `00bb6470` | `SnsIntegrationSecureSignatureResponse::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6120 and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this. |
