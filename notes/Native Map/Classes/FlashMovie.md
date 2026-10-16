# Class: FlashMovie

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d55700` | `FlashMovie::IncrementSharedRefcount ?` | 🟡 | virtual | Loads the pointer at this+0x1c, then atomically increments the 32-bit counter at (pointer+4) with LOCK XADD; returns the pointer in eax and the previous counter value in edx. |
| `00d55720` | `FlashMovie::SetFloat4C ?` | 🟡 | virtual | Stores the float argument (from [esp+4]) into this+0x4c with movss and returns with ret 4. |
| `00d56720` | `FlashMovie::Dtor ⚠downgrade` | 🟢 | dtor | Restores FlashMovie::vftable in *param_1 and its secondary subobject, drops three LOCK-decremented shared references at +0x30, +0x2c and +0x28 (releasing each object at zero via vtable slot 1), calls GFxWrap::ReleaseMov… |
| `00d56810` | `AdjustorThunk_sub14_00d56810` | ✅ | virtual | Adjustor thunk: subtracts 0x14 from ecx (this) (bytes 83 e9 14) and jumps (e9) to 00d56b40, the FlashMovie scalar deleting destructor. |
| `00d56890` | `— ⚠reject` | ⚪ | ctor | Installs FlashMovieInterface::vftable and FlashMovie::vftable, clears the movie fields, creates a Scaleform movie for the file name argument via GFxWrap::CreateMovie, takes a shared reference on a companion object at +0… |
| `00d56b40` | `FlashMovie::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls FlashMovie::Dtor (00d56720) on this, then calls _free(this) when bit 0 of the flags argument is set. |
