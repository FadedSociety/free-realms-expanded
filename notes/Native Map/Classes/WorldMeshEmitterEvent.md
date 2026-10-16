# Class: WorldMeshEmitterEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b6810` | `WorldMeshEmitterEvent::WorldMeshEmitterEvent ⚠downgrade` | 🟢 | ctor | Calls the WorldEffectEvent constructor 008b6130, installs WorldMeshEmitterEvent::vftable at *this and +0x44, stores param_4 into +0x90, zeroes +0x94, increments param_4's reference count and calls the virtual at param_4… |
| `008b68b0` | `WorldMeshEmitterEvent::GetField4c → WorldMeshEmitterEvent::EffectEventInterface…` | 🟢 | method | Returns the dword at this+0x4c; the third copy of this getter in the shard (camera 008b6280, rumble 008b6580). |
| `008b68c0` | `WorldMeshEmitterEvent::TailCallOwnerVtableSlot4` | 🟢 | virtual | Loads the pointer at this+0x90, takes its vtable and tail-calls slot 0x10 with no arguments; the third copy of this thunk. |
| `008b68e0` | `WorldMeshEmitterEvent::StopAndDeleteIfHandleMissingOrOwnerBit10Clear → WorldMes…` | 🟢 | virtual | When the pointer at +0x94 is null, or the byte at owner+0x10 has bit 0 clear, calls virtual slot 0x30 with (1,1) and then slot 0xc with 1; slot 0xc holds 008b6a00, the class deleting destructor, so with that flag the ob… |
| `008b6910` | `WorldMeshEmitterEvent::StopMeshEmitterEffect` | 🟢 | method | When the handle at +0x94 is non-null it calls 007ab880 only if param_3 is non-zero, always calls 007ac200(param_2), then clears +0x94 and the byte at +0x48. |
| `008b6950` | `WorldMeshEmitterEvent::AdjustingScalarDeletingDestructorThunk` | 🟢 | dtor | Subtracts 0x44 from ECX, then tail-jumps to 008b6a00; the existing stack deleting-flag argument reaches that target unchanged. |
| `008b6960` | `WorldMeshEmitterEvent::~WorldMeshEmitterEvent` | ✅ | dtor | Stores WorldMeshEmitterEvent::vftable at *this and +0x44; when the byte at +0x48 is set and +0x94 is non-null it calls 007ab880 and 007ac200(0), clears +0x94 and that byte, releases the owner reference at +0x90 and call… |
| `008b6a00` | `WorldMeshEmitterEvent::ScalarDeletingDestructor` | 🟢 | dtor | Calls the mesh-emitter event destructor 008b6960 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |
| `008d1370` | `WorldMeshEmitterEvent::Slot8_ComposeMatrix` | 🟢 | virtual | Virtual slot [8] of the mesh emitter event, taking a pointer to sixteen floats: reads the 4x4 matrix at (*(this+0x90))+0x50..+0x8c, computes the row-major product A x B with the argument matrix, stores the sixteen resul… |
| `008eb490` | `WorldMeshEmitterEvent::UpdateEmitter ⚠downgrade` | 🟢 | method | If emitter is empty but source exists, derives parameters, creates and stores emitter, notifies event, and sets completion flag. |
