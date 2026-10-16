# Class: Chess

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c07fa0` | `Chess::Assign` | ✅ | method | Copy assignment (operator=) for the whole Chess engine instance: memcpy(this, source, 0x1290) and returns this. |
| `00c07fc0` | `Chess_DefaultCtorClosure` | ✅ | method | Compiler-generated default-constructor closure: calls Chess::Chess(this, 0) (FUN_00f4eb70) with a null computer side and returns. |
| `00c07ff0` | `Chess_DeletingDtor → Chess::ScalarDeletingDtor` | 🟢 | dtor | Deleting destructor: calls Chess::~Chess (FUN_00f49190) on the object and then releases it with _free when bit 0 of the flag byte argument is set, returning the object pointer. |
| `00c08010` | `Chess_CreateAndStart` | ✅ | function | Destroys any existing Chess instance stored at *param_1 (Chess::~Chess + _free), allocates a fresh 0x1290-byte block (FUN_00d833a6), constructs Chess on it (FUN_00f4eb70), stores the pointer back through param_1 and cal… |
