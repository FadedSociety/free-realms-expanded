# Class: ClientHousingPacketRemoveCustomizationFromFixtureGroupAndType

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ab3e70` | `ClientHousingPacketRemoveCustomizationFromFixtureGroupAndType::Ctor` | ✅ | ctor | Constructs subtype 60, installs its request packet vtable and initializes two fixed 64-byte string fields. |
| `00ab3ee0` | `ClientHousingPacketRemoveCustomizationFromFixtureGroupAndType::ScalarDeletingDt…` | ✅ | dtor | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |
| `00ac0850` | `SendClientHousingPacketRemoveCustomizationFromFixtureGroupAndType` | 🟢 | function | Transport wrapper for 127/60: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910 and appends the two string members at +0xc and +0x60 through FUN_00911c30, wraps the buffer … |
