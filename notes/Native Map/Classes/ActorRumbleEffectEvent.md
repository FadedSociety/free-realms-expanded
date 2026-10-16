# Class: ActorRumbleEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c5540` | `ActorRumbleEffectEvent::ActorRumbleEffectEvent` | ✅ | ctor | Calls FUN_007dc7b0, stores ActorRumbleEffectEvent::vftable at this+0 and this+0x4c, stores the stack argument at this+0xa0, increments the refcount at argument+8 (old value kept), and if the old value was 0 calls the ar… |
| `008c55f0` | `ActorRumbleEffectEvent::~ActorRumbleEffectEvent` | ✅ | dtor | Stores ActorRumbleEffectEvent::vftable at this+0 and this+0x4c. If the owner at this+0xa0 has a positive refcount at owner+8, decrements it. Calls FUN_007dbab0 with this as ECX receiver to run the ActorEffectEvent base … |
| `008c5660` | `ActorRumbleEffectEvent::VSlot9_IsTargetKeyMinusOne ? ⚠rename` | 🟡 | virtual | Bytes: mov eax,[ecx+0xa0]; xor ecx,ecx; cmp dword [eax+0x1c],-1; sete cl; mov al,cl; ret. Returns 1 when the object at this+0xa0 has field +0x1c equal to -1, else 0. |
| `008c5680` | `ActorRumbleEffectEvent::VSlot1_GetField54` | ✅ | virtual | Bytes: mov eax,[ecx+0x54]; ret. Returns the dword at this+0x54 (secondary-base field). |
| `008c5690` | `ActorRumbleEffectEvent::ScalarDeletingDtor` | ✅ | dtor | Calls destructor 008c55f0 on this, then frees with _free when bit 0 of the stack flag byte is set. Returns this (ret 4). |
| `008c56b0` | `ActorRumbleEffectEvent::ScalarDeletingDtorThunk` | ✅ | dtor | Bytes: sub ecx,0x4c; jmp 008c5690. Adjusts this by -0x4c and jumps to ScalarDeletingDtor 008c5690. |
| `008d1c20` | `ActorRumbleEffectEvent::Slot10_CreateFromActor` | 🟢 | virtual | Virtual slot [10] of the actor rumble effect event: resolves an actor through FUN_007e8270 and, only when that object exists and the byte at +0x22d is set, allocates memory through FUN_00d833a6, builds a value via FUN_0… |
