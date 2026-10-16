# Class: WorldCameraEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b61d0` | `WorldCameraEffectEvent::WorldCameraEffectEvent` | ✅ | ctor | Calls the WorldEffectEvent constructor 008b6130, installs WorldCameraEffectEvent::vftable at *this and at +0x44, zeroes +0x94, stores param_4 into +0x90 and increments param_4's reference count (param_4[2]), calling the… |
| `008b6280` | `WorldCameraEffectEvent::GetField4c` | 🟢 | method | Returns the dword at this+0x4c; a single-load getter with no other side effect. |
| `008b6290` | `WorldCameraEffectEvent::TailCallOwnerVtableSlot4` | 🟢 | virtual | Bytes 8b 89 90 00 00 00 / 8b 01 / 8b 50 10 / ff e2: loads the owner pointer at this+0x90, takes its vtable and tail-jumps to byte offset 0x10 (slot 4) of it with no arguments. |
| `008b62b0` | `WorldCameraEffectEvent::StopAndDeleteIfHandleMissingOrOwnerBit18Clear ? ⚠rename` | 🟡 | virtual | When the pointer at +0x94 is null, or when the byte at that pointer+0x18 has bit 0 clear, calls the virtual in slot 0x30 of *this with (1,1) and then the virtual in slot 0xc with (1). |
| `008b62e0` | `— ⚠reject` | ⚪ | virtual | Calls the virtual in slot 0x30 of *this with (0,1); when +0x94 is null and +0x90 is not, calls 00769710, reads the camera parameter block at owner+0xa0..+0xb0 and hands it to 00769d90 to start a camera effect. |
| `008b63c0` | `WorldCameraEffectEvent::StopCameraEffect` | 🟢 | method | Bytes show: when the handle at +0x94 is non-null, 007d1b00 is called only if param_3 is non-zero, while 007d21d0(param_2) is always called and the handle at +0x94 and the byte at +0x48 are always cleared. |
| `008b6400` | `WorldCameraEffectEvent::AdjustingScalarDeletingDestructorThunk` | 🟢 | dtor | Subtracts 0x44 from ECX, then tail-jumps to 008b64b0; the existing stack deleting-flag argument reaches that target unchanged. |
| `008b6410` | `WorldCameraEffectEvent::~WorldCameraEffectEvent` | ✅ | dtor | Stores WorldCameraEffectEvent::vftable at *this and +0x44; when the byte at +0x48 is set and +0x94 is non-null it calls 007d1b00 and 007d21d0(0), clears +0x94 and that byte, releases the owner reference counted at +0x90… |
| `008b64b0` | `WorldCameraEffectEvent::ScalarDeletingDestructor` | 🟢 | dtor | Calls the camera effect event destructor 008b6410 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |
| `008d1220` | `WorldCameraEffectEvent::Slot8_ComposeMatrix` | 🟢 | virtual | Virtual slot [8] of the camera effect event, taking a pointer to sixteen floats: reads the 4x4 matrix at (*(this+0x90))+0x50..+0x8c, computes the row-major product A x B with the argument matrix, stores the sixteen resu… |
