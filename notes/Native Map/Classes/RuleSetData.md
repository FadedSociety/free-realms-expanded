# Class: RuleSetData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `013c5920` | `RuleSetData::RuleSetData` | ✅ | ctor | Stores PersistentBase::vftable, then PersistentComponent::vftable, then RuleSetData::vftable into param_1[0] and zeroes param_1[1] (ruleSetID) and param_1[2] (expression-tree pointer). |
| `013c59b0` | `RuleSetData::GetTypeName` | ✅ | method | Returns the constant string "RuleSetData". |
| `013c59f0` | `RuleSetData::~RuleSetData` | ✅ | dtor | Installs RuleSetData::vftable, then if param_1[2] (expression tree, +8) is non-null calls the deleting destructor through its vtable first dword ((*(code *)**(undefined4 **)param_1[2])(1)) and nulls the field; finally r… |
| `013c5a90` | `RuleSetData::FormatDebug` | ✅ | method | Appends the literal "(RuleSetData" to the stream param_3 via FUN_01385610, calls the virtual method at slot +0x38 of the object at +8 with (param_2, param_3), then appends ") => %s" with one of two global strings chosen… |
| `013c5ae0` | `RuleSetData::FormatDebugTyped` | ✅ | method | Calls FUN_013815f0 (stream/indent helper), appends the literal "(gvt:RuleSetData" to param_3, calls the virtual method at slot +0x3c of the object at +8 with (param_2, param_3), then appends ") => %s" with the true/fals… |
| `013c5b70` | `RuleSetData::Dump` | ✅ | method | Calling virtual slot +8 with the stream param_2, this writes "Starting RuleSetData\n", "RuleSetID:", the ID via FUN_013c23f0, "Expression Tree:\n", then if the +8 tree pointer is non-null and FUN_013c23f0(0) returns non… |
| `013c5c80` | `RuleSetData::~RuleSetData` | 🟢 | dtor | Calls FUN_013c59f0 (RuleSetData destructor body) and, when flag bit 0 of param_2 is set, frees the object and returns it. |
| `013c5ca0` | `RuleSetData::Matches ? ⚠rename` | 🟡 | method | Short-circuit conjunction: calls virtual slot +0x10 with param_2, then FUN_013c24d0(param_1+1), then FUN_012faed0(param_2, param_1+2), then virtual slot +0x14 with param_2; returns success only if all four checks succee… |
