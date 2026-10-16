# Class: SoeUtil::HashListMap<int,GodBeamDefinition,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009fefa0` | `AdjustThisPlus0x34AndJump009feec0` | ✅ | method | 8-byte adjustor thunk: adds 0x34 to ECX and jumps to 009feec0. |
| `009fefb0` | `AdjustThisPlus0x34AndJump009fef30` | ✅ | method | 8-byte adjustor thunk: adds 0x34 to ECX and jumps to 009fef30. |
| `009fefc0` | `HashListMap_GodBeamDefinition_DeletingDestruct ? ⚠downgrade` | 🟡 | dtor | Scalar deleting destructor: calls the container destructor body 009fe520 (which re-installs the HashListMap<int,GodBeamDefinition,8,1> vtable, drains the ordered list and frees the hash index) and then _free(this) when … |
