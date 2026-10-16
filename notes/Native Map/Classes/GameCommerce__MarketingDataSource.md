# Class: GameCommerce::MarketingDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d639e0` | `MarketingDataSource::ValidateRecords ?` | 🟡 | virtual | Returns false when the count at +0x18 is below one; otherwise walks the list at +0x10 and returns false on the first failed row check. |
| `00d63a10` | `— ⚠reject` | ⚪ | virtual | Walks the list beginning at this[4], saves each next pointer from node+0x98, then calls virtual slot +0x10 on that node. |
| `00d66330` | `MarketingDataSource::BuildRecordSet ?` | 🟡 | virtual | Creates a record-set instance, walks field descriptors, constructs typed element objects for types 1, 2, and 3, appends them to its array, and returns zero. |
| `00d665b0` | `MarketingDataSource::Ctor ? → GameCommerce::MarketingDataSource::Ctor (also con…` | 🟡 | ctor | Installs the marketing source vtable, initializes the record-set hash/list map and count fields, zeroes 0x80 bytes of bucket data, and resets two helper values. Returns param_1. |
| `00d66610` | `MarketingDataSource::Dtor ? → GameCommerce::MarketingDataSource::Dtor (body)` | 🟡 | dtor | Restores the source vtable, invokes virtual cleanup for linked records, recursively frees three record trees, and releases all hash-list records. |
| `00d66900` | `MarketingDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Runs MarketingDataSource cleanup and frees the object when the deleting flag is set. |
