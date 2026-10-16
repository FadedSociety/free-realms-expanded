# Class: NpcsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a79b50` | `NpcsDataSource::Ctor` | ✅ | ctor | Constructor: calls 00cb2920 with param_2 and the BaseClient.Npcs name, then stores the NpcsDataSource vtable at [this], +0x30 and +0x34 and returns this. |
| `00a79bf0` | `NpcsDataSource::DtorBody` | ✅ | dtor | Destructor body: stores the NpcsDataSource vtable at [this], +0x30 and +0x34 with the EH frame set up, then calls base destructor 00cb2810 on this and returns. |
| `00a79c50` | `NpcsDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (27 bytes, vtable slot 0): calls the destructor body 00a79bf0 on this, then if bit 0 of the flag is set calls _free (thunk 00d82463) on this, and returns this. |
| `00cb1ce0` | `DataSourceProfileColumnName` | ✅ | virtual | Maps numeric columns to the literal names Guid, Name, Description, profile fields, current and maximum resources, boss state, disposition, and friend state; returns an error label for unsupported indices. |
| `00cb1de0` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
| `00cb1e60` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
| `00cb1e80` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
| `00cb1ea0` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
| `00cb1ec0` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
| `00cb1ee0` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
| `00cb1f00` | `DataSourceRefreshAndClearFlag ? ⚠rename` | 🟡 | virtual | Clears the byte at this+0x258, then invokes the virtual method at the object's vtable offset 0x10. |
