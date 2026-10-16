# Class: WorldRumbleEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b64d0` | `WorldRumbleEffectEvent::WorldRumbleEffectEvent ⚠downgrade` | 🟢 | ctor | Calls the WorldEffectEvent constructor 008b6130, installs WorldRumbleEffectEvent::vftable at *this and +0x44, stores param_4 into +0x90, increments param_4's reference count and calls the virtual at param_4 vtable+0x24 … |
| `008b6580` | `WorldRumbleEffectEvent::GetField4c` | 🟢 | method | Returns the dword at this+0x4c; identical to 008b6280 in the camera event class. |
| `008b6590` | `WorldRumbleEffectEvent::TailCallOwnerVtableSlot4` | 🟢 | virtual | Loads the pointer at this+0x90, takes its vtable and tail-calls slot 0x10 with no arguments; byte-identical to 008b6290 in the camera event class. |
| `008b65a0` | `— ⚠reject` | ⚪ | virtual | Returns 0 when the pointer at +0x94 is non-null and the short at that pointer+0x66 equals 3, and 1 otherwise. |
| `008b65c0` | `WorldRumbleEffectEvent::StopAndDeleteIfHandleMissingOrOwnerBit18Clear ? ⚠rename` | 🟡 | virtual | When the pointer at +0x94 is null or the byte at owner+0x18 has bit 0 clear, calls virtual slot 0x30 of *this with (1,1) and then slot 0xc with (1); byte-identical to 008b62b0. |
| `008b65f0` | `— ⚠reject` | ⚪ | virtual | Calls virtual slot 0x30 of *this with (0,1); when +0x94 is null and +0x90 is not, calls 00769710, reads the rumble parameter block from the owner at +0xac..+0xb4 plus the byte at owner+0x98, and passes them to 00769e20 … |
| `008b66c0` | `WorldRumbleEffectEvent::StopRumbleEffect` | 🟢 | method | Bytes show: when the handle at +0x94 is non-null, 007d30a0 is called only if param_3 is non-zero, while 007d30b0(param_2) is always called and the handle at +0x94 and the byte at +0x48 are always cleared. |
| `008b6700` | `— ⚠reject` | ⚪ | method | Copies 16 dwords from param_2 into this+0x50, then tail-calls the virtual in slot 4 of the object held at this+0x94 with param_2. |
| `008b6740` | `WorldRumbleEffectEvent::AdjustingScalarDeletingDestructorThunk` | 🟢 | dtor | Subtracts 0x44 from ECX, then tail-jumps to 008b67f0; the existing stack deleting-flag argument reaches that target unchanged. |
| `008b6750` | `WorldRumbleEffectEvent::~WorldRumbleEffectEvent` | ✅ | dtor | Stores WorldRumbleEffectEvent::vftable at *this and +0x44; when the byte at +0x48 is set and +0x94 is non-null it calls 007d30a0 and 007d30b0(0), clears +0x94 and that byte, releases the owner reference at +0x90 and cal… |
| `008b67f0` | `WorldRumbleEffectEvent::ScalarDeletingDestructor` | 🟢 | dtor | Calls the rumble effect event destructor 008b6750 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |
