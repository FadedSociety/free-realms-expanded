# Class: ClientAccountDataManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00725030` | `ClientAccountDataManager_ClearDefinitions` | 🟢 | method | Drains every definition container: item definitions through 007227c0, the CharacterCreateNames list by repeated 00724150, then the four character-create lists through 00722850/007228f0/00722990/00722a30, and finally cal… |
| `00725180` | `ClientAccountDataManager::ClientAccountDataManager` | ✅ | ctor | Constructs the ItemDefinitionManager base chain through 007218e0, installs the two ClientAccountDataManager vtables at +0/+4, clears +8/+0xc/+0x10/+0x14, memsets the 40000-byte (10000-bucket) table at +0x18, initializes… |
| `00725470` | `ClientAccountDataManager::~ClientAccountDataManager` | ✅ | dtor | Installs the ClientAccountDataManager vtables at +0/+4, calls ClearDefinitions (00725030) to release all definition data, then drains the embedded List<CCNamesDefinition> at +0x9c58 through 00724150 and finally rewrites… |
| `00725520` | `ClientAccountDataManager_ScalarDeletingDtor` | 🟢 | dtor | Calls the ClientAccountDataManager destructor body 00725470 for this and frees through FUN_00401c00 when bit 0 of the flag byte is set. |
