# Class: SoeUtil::HashListMap<int,GameCommerce::MarketingDataRecordSetInstance,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d66440` | `HashListMap::Dtor` | ✅ | dtor | Installs the hash-list map vtable and repeatedly releases its current head node through 00d65b50 until the head is empty. |
| `00d66470` | `HashListMap::ScalarDeletingDtor → SoeUtil::HashListMap<int,GameCommerce::Market…` | ✅ | dtor | Installs the hash-list map vtable, drains head nodes through 00d65b50, returns the original pointer when flag bit 0 is clear, and frees the map when it is set. |
