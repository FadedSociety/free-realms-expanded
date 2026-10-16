# System: ui.options

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4d650` | `OptionsManager_Init` |  | Creates the three client options data sources identified by their exact interface names. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4e1f0` | `OptionsManager_LoadOptionElements` |  | Loads the Options page's element description file. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4e2b0` | `OptionsManager_LoadControllerElements` |  | Loads the Controller-options page's element description file. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4e380` | `OptionsManager_LoadHelpCreditsElements` |  | Loads the Help/Credits page's element description file. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4da10` | `OptionsElementRecord_ScalarDeletingDtor` |  | Deletes a parsed options-element record and its string members. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00c20c80` | `SetSoundChannels` |  | Applies a channel-count preference to the sound settings system. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20d50` | `SetSoundQuality` |  | Applies a sound-quality preference. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20ee0` | `SetGraphicsQuality` |  | Applies the graphics-quality preference. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20f30` | `GetGraphicsQuality` |  | Returns the current graphics-quality preference or a fallback. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20fc0` | `SetTextureQuality` |  | Applies the texture-quality preference. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c21010` | `GetTextureQuality` |  | Returns the current texture-quality preference or a fallback. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c220b0` | `MarkReleaseNotesRead ? ⚠downgrade` |  | Marks the current release notes as read in persisted settings. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4d7a0` | `— ⚠reject` |  | Destructor of the 0xc0-byte record that stores the eleven '^'-separated fields parsed from OptionElements/ControllerElements/HelpCreditsElements files. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4da30` | `— ⚠reject` |  | List-removal helper for the parsed option/controller/help-credits element lists (list header {head,tail,count} stores the node pointer being removed). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4dab0` | `— ⚠reject` |  | Record constructor used by the '^'-delimited element-file parser (00a4dd70) before the eleven fields are filled in. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4dd70` | `— ⚠reject` |  | Loads one of the three options-UI element files (OptionElements.txt, ControllerElements.txt, HelpCreditsElements.txt) into the corresponding list of parsed rows. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4e450` | `— ⚠reject` |  | Initial resource load of the options/help UI; caller 008a9010 ('Failed to reload options manager [prefix=%s]') drives it with the client resource prefix. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00c1f9c0` | `— ⚠reject` |  | Checks whether the client's release-notes build number differs from the last-read value. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

