# Class: VehicleRaceResultsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce28a0` | `VehicleRaceResultsDataSource::ColumnNameByIndex` | ✅ | virtual | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [1] (one stack argument): switch on the index, returning the string Position for 0, Guid for 1, Name for 2, BestLapTime for 3, TotalTime for 4, Finished for 5,… |
| `00ce29b0` | `VehicleRaceResultsDataSource::GetValueByCount ?` | 🟡 | virtual | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [2] (no stack arguments): reads count = dword [[this+0x224]+0x194]; returns 0 when count is below 10, the dword [[this+0x224]+0x2a0] when count is exactly 10, … |
| `00ce3060` | `VehicleRaceResultsDataSource::~VehicleRaceResultsDataSource` | ✅ | dtor | Thiscall destructor body: stores VehicleRaceResultsDataSource::vftable at this, +0x30 and +0x34, stores the Array<dsRaceResults> vtable at +0x258, truncates the array at +0x260 through FUN_00ce2c00, frees the data at +0… |
| `00ce3140` | `VehicleRaceResultsDataSource::VehicleRaceResultsDataSource` | ✅ | ctor | Thiscall constructor with one stack argument (stored at +0x254): calls FUN_00922890 with the BaseClient.VehicleRaceProcessor.RaceResults debug-name pointer, stores VehicleRaceResultsDataSource::vftable at this, +0x30 an… |
| `00ce31d0` | `VehicleRaceResultsDataSource::ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): calls the body 00ce3060, then calls _free(this) when bit 0 of its stack argument is set; returns this in eax. |
| `00ce31f0` | `VehicleRaceResultsDataSource::GetCell` | 🟢 | virtual | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [4] (three stack arguments, row, column and an output buffer): returns 0 unless the row is below the count from vtbl+8 and below the dword at +0x230; then swit… |
| `00ce3330` | `— ⚠reject` | ⚪ | method | Thiscall with the row count argument (or the count from vtbl+8 of the owner when it is zero): for each row it calls FUN_00ce2fd0 when the row is past the end, writes the constant pair DAT_01bf1138/DAT_01bf113c at row+0 … |
