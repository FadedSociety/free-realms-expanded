# Class: ClientHousingPacketSaveFixture

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aad140` | `ClientHousingPacketSaveFixture::ctor` | ✅ | ctor | Constructor of the save-fixture request: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 5 (base opcode 127), stamps ClientHousingPacketSaveFixture::vftable at +0 and initialises the embedded … |
| `00ab0640` | `ClientHousingPacketSaveFixtureScalarDeletingDtor` | ✅ | dtor | Calls the BaseHousingPacket destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00abf620` | `ClientHousingPacketSaveFixture_SerializeBody` | ✅ | function | Body writer for housing subtype 5: emits the 127 family header with FUN_00aba910, writes the u64 at source+0x10 with the u64 emitter FUN_008fd1a0, writes the two groups of four dwords at source+0x20 and source+0x30 with… |
