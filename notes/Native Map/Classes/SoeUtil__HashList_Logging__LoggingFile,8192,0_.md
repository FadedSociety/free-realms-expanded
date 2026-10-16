# Class: SoeUtil::HashList<Logging::LoggingFile,8192,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089a590` | `HashListLoggingFile8192AllocateNode` | ✅ | method | Adjustor thunk (bytes 81 c1 14 80 00 00 e9 15 ff ff ff): adds 0x8014 to this and jumps to NodePool8e0Pop FUN_0089a4b0, serving as the virtual node-allocator slot of the 8192-bucket map whose pool sub-object starts at +0… |
| `0089a5a0` | `HashListLoggingFile8192FreeNode` | ✅ | method | Adjustor thunk (bytes 81 c1 14 80 00 00 e9 75 ff ff ff): adds 0x8014 to this and jumps to NodePool8e0Push FUN_0089a520, serving as the virtual node-free slot of the 8192-bucket map. |
| `0089a840` | `SoeUtil::HashListLoggingFile8192Ctor` | ✅ | ctor | Constructor of SoeUtil::HashList<Logging::LoggingFile,8192,0>: clears the head/tail/count at +8/+0xc/+0x10 and the field at +4, zeroes the 8192-bucket table at +0x14, clears the auxiliary pool tree and its counters/flag… |
| `0089a890` | `SoeUtil::HashListLoggingFile8192ScalarDtor` | 🟢 | dtor | Scalar deleting destructor of the Logging::LoggingFile hash list: runs the shared map teardown FUN_00899b50 and calls _free on the object when bit 0 of param_2 is set. |
