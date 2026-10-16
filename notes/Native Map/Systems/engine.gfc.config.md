# System: engine.gfc.config

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00408920` | `LoadTooltipTheme` |  | Theme attribute loader of the GFC tooltip control: it maps the theme XML/config keys onto the tooltip control's fields. | [[Native Map/Shards/S0002 00406c2f-0040afca|S0002]] |
| `00408aac` | `SaveTooltipTheme` |  | Theme attribute serializer of the GFC tooltip control, the mirror of LoadTooltipTheme (00408920). | [[Native Map/Shards/S0002 00406c2f-0040afca|S0002]] |
| `004094c7` | `SaveLayoutTheme` |  | Theme attribute serializer of the GFC layout control, sibling of SaveTooltipTheme; three callers in the 0x415xxx theme-loading region. | [[Native Map/Shards/S0002 00406c2f-0040afca|S0002]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00401d80` | `ConfigReadInt64` |  | Integer leaf of the client's config/ini access path; the config wrapper 00403280 calls it to convert a read string to a number. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00403280` | `ConfigReadInt64` |  | Reads one integer value out of the client configuration; called from the program entry 00403340. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |

