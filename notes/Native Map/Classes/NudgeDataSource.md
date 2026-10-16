# Class: NudgeDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a41490` | `NudgeDataSource::GetFieldName` | ✅ | virtual | Virtual slot 1 of NudgeDataSource: maps a field id 0..0x14 to a literal field name and returns it, covering Id, ProgramId, GroupId, ContentId, StartDate, EndDate, DelaySeconds, IconId, TitleStringId, BodyStringId, Butto… |
| `00a41780` | `NudgeDataSource::GetField1C` | ✅ | virtual | Virtual slot 2 of NudgeDataSource returns the dword at *(*(*(param_1+0x224)+0x3e0)+0x1c). Its vtable is installed at object+0x30, so param_1+0x224 addresses object+0x254; the value at +0x3e0 is then dereferenced and the… |
| `00a419e0` | `NudgeDataSource::GetFieldValue` | ✅ | virtual | Virtual slot 4 of NudgeDataSource: walks the nudge record list at *(*(this+0x224)+0x3e0)+0x14 (next at +0xd8) to row param_2, then writes field id param_3 into the output param_4: ids 0..3 and 6..0x14 as dwords from +0x… |
| `00a41ee0` | `NudgeDataSource::~NudgeDataSource` | ✅ | dtor | Destructor body of the nudge data source: installs NudgeDataSource::vftable at +0/+0x30/+0x34 and calls the table base destructor FUN_0091bf60 with this. |
| `00a41f60` | `NudgeDataSource::NudgeDataSource` | ✅ | ctor | Constructor of the nudge data source: runs the table base constructor FUN_00922890 with the registry name that the caller supplied through PTR_s_BaseClient.Nudges (the string address is the same global used at the call … |
| `00a41fe0` | `NudgeDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of NudgeDataSource: calls the destructor body 00a41ee0 and frees the 0x258-byte object with _free when bit 0 of the delete flag is set. |
