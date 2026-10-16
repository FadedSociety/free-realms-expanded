# Class: SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureFixtureGuid,0>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbc0d0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the Array<StrongType<__int64,FixtureGuid>> class (thiscall, count, out capacity pointer, flag): same growth policy as ArrayGrowAllocStride8Guid (00cbbe50); returns a new buffer of capacity*8 bytes from… |
| `00cbc170` | `ArrayFreeBufferAlign8C ?` | 🟡 | virtual | Virtual slot 2 of the Array<StrongType<__int64,FixtureGuid>> class: bytes 8b 44 24 04 6a 08 50 e8 ... 83 c4 08 c2 08 00 decode to push 8 and the first argument, call 00401c00 (free with alignment 8), add esp,8, ret 8. E… |
| `00cbc190` | `ArrayExchangeHeadersC ?` | 🟡 | virtual | Virtual slot 3 of the Array<StrongType<__int64,FixtureGuid>> class: same body as ArrayExchangeHeaders (00cb7920): flag nonzero calls slot 3 of *param_2, flag zero exchanges receiver+4, +8 and +0xc with the other header … |
| `00cbc4f0` | `ArrayDtorBodyFixtureGuid` | ✅ | dtor | Fastcall on receiver p: stores the Array<FixtureGuid> vftable at p+0, zeroes p+8, calls 00401c00 on the buffer at p+4 with alignment 8, and zeroes p+4. |
| `00cbc5f0` | `ArrayDeletingDtorFixtureGuid` | ✅ | dtor | Virtual slot 0 of the Array<FixtureGuid> class (thiscall, one byte flag): stores the vftable at receiver+0, zeroes +8, calls 00401c00 on the buffer at +4 with alignment 8, zeroes +4, and when flag bit 0 is set calls _fr… |
