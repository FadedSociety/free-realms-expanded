# Class: InventoryNavigationDataSource::ItemGroupNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7a530` | `InventoryNavigationDataSource::ItemGroupNode::Slot1_ReturnThis` | ✅ | virtual | Three bytes (8b c1 c3): moves ECX to EAX and returns, so it returns its this pointer unchanged as the interface pointer. |
| `00a7ba90` | `ItemGroupNode::MatchesFilter` | ✅ | virtual | Looks up this node's id at +0x10 in the manager table (param_2+0x20c), using id%10 and the bucket chain; returns false if absent. It starts with a true result and calls 00bae910 when the found record's byte +0x3c is cle… |
| `00a7c310` | `ItemGroupNode::GetColumnValue ⚠downgrade` | 🟢 | virtual | Column value getter of an item-group row node. Looks the node id at this+0x10 up in the manager's table at param_2+0x20c (bucket chain at +0x20 + (id % 10)*4) and calls FUN_00bae910 when the item byte at +0x3c is clear.… |
