# Class: BossesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a79c80` | `BossesDataSource::Slot1_ColumnName` | ✅ | virtual | Virtual slot 1 of BossesDataSource (vtable 01816a3c): switch on the index argument returning a column name string: 0 Guid, 1 Name, 2 CurHealth, 3 MaxHealth, 4 CurMana, 5 MaxMana, 6 Type, 7 Disposition; any other index r… |
| `00a79de0` | `BossesDataSource::Slot2_CountMatching` | 🟢 | virtual | Virtual slot 2 of BossesDataSource (vtable 01816a3c): walks the list from 00984350 with 009849e0, looks up each node's key pair with 00984960, and counts the records that are non-null and pass a flag test: records with … |
| `00a79eb0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 4 of BossesDataSource (vtable 01816a3c): for column argument 2 below 8 it walks the list from 00984350 and counts the visible records (same flag test as slot 2); when the count reaches row argument 1 it wri… |
| `00a7a1d0` | `BossesDataSource::DtorBody` | ✅ | dtor | Destructor body: stores the BossesDataSource vtable at [this], +0x30 and +0x34 with the EH frame set up, then calls the base destructor 0091bf60 (Dtor_GuiUserDataTableAt34) and returns. |
| `00a7a250` | `BossesDataSource::Ctor` | ✅ | ctor | Constructor: calls the base constructor 00922890 with the BaseClient.Bosses name, stores the BossesDataSource vtable at [this], +0x30 and +0x34, stores its argument at +0x254 (the owner pointer) and returns this. |
| `00a7a2d0` | `BossesDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (27 bytes, vtable slot 0): calls the destructor body 00a7a1d0 on this, then if bit 0 of the flag is set calls _free (thunk 00d82463) on this, and returns this. |
