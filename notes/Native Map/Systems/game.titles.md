# System: game.titles

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097c390` | `PlayerTitleTemplateExpand` |  | Builds a player title text from a title descriptor: prefix, suffix or alias form, falling back to the player name. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00b49cd0` | `TitleManagerDtor` |  | Destruction of the client-side player-title registry that 00b49ed0 constructs; the same +0x98/+0x9c members are used by 00b49a10's setter. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b4a190` | `ReadTitleEntryPair` |  | Title entry reader used by the update-all path of the title packet handler 00b4a3a0 before the collected titles are applied with FUN_0090a890. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |

