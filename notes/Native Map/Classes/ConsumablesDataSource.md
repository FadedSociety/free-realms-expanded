# Class: ConsumablesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c31d10` | `ConsumablesDataSourceCtor` | ✅ | ctor | Registers BaseClient.Inventory.Consumables with the shared datasource constructor, then installs four ConsumablesDataSource vtables. |
| `00c31de0` | `ConsumablesDataSourceHasRows ⚠downgrade` | 🟢 | method | Checks global player and inventory state, a helper predicate, job value and positive inventory count, then compares this datasource's paging fields to decide whether a row remains. |
| `00c31e40` | `ConsumablesDataSourceDtorAdjustor` | ✅ | method | Subtracts 0x254 from ECX then jumps to 00c31e50, adjusting this for the shared scalar-deleting destructor. |
| `00c31e50` | `ConsumablesDataSourceScalarDeletingDtor` | ✅ | dtor | Runs common datasource cleanup, then frees this object when flag bit 0 is set. |
| `00c31eb0` | `ConsumablesDataSourceGetFieldValue` | ✅ | method | For field 4 formats the datasource paging offset minus its page-size/count fields; other field IDs delegate to FUN_00cb1500. |
