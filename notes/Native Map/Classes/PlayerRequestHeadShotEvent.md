# Class: PlayerRequestHeadShotEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00959d40` | `PlayerRequestHeadShotEventCtor ?` | 🟡 | ctor | Calls a base-event initializer, installs PlayerRequestHeadShotEvent vtable, and stores constructor arguments at fields 0x48 and 0x4c. |
| `00959dc0` | `PlayerRequestHeadShotEvent::ScalarDeletingDtor ?` | 🟡 | dtor | Calls 00bf6840 and frees this object when deletion flag bit 0 is set. |
| `00bf6840` | `PlayerRequestHeadShotEvent::DtorBody` | 🟢 | dtor | Stores PlayerRequestHeadShotEvent::vftable into the object first, then calls 007e7980. The body has no member cleanup of its own. |
| `00bf68c0` | `— ⚠reject` | ⚪ | virtual | PlayerRequestHeadShotEvent::vftable[1]. When stack arg2 bit 0 is set: reads this+0x48 and +0x4c, calls 00769710 then 00984960 on them; if the result is nonzero and its byte at +0x145 lacks mask 0x40, calls 00769710 agai… |
