# Class: FishingDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd4420` | `FishingDataSource::FieldName` | ✅ | virtual | Virtual slot 1 of FishingDataSource (vtable 0183955c): returns the static name for column index 0 to 5: FishType, FishName, FishIcon, FishSpecial, FishCatchable, FishLureRequirement; any other index returns the static s… |
| `00cd45e0` | `FishingDataSource::Value29C ?` | 🟡 | virtual | Virtual slot 2 of FishingDataSource: returns 0 when the pointer at this+0x224 is null; otherwise returns the dword at +0x29c of the object it points to. |
| `00cd4620` | `FishingDataSource::GetCellValue` | ✅ | virtual | Virtual slot 4 of FishingDataSource: returns 0 when the count at +0x29c of the object at +0x224 is below arg2. Otherwise column arg3 (0 to 5) selects a list accessor (00b63070, 00b630a0, 00b630d0, 00b63100, 00b63130 or … |
| `00cd4930` | `FishingDataSource::~FishingDataSource` | ✅ | dtor | Stores FishingDataSource::vftable at this, +0x30 and +0x34, then calls base routine 0091bf60. |
| `00cd4a10` | `FishingDataSource::FishingDataSource` | ✅ | ctor | Calls 00922890 with the name pointer for BaseClient.FishingProcessor.FishTypes, stores FishingDataSource::vftable at this, +0x30 and +0x34, stores arg2 at +0x254, and returns this. |
| `00cd4a90` | `FishingDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls destructor body 00cd4930 on this, then frees this when bit 0 of the flag argument is set; returns this. |
