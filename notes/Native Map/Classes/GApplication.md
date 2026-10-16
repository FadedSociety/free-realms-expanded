# Class: GApplication

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00405d3d` | `GApplication::InitializeControlManager` | ✅ | method | Part of GApplication::Initialize: when the byte at +0x1A0 is clear it initializes the control manager at +0x1A8 through FUN_005180a5; on failure it reports the wide warning about the applications message source list thr… |
| `00405ddc` | `GApplication::ShutdownControlManager` | 🟢 | method | When the flag byte at +0x1A0 is set, calls FUN_00518119 on the control manager at +0x1A8 and removes it through virtual slot +0xAC. |
| `0043baad` | `GApplication::CleanupControlManagerState` | 🟢 | method | When field +0x174 is negative, tears down the control manager's entry list through 0043a965, calls 00405ddc on this, and clears byte +0x1a0; otherwise it returns without changes. |
| `007055e0` | `GApplicationDefaultConstructorClosure ?` | 🟡 | function | Invokes the GApplication default-construction routine with a zero initialization argument. |
