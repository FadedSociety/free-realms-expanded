# Class: SoundControl3d

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007bd670` | `SoundControl3d::Read4Dwords_0x50` | ✅ | virtual | Virtual slot [3]: copies four dwords from this+0x50..0x5c to the 16-byte buffer at the stack argument, then ret 4 (bytes 8b 51 50 ... c2 04 00). |
| `007bd690` | `SoundControl3d::Write4Dwords_0x50` | ✅ | virtual | Virtual slot [4]: stores four dwords from the stack argument into this+0x50..0x5c; ret 4 (bytes 8b 44 24 04 ... c2 04 00). |
| `007bd6b0` | `SoundControl3d::Read4DwordsPair_0x60_0x70` | ✅ | virtual | Virtual slot [5]: copies this+0x60..0x6c to the first stack buffer and this+0x70..0x7c to the second; ret 8 (bytes 8b 51 60 ... c2 08 00). |
| `007bd6f0` | `SoundControl3d::Write4DwordsPair_0x60_0x70` | ✅ | virtual | Virtual slot [7]: stores the first stack buffer into this+0x60..0x6c and the second into this+0x70..0x7c; ret 8 (bytes 8b 44 24 04 ... c2 08 00). |
| `007be070` | `SoundControl3d::SetCosSinFromAngle` | ✅ | virtual | Virtual slot [6] (thiscall, one float stack argument, ret 4): builds the vector (cos(angle), 0, sin(angle)) on the stack, calls 00769710, then calls this->vtable[+0x1c] (slot 7, 007bd6f0 in SoundControl3d) with (&vector… |
| `007be0c0` | `SoundControl3d::Diff4Floats_0x50` | ✅ | virtual | Virtual slot [9] (thiscall, two stack args, ret 8): computes the four-float difference (this+0x50..0x5c) minus the first argument vector (movups/subps) and stores it into the second argument buffer. |
| `007bea60` | `SoundControl3d::SumSqDiff4_0x50` | ✅ | virtual | Virtual slot [8] (thiscall, one stack arg (pointer to four floats), ret 4): returns in st0 the sum of squares of the four component differences between the argument vector and the block at this+0x50..0x5c. |
