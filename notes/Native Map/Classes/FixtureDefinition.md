# Class: FixtureDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00abd600` | `FixtureDefinitionDtor` | 🟢 | dtor | Destructor of a fixture-definition record: destroys the embedded SoeUtil::HashListMap<int,FixtureDefinition::FixtureAction> member at +0xc8 through FUN_00abd310, then releases two fixed 64-byte string members with the I… |
| `00abeb40` | `ReadFixtureDefinitionTail` | ✅ | function | Reads the fixture-definition tail (50 bytes in the captured empty-string/empty-action case; variable strings and actions change the total length) from the bounded reader (cursor +8, end +0xc, error byte +0x10) into the … |
| `00abf010` | `FixtureDefinitionScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor of a fixture-definition record: runs the destructor body 00abd600 in place and frees the object through _free when the low bit of the flag argument is set. Bytes decoded as 56 8b f1 e8 .. f6 4… |
