# System: engine.gfc.controls

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00418ce1` | `StoreControlProperties ⚠downgrade` |  | Writes the persistent control property block used by every GFC control class: it is called from all the control implementations in the 0x6060xxx-0x660xxxx region (buttons, labels, panels, tables), which is what makes it… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00418f4d` | `SetStyleId` |  | Runtime setter of the StyleId field at +0xB0: the confirmed property writer 00418ce1 stores the same field under the literal 'StyleId' with the same DAT_01BC3570 default, which is the anchor for the semantic name. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00418fc6` | `ResolveLinkedObject` |  | Shared linked-object accessor called by many routines across the control region (00419846 and the 004199xx-006776xx callers); the class behind the +0x44 manager is not established. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00419123` | `WalkChildrenRecursive` |  | Recursive walk over the control's children with a clip/damage-region update on the 0x8000000 path (FUN_0040F723 composes the rect, FUN_004E820B applies it); the same GObjectIterator slot +0x50(iter, 3) pattern is used b… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0041925b` | `FindDirectionalNeighbor` |  | Implements directional control navigation (moving focus to the nearest control in a direction); the nine anchor points and the distance minimisation are the classic GFC control-tree behaviour behind arrow-key focus move… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0041950b` | `NotifyIfFlagged0E` |  | Guarded notification emission in the control family (the 0xE kind matches the notification type word used by the style setter 00418f4d), reached from the 00648d61/00845910/010CE100 regions. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004195ff` | `ClearChildField48` |  | Clears a per-child back-link/flag field, part of the control-tree maintenance operations alongside 00419790/004197f3. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004196d8` | `SetTreeObjectRecursive` |  | Recursive attach of the tree-wide object reference at +0xC0; the retain-before-release order is required by the shared interface convention (slot +0xC = AddRef, slot +0x10(0) = Release). | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00419790` | `SetField40Recursive` |  | Recursive assignment of a tree-wide back-pointer/category field over the control hierarchy; the +0x40 field is the same one cleared by 004197f3 and set by the walks called from 00430FF9/00431CA2. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004197f3` | `ClearField40Recursive` |  | Recursive teardown of the +0x40 tree field (the clearing counterpart of 00419790), used when the control hierarchy is being reset. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00419846` | `ScalePositionToTarget` |  | Position/scale mapping between a control and its target rectangle, part of the control pointing/focus maths; it consumes 00418fc6 (the resolved control) and the float constant _DAT_0175db04 (0.0 or 360.0) for the roundi… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00418ffa` | `— ⚠reject` |  | State setter around bit 0x4000000 with manager bookkeeping and change notification; the bit is the one 00418ce1 writes under the CtrlFlags attribute default GFC_CF_NOTVISIBLE, so the visibility reading is a cross-functi… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004191af` | `— ⚠reject` |  | Subtree search helper over the control children (no callers in the export); the return contract is the parent-of-match convention above, so it cannot be described as a plain finder and the earlier focus/hit-code use was… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0041951c` | `— ⚠reject` |  | Mechanically fills the caller's context descriptor and optionally queries the +0x24 object with the DAT_01BC3920 key; zero callers, so the earlier hit-test purpose is not verified. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0041964d` | `— ⚠reject` |  | Size pair accessor used by the layout-side caller 0041B072; the field roles above come from the code only. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |

