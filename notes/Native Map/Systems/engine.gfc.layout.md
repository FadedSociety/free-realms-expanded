# System: engine.gfc.layout

24 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004159a4` | `LoadControlNameAndFixedSize` |  | Read half of the control-name/fixed-size state pair (vtable slot 17 of the box-layout family); its writer counterpart is 00415a37. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415a37` | `StoreControlNameAndFixedSize` |  | Stores the control-name/fixed-size state of a layout/control object into its data node; counterpart of 004159a4. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415c22` | `StoreSpacerState` |  | Saves the spacer configuration of a layout object into its data node; counterpart of 00415b66. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00416099` | `StoreTableSeparatorState` |  | Writes the separator/outer-margin configuration of a table/list layout into its data node (vtable slot 18 of the 0x175EF60 class group), reusing the item store for the child list. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0043d778` | `GCtrl::GetOrCreateAttachedVBox` | GCtrl | Returns the GVBoxLayout attached at this+0xb4 and creates and attaches one when none exists and the byte argument is nonzero. | [[Native Map/Shards/S0806 00406512-00621d93|S0806]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004152c2` | `InstantiateItemClassFromNode` |  | Creates a layout/control item instance from the stored 'ItemClass' blob and binds its 'ItemData' object; the attribute names sit in the layout attribute string cluster with SpacerType/ItemIndex. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004153b9` | `StoreItemClassAndData` |  | Stores the item class/data binding of a layout item into the data node; called from the layout item list code at 00415e55 and from the control rebuild at 00418ce1. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004154e7` | `LoadFlagsAndStretch` |  | Loads the layout flags/stretch block of a control or layout object from its data node, the read half of the state stored by 004155a8; attribute names match the layout attribute cluster (SpacerType/ItemIndex). | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004155a8` | `CopyStateBlockToCreatedObject` |  | Instantiates a new layout/control object (through the FUN_004ccf27/FUN_004cd239 path used by 004152c2) and seeds it with this object's flags/stretch state, mirroring the loader at 004154e7. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004155fe` | `GetCachedPairByIndex` |  | Accessor returning one cached min/max pair (of the four-pair table at +0x2C) selected by an index; used by the arrange code 004156b2 and by control layout callers such as 00617826. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004156b2` | `ArrangeChildRectInParent` |  | Layout placement routine that arranges a child box inside the parent using the alignment flag bits returned by the object's own slot +0x2C; the same routine family is used by the box layouts (GVBoxLayout/GHBoxLayout sha… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0041588c` | `GetSizeValueById` |  | Measurement resolver behind the spacer/fixed-size code: 00415b23 uses it to fill the layout object's size pair from the stored 16-bit id and cached pair. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415f75` | `OffsetRectAndChildren` |  | Recursive layout translation (vtable slot 9 of the box-layout group) used when a container moves; the 0x100 child flag marks children that must be re-arranged instead of simply moved. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00416025` | `GetWords54_58` |  | Accessor exposing the +0x54/+0x58 word pair to callers; the pair is the field written by the control-name/fixed-size loader 00415a37/00415e55, but that link is cross-function so the name stays mechanical. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004165ef` | `StoreStackState ⚠downgrade` |  | Writes the stack/list position of a stacked layout object into its data node (vtable slot 18 of the 0x175EFE8 class group); the ItemIndex it stores is the value 00416315/004161ea use as traversal start. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00416037` | `ResolveSeparatorMetricValue ? ⚠rename` |  | Turns the object's separator id (defaulted when unset) into the concrete metric value used by row/table layout; called from 0041EF78 inside the control code. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00416315` | `MaxOverChildPairs ? ⚠downgrade` |  | Minimum-size computation of the layout (vtable slot 7 of the 0x175EFE8 class group) used before arranging children in 00416463. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0041593b` | `— ⚠reject` |  | Value resolver reached from 00416037 for the effective separator id; that caller discards both the return value and the written argument slot, so no value reaches the layout state at this call site. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415b23` | `— ⚠reject` |  | Exposes the effective fixed size of the layout object to callers such as 0041B6CF, combining the id-resolved value with the stored pair. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415b66` | `— ⚠reject` |  | Read half of the spacer state of the box-layout family (vtable slot 17 of the 0x175EE80/0x175EF60 class group); the writer is 00415c22. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415cfe` | `— ⚠reject` |  | Propagates child stretch requirements up a layout container (vtable slot 15 of the vbox/hbox class group); the same 0x80000/0x800000 bits drive the arrange code 004156b2/00416315. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00415e55` | `— ⚠reject` |  | Writes a stack/box layout's items and their per-item class/data bindings into the data node; called by the two attribute writers 00416099 (table separators) and 004165ef (stack state). | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `004161ea` | `— ⚠reject` |  | Stretch propagation for the stacked variant of the layout (vtable slot 15 of the 0x175EFE8 class group), respecting a start index for stacked display. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00416463` | `— ⚠reject` |  | Main child-arrangement pass of the box layout (vtable slot 8 of the 0x175EFE8 class group); it consumes the minimum pairs from 00416315 and the arranger 004156b2. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |

