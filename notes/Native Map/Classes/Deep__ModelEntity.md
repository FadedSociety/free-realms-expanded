# Class: Deep::ModelEntity

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007889a0` | `ModelEntity::CopyMatrix16AndNotify ?` | 🟡 | virtual | Copies 16 dwords (64 bytes) from the argument into this+0x160. If this+0xc is non-null, calls its vtable slot +0x18 with the argument. If this+4 is non-null, calls 007351c0. Returns with ret 4. Virtual slot [7] of Deep:… |
| `007889e0` | `ModelEntity::CallThenNotifyMatrix ?` | 🟡 | virtual | Calls 00764c50(argument) with this in ECX, then calls the vtable slot +0x18 of the object at this+0xc with &this[0x160] when that object is non-null, then calls 007351c0 when this+4 is non-null. Returns with ret 4. |
