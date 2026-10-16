# Class: GameCommerce::MarketingDataElementInstance<SoeUtil::StringFixed<32>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63a80` | `— ⚠reject` | ⚪ | dtor | Releases the string-array storage at this[3], then calls the object's virtual deallocator with size 8. |
| `00d65160` | `MarketingDataElementInstanceString32::Ctor ?` | 🟡 | ctor | Installs the string element-instance vtable, initializes its inline string array and zero counts, then requests capacity eight. |
| `00d65200` | `MarketingDataElementInstanceString32::Dtor ?` | 🟡 | dtor | Runs inline string-array cleanup, then restores the common marketing-element-instance base vtable. |
