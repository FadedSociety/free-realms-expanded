# Class: HousingPacketFixtureAsset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ab3fb0` | `HousingPacketFixtureAsset::Ctor` | ✅ | ctor | Constructs subtype 42, installs its vtable, initializes fixed strings and a strong-type array, and stores a supplied word. |
| `00ab4080` | `HousingPacketFixtureAsset::ScalarDeletingDtor` | ✅ | dtor | Runs packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |
| `00ac1f50` | `— ⚠reject` | ⚪ | function | Body parser for the fixture-asset message: initialises the bounded reader with FUN_00ab2bc0, reads two dwords into destination+0xc and +0x10, reads the fixture-definition tail into the destination with FUN_00abeb40, ski… |
