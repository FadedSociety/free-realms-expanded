# Class: SoeUtil::Array<SocialShareDataSource::SocialShareData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a71580` | `SocialShareDataArray::ReserveBuffer` | ✅ | virtual | Virtual slot [1]. Reads the current capacity at this+0xc. With flag 0 it grows to 5/4 of a larger request, or shrinks to 6/5 when capacity exceeds 4/3 of the request, otherwise keeps the current buffer; with flag nonzer… |
| `00a71620` | `SocialShareDataArray::FreeBuffer` | ✅ | virtual | Virtual slot [2]. Reads its first stack argument and calls FreeMemory(pointer, 4); the second stack argument is ignored (ret 8). |
| `00a71640` | `SocialShareDataArray::SwapOrDelegate ?` | 🟡 | virtual | Virtual slot [3]. If the byte argument is nonzero it calls vtable slot +0xc of the object pointed to by the first stack argument and returns. Otherwise it exchanges this+4 (buffer) with the other object's +4, and XOR-sw… |
| `00a71af0` | `SocialShareDataArray::DtorBody` | ✅ | dtor | Stores the Array vtable at this, truncates all elements through 00a718a0 with the current count, frees the buffer at +4 through 00401c00 (FreeMemory, alignment 4) and sets +4 to 0. Does not free this. |
| `00a71bb0` | `SocialShareDataArray::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Stores the Array vtable, truncates all elements through 00a718a0, frees the buffer through 00401c00 (alignment 4), clears +4, then calls _free on this when bit 0 of its stack argument is set; returns t… |
