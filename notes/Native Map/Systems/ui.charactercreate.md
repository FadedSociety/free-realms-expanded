# System: ui.charactercreate

20 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0070fcd0` | `DispatchCharacterCreateOffset` |  | Forwards character-creation offset update to UI/account state. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070fdc0` | `ClientAccountManager::ShowWebServiceErrorMessage` | ClientAccountManager | Displays a web-service error in character-creation popup flow. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070ff40` | `ClientAccountManager::ShowErrorMessage` | ClientAccountManager | Displays a generic character-creation error popup. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0071afa0` | `ShowCharacterCreateHide` |  | Hides the character-creation screen. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071d030` | `ClientAccountInterface::BuildItemData` | ClientAccountInterface | Populates character-creation option rows and visible item metadata. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071daa0` | `BuildCharacterCreateCategoryButtons` |  | Builds initial character-creation category button data. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071e770` | `BuildCharacterNameData` |  | Builds character-name selection lists from account name data. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071efe0` | `ShowCharacterCreateScreen` |  | Dispatches the character-create show event, refreshes related data through helper calls, and updates the state field. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071f030` | `ClientAccountInterface::ChangeItem` | ClientAccountInterface | Character-create appearance handler: applies one appearance-item choice for a category (skin tone, hair style/color, eye color, facial hair, facepaint, wings, clothes, head/face type) and updates the matching race/gende… | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `0071fd60` | `ClientAccountInterface::BuildCategoryItems` | ClientAccountInterface | Builds the character-create category buttons (Skin Tone, Hair Style, Hair Color, Eye Color, Facial Hair, Facepaint, Wings, Clothes, Face Type) and their initial item lists through the Lua CharacterCreate UI. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00c13e70` | `RegisterCharacterCreateUiLuaModule` |  | Publishes the character-creation UI table; it names every character-creation entry point in this shard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14000` | `CreationNameAccepted` |  | Commits the player-chosen character name (first/middle/last parts) to the creation system. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0071efb0` | `RequestCharacterCreateResourcesAndBuildNameData` |  | Requests the character-create resource path, sets a latch, and builds name data; the code does not establish that loading completed. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `00c13b40` | `CreationChangeCategory` |  | Character-creation UI callback that changes the visible customisation category. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13b90` | `CreationChangeItem` |  | Character-creation UI callback that changes the selected item within a customisation category. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13bf0` | `CreationRequestItemData` |  | Character-creation UI callback that asks the creation system for per-item data (e.g. colours/variants). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c13e90` | `CreationSelectCurrentCharacter ? ⚠downgrade` |  | Selects the existing character (by guid) or, without an argument, the default character in the creation UI. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00705870` | `— ⚠reject` |  | Provides race and gender values consumed by the character-creation Lua binding. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00c13da0` | `— ⚠reject` |  | Reports the existing character's race and gender to the creation UI, using -1/-1 to mean 'no selection'. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13f00` | `— ⚠reject` |  | Returns the creation state plus an accompanying string (a name/status label) to the creation UI. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

