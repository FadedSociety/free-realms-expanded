# Class: SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureHousingInstanceGuid,0>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbbe50` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the Array<StrongType<__int64,HousingInstanceGuid>> class (thiscall, count, out capacity pointer, flag): same growth policy as ArrayGrowAllocStride98 (flag zero: grow to (5n)/4, keep when capacity is wi… |
| `00cbbef0` | `ArrayFreeBufferAlign8B ?` | 🟡 | virtual | Virtual slot 2 of the Array<StrongType<__int64,HousingInstanceGuid>> class: same 18-byte body as ArrayFreeBufferAlign8 (00cbbdc0): pushes 8 and the first argument, calls 00401c00 (free with alignment 8), and returns wit… |
| `00cbbf10` | `ArrayExchangeHeadersB ?` | 🟡 | virtual | Virtual slot 3 of the Array<StrongType<__int64,HousingInstanceGuid>> class: same body as ArrayExchangeHeaders (00cb7920) and 00cbbde0; flag nonzero calls slot 3 of *param_2, flag zero exchanges the receiver's base, coun… |
| `00cbc4c0` | `ArrayDtorBodyHousingGuid` | ✅ | dtor | Fastcall on receiver p: stores the Array<HousingInstanceGuid> vftable at p+0, zeroes p+8, calls 00401c00 on the buffer at p+4 with alignment 8, and zeroes p+4. |
| `00cbc5b0` | `ArrayDeletingDtorHousingGuid` | ✅ | dtor | Virtual slot 0 of the Array<HousingInstanceGuid> class (thiscall, one byte flag): stores the vftable at receiver+0, zeroes +8, calls 00401c00 on the buffer at +4 with alignment 8, zeroes +4, and when flag bit 0 is set c… |
