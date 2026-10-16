# Class: SoeUtil::HashListMap<SoeUtil::StrongType<unsigned___int64,SoeUtilSignatureCharacterGuid,0>,Fixture::EffectStorageNode,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb90d0` | `HashListMap32Ctor` | ✅ | ctor | Fastcall on receiver p: stores the HashListMap vftable at p+0, zeroes p+4, p+8, p+0xc and p+0x10, and zeroes 0x80 bytes (32 dwords) at p+0x14 with memset; returns the receiver. |
| `00cb9220` | `HashListMapNodeAlloc ?` | 🟡 | virtual | Virtual slot 1 of the HashListMap class (no arguments): calls 006fb9c0, then tail-calls slot 1 of the object from 006fb860 when it is nonnull, else slot 1 of DAT_01bdf280 when nonnull, else returns the result of allocat… |
| `00cb9260` | `HashListMapFreeAlign8 ?` | 🟡 | virtual | Virtual slot 2 of the HashListMap class: bytes 8b 44 24 04 6a 08 50 e8 ... 83 c4 08 c2 04 00 decode to call 00401c00 with the argument pointer and alignment 8, then ret 4 with one stack argument. |
| `00cba220` | `HashListMapDtorBody` | ✅ | dtor | Fastcall on receiver p: stores the HashListMap vftable at p+0, then while the head at p+8 is nonzero calls 00cb9870 on the head (drains every node; the head is re-read each pass). |
| `00cba300` | `HashListMapDeletingDtor → SoeUtil::HashListMap<SoeUtil::StrongType<unsigned___i…` | ✅ | dtor | Virtual slot 0 of the HashListMap class (thiscall, one byte flag): stores the vftable at receiver+0, drains the list through 00cb9870 while the head at +8 is nonzero, then when flag bit 0 is set calls _free (thunk 00d82… |
