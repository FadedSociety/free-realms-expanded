# Class: SoeUtil::List<Logging::LoggingApi::QueueEntry,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00896f60` | `NodePoolPopThunk` | ✅ | function | Eight-byte adjustor thunk (raw bytes 83 c1 10 e9 18 ff ff ff): adds 0x10 to this and jumps to NodePoolPopNode FUN_00896e80, so it can serve as the virtual allocation slot of a List object whose pool sub-object starts at… |
| `00896f70` | `NodePoolPushThunk` | ✅ | function | Eight-byte adjustor thunk (raw bytes 83 c1 10 e9 78 ff ff ff): adds 0x10 to this and jumps to NodePoolPushFreeNode FUN_00896ef0, serving as the virtual node-release slot of the owning List. |
| `00896f80` | `SoeUtil::QueueEntryList128Dtor` | ✅ | dtor | Destructor body: installs SoeUtil::List<Logging::LoggingApi::QueueEntry,128>::vftable, calls lock/container teardown FUN_008955c0, frees the block-registry tree rooted at +0x10 (children via FUN_00894920, header via Fre… |
| `00897030` | `QueueEntryList128ScalarDtor` | 🟢 | dtor | Scalar deleting destructor: runs the QueueEntry list destructor FUN_00896f80 and, when bit 0 of param_2 is set, releases the object with _free. |
