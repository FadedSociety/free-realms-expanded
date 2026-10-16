# Class: Fixture

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cba340` | `Fixture::Ctor` | ✅ | ctor | Thiscall with seven stack arguments (param_2..param_8): calls 00cb9f80 on the receiver with param_2..param_7; stores Fixture::vftable at p+0 and p+0x50; installs SoeUtil::IString<char>::vftable at p+0x130, p+0x140 and p… |
| `00cba4d0` | `Fixture::AdjustorThunk50 ?` | 🟡 | virtual | Adjustor thunk at slot 0 of the secondary Fixture vftable (01837374): bytes 83 e9 50 e9 68 0b 00 00 subtract 0x50 from ECX and jump to 00cbb040 (rel32). |
| `00cba4e0` | `Fixture::CtorFromObject` | ✅ | ctor | Thiscall with three stack arguments (param_2..param_4): calls 00cb9dc0 on the receiver (HousePiece constructor), then installs Fixture::vftable at p+0 and p+0x50, the SoeUtil::IString<char> members at p+0x130, p+0x140 a… |
| `00cba720` | `Fixture::DtorBody` | ✅ | dtor | Fastcall on receiver p: stores Fixture::vftable at p+0 and p+0x50; calls slot 4 of the object at p+0x68 with p+0x70 as argument; when that result is nonzero and the object pointer at p+0x170 is set, calls 00774c80 with … |
| `00cbb040` | `Fixture::DeletingDtor` | ✅ | dtor | Virtual slot 6 of the primary Fixture vftable (018373b4), thiscall with one byte flag: calls the Fixture dtor body 00cba720 on the receiver, then when flag bit 0 is set calls _free (thunk 00d82463); returns the receiver… |
| `00cbb060` | `Fixture::Slot1ThenCandidateBuild ?` | 🟡 | virtual | Virtual slot 1 of the secondary Fixture vftable (01837374), thiscall with one stack argument: stores 1 at receiver+0x25; when byte +0x24 is nonzero and bit 1 of the byte at the argument+0x10 is clear, calls 00705200, 00… |
