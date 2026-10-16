# Class: ClientInGamePurchaseStoreManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c39c80` | `— ⚠reject` | ⚪ | method | Initialises the in-game-purchase store manager: stores the data path at this+0x18, initialises the marketing-data updater (007e1270), builds the schema path from "MarketingDataClientSchema.xml" via IStringAppendFormat, … |
| `00c41270` | `ClientInGamePurchaseStoreManager::OnDataUpdate` | ✅ | method | Logs reload start, clears old portal-category and billboard-panel entries, reloads both definition sets from the supplied source, runs post-load refreshes, and logs elapsed milliseconds. |
| `00c41fc0` | `ClientInGamePurchaseStoreManagerCtor` | ✅ | ctor | Installs MarketingDataHandler then store-manager vtables, initializes logging and all owned store definition maps, billboard maps, and item/bundle-of-the-day objects. |
| `00c42100` | `ClientInGamePurchaseStoreManagerDtor` | ✅ | dtor | Destroys billboard and store maps, item/bundle-of-the-day objects, the store-ID set map and logging state, then restores the MarketingDataHandler base vtable. |
| `00c42240` | `ClientInGamePurchaseStoreManager::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the manager destructor and frees the object when flag bit 0 is set. |
