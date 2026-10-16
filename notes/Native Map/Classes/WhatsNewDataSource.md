# Class: WhatsNewDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e1b0` | `WhatsNewDataSource_FieldName ⚠downgrade` | 🟢 | method | Field-name lookup of the what's-new data source: a switch over the field index returning "Id" (0), "Priority" (1), "IconId" (2), "TitleStringId" (3), "BodyStringId" (4), "ButtonStringId" (5), "LuaCall" (6), "Param1".."P… |
| `00a3e670` | `WhatsNewDataSource_Rebuild` | 🟢 | method | Rebuild of the what's-new rows: calls virtual slot +0xc of the source object held at this+0x34, then loops the index 0..count-1 (count read from the array object at this+0x258 field +8) calling that object's virtual slo… |
| `00a3e960` | `WhatsNewDataSource_RowCount` | ✅ | method | Row-count getter: when the flag byte at +0x22c is clear it first calls the rebuild routine 00a3e670, then returns the element count field (+8) of the array object referenced at +0x228. |
| `00a3f940` | `WhatsNewDataSource::GetFieldValue` | ✅ | virtual | Virtual slot 4 of WhatsNewDataSource reached through the +0x30 subobject vtable: resolves a row index by a virtual call on the object at this+4, indexes the announcement array with *row * 0x54 + *(*(this+0x228)+4) and s… |
| `00a406c0` | `WhatsNewDataSource::~WhatsNewDataSource` | ✅ | dtor | Destructor body of the announcement data source: installs WhatsNewDataSource::vftable at +0 and at the two secondary subobjects +0x30 and +0x34, then calls the table base destructor FUN_0091bf60 with this as its argumen… |
| `00a407c0` | `WhatsNewDataSource::WhatsNewDataSource` | ✅ | ctor | Constructor: runs the table base constructor FUN_00922890 with the registry-name string, installs WhatsNewDataSource::vftable at +0/+0x30/+0x34, stores the owner pointer at +0x254 and the announcement array address at +… |
| `00a40850` | `WhatsNewDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of WhatsNewDataSource: calls the destructor body 00a406c0 and frees the 0x260-byte object with _free when bit 0 of the delete flag is set. |
