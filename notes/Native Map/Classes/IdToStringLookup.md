# Class: IdToStringLookup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00725f40` | `IdToStringLookup::IdToStringLookup` | ✅ | ctor | Installs BaseDefinitionManager::vftable then IdToStringLookup::vftable, clears +4/+8, memsets the 400-byte (100-bucket) table at +0xc, constructs the embedded IStringNoShare member through 00722da0 and, when param_2 is … |
| `00725ff0` | `IdToStringLookup::~IdToStringLookup` | ✅ | dtor | Installs IdToStringLookup::vftable, calls RemoveAll (00725d90) to free every entry, destroys the embedded IStringNoShare through 00722bf0 and finishes with the BaseDefinitionManager vtable. |
