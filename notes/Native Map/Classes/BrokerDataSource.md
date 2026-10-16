# Class: BrokerDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cee410` | `BrokerDataSource::FieldName` | ✅ | virtual | Maps indices 0 through 4 to broker GUID, maximum items, markup modifier, listing fee modifier, and listing duration; returns <error> otherwise. |
| `00cefb90` | `— ⚠reject` | ⚪ | virtual | Returns formatted broker GUID, maximum item count, markup modifier, listing-fee modifier, or listing duration for indices 0 through 4; returns zero for other indices. |
| `00cefe40` | `BrokerDataSource::Ctor` | ✅ | ctor | Installs BrokerDataSource vtables, initializes base data fields and strings, constructs SearchItems, MyItems, and PlayerInventoryItems child data sources, and stores supplied owner arguments. |
| `00cf0040` | `BrokerDataSource::Dtor` | ✅ | dtor | Releases the embedded IString buffer, three shared-object members when release mode allows, then destroys the base GUI user-data table subobject. |
| `00cf0150` | `BrokerDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls BrokerDataSource cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
