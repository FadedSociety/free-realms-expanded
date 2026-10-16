# System: ui.dstree

17 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099ef20` | `GuiDsTreeNode::GuiDsTreeNode` | GuiDsTreeNode | Constructs a GUI data-source tree node. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099efb0` | `GuiDsTreeNode::ScalarDeletingDtor` | GuiDsTreeNode | Scalar deleting destructor for a GUI data-source tree node. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009a0020` | `CategoriesTreeNodeGetMember34` | InGamePurchaseDataSource::StoreBundleCa… | RTTI identifies the category-tree-node virtual; the returned member meaning is unknown. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009a0050` | `CategoriesTreeNodeGetField28` | InGamePurchaseDataSource::StoreBundleCa… | Returns a category-node field; semantic meaning is unknown. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00c11da0` | `PushDsTreeNodeUserdata` |  | Wraps an engine tree node for Lua so scripts can call DsTreeNode methods on it. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11e20` | `CheckDsTreeNodeArg` |  | Argument accessor used by every DsTreeNode method binding in this shard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12010` | `RegisterDsTreeNodeLuaModule` |  | Creates the DsTreeNode metatable and method table used by the tree scripting bindings; called by the DsTree registration 00c12300. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12120` | `CheckDsTreeArg` |  | Argument accessor for the DsTree method bindings. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c121c0` | `DsTreeEnableScriptEvents` |  | Turns on script-event delivery for the tree (the boolean twin of DisableScriptEvents 00c12200). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12200` | `DsTreeDisableScriptEvents` |  | Turns off script-event delivery for the tree. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12300` | `RegisterDsTreeLuaModule` |  | Creates the DsTree metatable and methods; the only caller is the module-registration dispatcher 00976070, i.e. this is exported to the client's Lua UI at startup. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099ef90` | `GetStaticPointer175B400` | GuiDsTreeNode | GuiDsTreeNode virtual slot 4; the pointed-to bytes are not decoded, so their meaning is unknown. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00c11f20` | `DsTreeNodeGetDataName` |  | Returns the column/data name for a data index of the wrapped tree node; the DsTreeNode registration table 00c12010 binds this address under the name GetDataName. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12180` | `DsTreeGetRootNode` |  | Hands the tree's root node to Lua as a DsTreeNode handle. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c123e0` | `DsTreeNodeGetChildCount` |  | Reports how many children the tree node has. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a0e70` | `— ⚠reject` |  | Formats a tree's integer identifiers for display. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00c12480` | `— ⚠reject` |  | Returns the data value (string) stored at a data index of the node, with the engine IString used as the conversion buffer. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

