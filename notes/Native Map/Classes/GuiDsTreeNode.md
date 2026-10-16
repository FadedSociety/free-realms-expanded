# Class: GuiDsTreeNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099ef20` | `GuiDsTreeNode::GuiDsTreeNode` | ✅ | ctor | Installs GuiDsTreeNode and child-array vtables, clears the parent pointer, and initializes child-array storage and counters to zero. |
| `0099ef90` | `GetStaticPointer175B400` | 🟢 | virtual | Returns the static pointer at 0x0175B400. |
| `0099efb0` | `GuiDsTreeNode::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_010CDB00 for node cleanup, conditionally frees the object when the deleting flag is set, and returns its pointer. |
