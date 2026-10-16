# Class: HouseRatingKeywordsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb4e0` | `HouseRatingKeywordsDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of HouseRatingKeywordsDataSource (thiscall, index argument): 0 returns Id, 1 returns String; any other index returns <error>. |
| `00cbc480` | `HouseRatingKeywordsDataSource::GetValueAfterRebuild ?` | 🟡 | virtual | Virtual slot 2 of HouseRatingKeywordsDataSource (vtable 01837938; thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbbc50 with ECX = receiver-0x30 (the base sub-object); then returns the dword at… |
| `00cbe430` | `HouseRatingKeywordsDataSource::GetValue` | ✅ | virtual | Looks up a rating-keyword row and returns one of its three fields, resolving the middle field through a secondary lookup; reports explicit errors for missing rows or keywords. |
| `00cbee50` | `HouseRatingKeywordsDataSource::Destroy` | ✅ | dtor | Restores HouseRatingKeywordsDataSource vtables at the primary and two embedded bases, then destroys the embedded GUI user-data holder. |
| `00cbf4c0` | `HouseRatingKeywordsDataSource::HouseRatingKeywordsDataSource` | ✅ | ctor | Registers BaseClient.HouseRatingKeywordsList, installs the class's three vtables, stores two constructor arguments, and clears the byte flag at offset 0x25c. |
| `00cbf550` | `HouseRatingKeywordsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the rating-keyword data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
