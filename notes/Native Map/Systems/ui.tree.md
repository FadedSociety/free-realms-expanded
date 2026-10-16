# System: ui.tree

19 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c126a0` | `PushTreeNodeUserdata` |  | Wraps a GUI tree-control node for the Tree module's Lua bindings. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12720` | `CheckTreeNodeArg` |  | Argument accessor for the Tree module's TreeNode handles. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c128c0` | `RegisterTreeNodeLuaModule` |  | Creates the TreeNode metatable consumed by the Tree module bindings; called from the Tree registration 00c12c60. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c129d0` | `CheckTreeArg` |  | Argument accessor for the Tree (tree-control) module methods. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12c60` | `RegisterTreeLuaModule` |  | Creates the Tree module tables for the GUI tree control; called from the startup registration dispatcher 00976070. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12d40` | `CastToGuiTreeCtrl` |  | Helper that turns a generic GUI object handle into a GuiTreeCtrl pointer for the Tree module functions. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008362c0` | `GuiTreePrivateCtrl::OnPointerDownUpMove` | GuiTreePrivateCtrl | Tracks a pointer drag for the tree control: a press starts the drag flag, a release ends it, and a move while dragging reports the offset. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `00c12a30` | `TreeGetNodeForRow` |  | Maps a visible row in the tree control to its node handle for scripts. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12b10` | `TreeGetSelectedNode` |  | Returns the currently selected node of the tree control to scripts. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12b80` | `TreeGetSelectedRow` |  | Returns the index of the selected row in the tree control. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12bb0` | `TreeGc` |  | Releases the tree-control handle when the userdata is collected. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0067b32f` | `GTreeCtrl::ParentWalkThenVirtual11c ?` | GTreeCtrl | Thiscall with (char param_2, int param_3) and ret 0xc. When param_2 is 0 it walks p = param_3, p = 00406665(p) while p is nonzero, returning early if p equals this. Then, if [this+0x1c4] is nonzero, it calls that object… | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `00835c20` | `GuiTreePrivateCtrl::MatchShortAgainstGlobals ?` | GuiTreePrivateCtrl | Thiscall with one short argument, ret 0x1c. Returns AL=1 when param_2 equals the .data word DAT_01bc7488. Otherwise, when it equals DAT_01bc7244, it stores 0 at [this+0x1d8] and returns 1. Otherwise returns 0. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `008363a0` | `GuiTreePrivateCtrl::ClassifyMessage158 ?` | GuiTreePrivateCtrl | Thiscall with eight stack arguments, ret 0x20. For messages DAT_01bc7308, DAT_01bc713c and DAT_01bc7488 it sets *param_9=1 when param_9 is non-null, and sets *param_8=3 when [this+0x60] has bit 0x800. For DAT_01bc7488 i… | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0069a47e` | `—` | GTreeCtrl | Virtual slot [159] shared by GuiTreePrivateCtrl and GTreeCtrl (both vtable slots hold 0069a47e). Body (18012 bytes of pseudocode) was not read in this pass. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `006a5ebb` | `— ⚠reject` | GTreeCtrl | Thiscall with one stack argument, ret 4. Pushes the argument, then [this+0xf4], then [this+0xf0], calls 006a2b97 with no stack cleanup of its own, then returns with ret 4. EAX is not set. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `00c12820` | `— ⚠reject` |  | Returns the column name for a data index of a tree-control node. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12ab0` | `— ⚠reject` |  | Selects a node in the tree control from a script and reports whether the selection succeeded. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c12f00` | `— ⚠reject` |  | Returns the string data at an index of a tree-control node. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

