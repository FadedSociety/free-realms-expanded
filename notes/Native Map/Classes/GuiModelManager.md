# Class: GuiModelManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090c270` | `GuiModelManager::GuiModelManager` | ✅ | ctor | Installs GuiModelManagerInterface then GuiModelManager vtables, clears the first three dwords and fields +0xb0/+0xb4, and calls 008f9fb0. |
| `0090c2f0` | `GuiModelManager::ScalarDeletingDtor ?` | 🟡 | dtor | Calls 010caaf0 to destroy the manager, frees the object when flag bit 0 is set, and returns its pointer. |
