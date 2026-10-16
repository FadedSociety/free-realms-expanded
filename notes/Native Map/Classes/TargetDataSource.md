# Class: TargetDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7a2f0` | `TargetDataSource::Ctor` | ✅ | ctor | Constructor: calls the base constructor 00cb2920 with the BaseClient.Target name, then stores the TargetDataSource vtable at [this], +0x30 and +0x34 and returns this. |
| `00a7a370` | `TargetDataSource::DtorBody` | ✅ | dtor | Destructor body: stores the TargetDataSource vtable at [this], +0x30 and +0x34, then calls base destructor 00cb2810 on this and returns. |
| `00a7a3d0` | `TargetDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (27 bytes, vtable slot 0): calls the destructor body 00a7a370 on this, then if bit 0 of the flag is set calls _free (thunk 00d82463) on this, and returns this. |
| `00a7a3f0` | `TargetDataSource::Slot14_IsNotCurrentTarget` | 🟢 | virtual | Virtual slot 14 of TargetDataSource (vtable 01816b04), 57 bytes: loads the owner pointer at this+0x254, calls the table method 00984960 on the table at owner+0xc0 with the pair at owner+0x920 and owner+0x924, and return… |
