# Class: CoinStoreDataSource::ActiveFilters

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf0a30` | `CoinStoreDataSource::ActiveFilters::FieldName` | ✅ | virtual | Returns FilterId for index 0, OptionList for index 1, and <error> for other indices. |
| `00cf1ee0` | `— ⚠reject` | ⚪ | virtual | Calls the base datasource field accessor, resolves a row from the selected datasource, obtains its map, then either releases the result for field 0 or formats mapped tree keys into the output for field 1. |
| `00cf21a0` | `CoinStoreDataSource::ActiveFilters::Dtor` | ✅ | dtor | Restores the ActiveFilters vtables at the object and embedded-base offsets, destroys its fixed string through 00402140, then destroys its embedded GUI datasource base through 0091bf60. |
| `00cf2230` | `CoinStoreDataSource::ActiveFilters::GetCount` | ✅ | virtual | Returns the 32-bit value at this+0x338. |
| `00cf22e0` | `CoinStoreDataSource::ActiveFilters::Ctor` | ✅ | ctor | Constructs the GUI datasource base, installs ActiveFilters vtables, initializes a fixed string from the embedded class-name string, and clears the count at +0x368. |
| `00cf2370` | `CoinStoreDataSource::ActiveFilters::ScalarDeletingDtor` | ✅ | dtor | Calls the ActiveFilters destructor body, frees the object when the low bit of the flag is set, and returns the object pointer otherwise. |
