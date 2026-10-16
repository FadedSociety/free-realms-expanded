# Class: ClientLuaInternals

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00976cb0` | `ClientLuaInternalsInitState` | 🟢 | method | Creates the Lua state with FUN_00e39980 and stores it at this+0x94 (returns 0 when that is null). Zeroes this+0x1ac, sets this+0x1b8 = 1000, stores a clock reading (FUN_007027f0) at this+0x1b0/+0x1b4, then calls FUN_00e… |
| `009783c0` | `ClientLuaInternals::RunRegisteredKeyChunk` | 🟢 | virtual | Takes a text argument (param_2, converted with FUN_0083d1e0) and a hashed key (param_4). Returns 1 at once when param_3 is nonzero. For each registered name stored under the key in the hash bucket at this+0xac (the same… |
| `00979120` | `ClientLuaInternals::ClientLuaInternals` | ✅ | ctor | Installs the CommandProcessor::CommandEventHandler vtable then the ClientLuaInternals vtable at this+0, calls FUN_00fc4580(4000) on this+8, stores param_2 at +0x90, runs the embedded-list constructor 00979080, clears +0… |
| `009791c0` | `ClientLuaInternals::DtorBody` | ✅ | dtor | Stores the ClientLuaInternals vtable at this+0, calls FUN_00986c10 with this as its argument, closes the Lua state at +0x94 with FUN_00e3bba0 when it is non-null (then zeroes +0x94), destroys the embedded HashListMap at… |
| `00979260` | `ClientLuaInternals::ScalarDeletingDtor` | ✅ | dtor | Calls the ClientLuaInternals destructor body 009791c0 on this, then frees the object with _free when bit 0 of the flag argument is set; returns this. |
