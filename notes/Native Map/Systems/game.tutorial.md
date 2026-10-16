# System: game.tutorial

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a89a0` | `SetTutorialModeAndEnsureManager` |  | Original tutorial-mode entry: turns the mode flag on and lazily creates the tutorial manager; both are read by Ui.IsInNPXTutorial and disposed on encounter entry (see the two vault notes that cite this address). | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008a8a30` | `DisposeTutorialManagerIfPresent` |  | Manager teardown used by the encounter-entry path (single caller 00aa36c0); the field review note records that the pointer stays set after this call is reached. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008a9200` | `SetTutorialFlag780` |  | Writes the NPX-tutorial mode flag that Ui.IsInNPXTutorial reads (player+0x780 in the vault's field review); the tutorial notes cite this exact address. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |

