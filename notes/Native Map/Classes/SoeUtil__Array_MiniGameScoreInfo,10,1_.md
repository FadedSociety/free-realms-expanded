# Class: SoeUtil::Array<MiniGameScoreInfo,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b5260` | `ArrayMiniGameScoreInfoAllocate` | ✅ | virtual | For capacity <=10 returns aligned inline storage and reports capacity 10; otherwise delegates allocation to FUN_009add60. |
| `009b5bb0` | `ArrayMiniGameScoreInfoSmallBuffer::ScalarDeletingDtor` | ✅ | virtual | Calls inline-capacity array cleanup, optionally frees the array object based on flag bit 0, and returns the object pointer otherwise. |
| `00c42810` | `Array_MiniGameScoreInfo_Construct` | ✅ | method | Constructor for the array object: stores the SoeUtil::Array<MiniGameScoreInfo,0,1> vftable, zeroes the buffer/count/capacity dwords at +4/+8/+0xc, then immediately overwrites the vtable slot with SoeUtil::Array<MiniGame… |
