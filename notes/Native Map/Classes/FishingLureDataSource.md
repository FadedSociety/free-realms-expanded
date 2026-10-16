# Class: FishingLureDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd4b50` | `FishingLureDataSource::FishingLureDataSource` | ✅ | ctor | Calls 00cb1bb0 with the name pointer for BaseClient.FishingProcessor.FishingLures, stores FishingLureDataSource::vftable at this, +0x30, +0x34 and +0x254, and returns this. |
| `00cd4c20` | `— ⚠reject` | ⚪ | virtual | Virtual slot 15 of FishingLureDataSource. Returns 0 unless 00b0d810 returns non-null twice; then returns 1 when 00bae6a0 is non-zero, the record from the first call has dword +0x6c equal to 1 and dword +0x8c in (0x10be,… |
| `00cd4c90` | `FishingLureDataSource::AdjustorThunk_Sub254` | ✅ | virtual | Adjustor thunk: subtracts 0x254 from ECX and jumps (jmp, not call) to 00cd4ca0. |
| `00cd4ca0` | `FishingLureDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of FishingLureDataSource: runs base routine 00cb1b10 on this inside an EH frame, then frees this when bit 0 of the flag argument is set; returns this. |
