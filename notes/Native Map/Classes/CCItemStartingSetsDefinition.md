# Class: CCItemStartingSetsDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00723200` | `CCItemStartingSetsDefinition::CCItemStartingSetsDefinition` | ✅ | ctor | Installs CCItemStartingSetsDefinition::vftable, clears +4..+0x10, stores param_2 at +0x14, constructs the embedded StringFixed<128> at +0x18 through 00402950 and clears +0xac..+0xd8 (the twelve appearance columns filled… |
| `007232c0` | `CCItemStartingSetsDefinition::~CCItemStartingSetsDefinition` | ✅ | dtor | Installs CCItemStartingSetsDefinition::vftable and destroys the embedded IStringFixed<char,128> at +0x18 through 00401fd0. |
| `00723320` | `CCItemStartingSetsDefinition_ScalarDeletingDtor` | 🟢 | dtor | Calls the starting-set destructor body 007232c0 for this and calls _free when bit 0 of the flag byte is set. |
| `007257d0` | `CCItemStartingSetsDefinition_ParseFields` | 🟢 | method | Tokenizes the first column into +0x14, skips one delimited token, then stores twelve consecutive numeric columns at +0xac, +0xb0, +0xb4, +0xb8, +0xbc, +0xc0, +0xc4, +0xc8, +0xcc, +0xd0, +0xd4 and +0xd8. |
