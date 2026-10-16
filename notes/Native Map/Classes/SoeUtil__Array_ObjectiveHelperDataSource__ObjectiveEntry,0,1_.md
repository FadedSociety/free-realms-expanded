# Class: SoeUtil::Array<ObjectiveHelperDataSource::ObjectiveEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb7860` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the Array<ObjectiveEntry> class (thiscall, arg param_2 = requested count, param_3 = out capacity, param_4 = flag). Flag zero: grows to (5n)/4 when the current capacity is below n, keeps the current buf… |
| `00cb7900` | `ArrayFreeBufferAlign4 ?` | 🟡 | virtual | Virtual slot 2 of the Array<ObjectiveEntry> class: bytes 8b 44 24 04 6a 04 50 e8 ... 83 c4 08 c2 08 00 decode to push 4 and the first stack argument, call 00401c00 (free with alignment 4), add esp,8, ret 8. ECX is not u… |
| `00cb7920` | `ArrayExchangeHeaders ?` | 🟡 | virtual | Virtual slot 3 of the Array<ObjectiveEntry> class (thiscall, param_2 = other array header pointer, param_3 = byte flag). Flag nonzero: indirect call through slot 3 of *param_2 and return. Flag zero: exchanges receiver+4… |
| `00cb80c0` | `ArrayDtorBody` | ✅ | dtor | Fastcall on receiver p: stores Array<ObjectiveEntry> vftable at p+0, calls 00cb7aa0 on the array header with count p+8 (truncates to zero records), calls 00401c00 to free the buffer at p+4 with alignment 4, and sets p+4… |
| `00cb80f0` | `ArrayDeletingDtor → SoeUtil::Array<ObjectiveHelperDataSource::ObjectiveEntry,0,…` | ✅ | dtor | Virtual slot 0 of the Array<ObjectiveEntry> class (thiscall, one byte flag): stores the vftable at receiver+0, truncates the records through 00cb7aa0 (count at +8), frees the buffer at +4 through 00401c00 (alignment 4),… |
