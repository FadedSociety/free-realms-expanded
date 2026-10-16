# Class: FishingScoringDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd44e0` | `FishingScoringDataSource::FieldName` | ✅ | virtual | Virtual slot 1 of FishingScoringDataSource (vtable 018395d4): returns the static name for column index 0 to 6: FishName, FishIconId, FishDifficulty, FishRarity, FishSize, FishScore, TimeToCatch; any other index returns … |
| `00cd4600` | `FishingScoringDataSource::Value220 ?` | 🟡 | virtual | Virtual slot 2 of FishingScoringDataSource: returns 0 when the pointer at this+0x224 is null; otherwise returns the dword at +0x220 of the object it points to. |
| `00cd47b0` | `FishingScoringDataSource::GetCellValue` | ✅ | virtual | Virtual slot 4 of FishingScoringDataSource: returns 0 unless arg2 is below the count at +0x220 of the object at +0x224. It walks the list headed at +0x218 (next link at node+0x1c) to row arg2; column arg3 0 writes the n… |
| `00cd49a0` | `FishingScoringDataSource::~FishingScoringDataSource` | ✅ | dtor | Stores FishingScoringDataSource::vftable at this, +0x30 and +0x34, then calls base routine 0091bf60. |
| `00cd4ab0` | `FishingScoringDataSource::FishingScoringDataSource` | ✅ | ctor | Calls 00922890 with the name pointer for BaseClient.FishingProcessor.FishScored, stores FishingScoringDataSource::vftable at this, +0x30 and +0x34, stores arg2 at +0x254, and returns this. |
| `00cd4b30` | `FishingScoringDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls destructor body 00cd49a0 on this, then frees this when bit 0 of the flag argument is set; returns this. |
