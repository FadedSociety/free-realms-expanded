# Class: AchievementCategoryDataSource::CategoriesTreeNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce5ad0` | `AchievementCategoryDataSource::CategoriesTreeNode::ColumnName ?` | 🟡 | virtual | Maps indices 0 through 6 to Id, Name, Description, Image Set Id, Tint Id, Type, and TypeDataId; returns <error> otherwise. |
| `00ce5b40` | `AchievementCategoryDataSource::CategoriesTreeNode::GetField28 ?` | 🟡 | virtual | Returns the dword stored at object offset +0x28. |
| `00ce5b60` | `AchievementCategoryDataSource::CategoriesTreeNode::GetField30 ?` | 🟡 | virtual | Returns the pointer at object offset +0x30. |
| `00ce6020` | `AchievementCategoryDataSource::CategoriesTreeNode::Ctor ?` | 🟡 | ctor | Constructs GuiDsTreeNode and IconData bases, stores node fields, initializes a fixed string, and loads localized name/description with fallback text. |
| `00ce6140` | `AchievementCategoryDataSource::CategoriesTreeNode::DtorBody ?` | 🟡 | dtor | Destroys embedded objects, restores Loggable vtable at +0x24, and destroys another embedded subobject. |
| `00ce61d0` | `AchievementCategoryDataSource::CategoriesTreeNode::ScalarDeletingDtor ?` | 🟡 | dtor | Calls tree node destructor body, conditionally frees object when flag bit 0 is set, and returns the pointer. |
| `00ce64b0` | `AchievementCategoryDataSource::CategoriesTreeNode::GetColumn ?` | 🟡 | virtual | For indices 0..6 writes Id, Name, Description, image set, tint, type, or type-data into output and returns success; otherwise 0. |
