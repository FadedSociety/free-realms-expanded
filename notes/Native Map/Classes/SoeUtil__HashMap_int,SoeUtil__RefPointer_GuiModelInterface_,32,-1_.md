# Class: SoeUtil::HashMap<int,SoeUtil::RefPointer<GuiModelInterface>,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a8af30` | `HashMapGuiModelDtorBody` | ✅ | dtor | Stores the HashMap<int,RefPointer<GuiModelInterface>,32,-1> vptr at [ecx] and jumps to 00a8aca0 (HashListClear32Refs) as a tail jump; bytes: mov dword [ecx],01817bc0; jmp 00a8aca0. |
| `00a8b0b0` | `HashMapGuiModelScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of HashMap<int,RefPointer<GuiModelInterface>,32,-1>: stores the HashMap vptr at *this, calls 00a8aca0 (HashListClear32Refs), then calls _free(this) through thunk 00d82463 when bit 0 of param_2… |
| `00a8b8e0` | `HashMapGuiModelDtorFrame` | ✅ | dtor | Destructor body with an MSVC EH frame: stores the HashMap<int,RefPointer<GuiModelInterface>,32,-1> vptr at *param_1, keeps the receiver in a local, and calls 00a8aca0 (HashListClear32Refs); returns. |
