# System: ui.whatsnew

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a3e960` | `WhatsNewDataSource_RowCount` | WhatsNewDataSource | Count accessor of the what's-new data source; it is what drives the UI to show one row per announcement entry, and it lazily materialises the rows on first use. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3e990` | `EmbeddedStrings3_Dtor_18_34_44` | WhatsNewInfo | Element destructor of the 0x54-byte what's-new row type: the array truncation 00a3ef10 and the array reallocation 00a3f160 call it once per removed element. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3ea70` | `EmbeddedStrings3_Ctor_18_34_44` | WhatsNewInfo | String-field constructor of the what's-new row (0x54-byte element of SoeUtil::Array<WhatsNewInfo,0,1>); the caller 00a3f810 creates rows with this layout. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3efb0` | `EmbeddedStrings3_CopyCtor_18_34_44` | WhatsNewInfo | Copy constructor used while SoeUtil::Array<WhatsNewInfo,0,1> is relocated by 00a3f160, so every existing row is deep-copied into the new block. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a3e1b0` | `WhatsNewDataSource_FieldName ⚠downgrade` | WhatsNewDataSource | Exposes the schema of a what's-new announcement row to the generic data-source machinery (script/UI column names), and names the announcement fields the server could populate. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3e670` | `WhatsNewDataSource_Rebuild` | WhatsNewDataSource | Fills the what's-new list rows from the announcement data source; the row-count getter 00a3e960 calls it lazily the first time the UI asks for the count. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3edb0` | `ScalarDelete_a3edb0` | WhatsNewInfo | Heap-delete path for the what's-new row type built by 00a3ea70. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |

