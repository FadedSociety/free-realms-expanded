# Class: ClientHousingPacketApplyCustomizationToFixtureGroupAndType

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ab3dd0` | `ClientHousingPacketApplyCustomizationToFixtureGroupAndType::Ctor` | ✅ | ctor | Constructs subtype 59, installs its request packet vtable and initializes an embedded value and two fixed 64-byte strings. |
| `00ab3e50` | `ClientHousingPacketApplyCustomizationToFixtureGroupAndType::ScalarDeletingDtor` | ✅ | dtor | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |
| `00abcce0` | `ClientHousingPacketApplyCustomizationToFixtureGroupAndType_SerializeBody` | 🟢 | function | Calls FUN_00aba910 for the housing header, writes the dword at source+0xc through FUN_007cc400, then calls FUN_00911c30 on string members at source+0x10 and source+0x64. |
| `00ac0740` | `SendClientHousingPacketApplyCustomizationToFixtureGroupAndType` | 🟢 | function | Transport wrapper for 127/59: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcce0 over the caller's packet (one u32 plus two string blocks), wraps the produced buffer in a tunneled C… |
