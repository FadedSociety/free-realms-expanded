# Class: SoeUtil::HashListMap<int,Logging::LoggingFile,8192,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00897970` | `SoeUtil::HashListMapLoggingFile8192Ctor` | ✅ | ctor | Constructor of the SoeUtil::HashListMap<int,Logging::LoggingFile,8192,-1> container: installs the class vtable, clears the list head/tail/count at +8/+0xc/+0x10 and the field at +4, and zeroes the 8192-bucket pointer ta… |
| `00897f20` | `HashListMapLoggingFile8192AllocateNode` | ✅ | method | Allocates a 0x8e0-byte map node through the current-thread memory manager, then the global manager, or FUN_006fb950(0x8e0,8) as fallback; the second allocator argument is 8. |
| `00897f60` | `HashListMapLoggingFile8192FreeNode` | ✅ | method | Thin free wrapper: releases param_1 through FreeMemory with alignment 8 and returns nothing. |
| `00899ae0` | `HashListMapLoggingFile8192Dtor` | ✅ | dtor | Destructor of the negative-parameter file map: installs the class vtable and loops removing every element from the ordered list (head at +8) through FUN_008997e0; reached from an EH unwind funclet. |
| `00899b10` | `HashListMapLoggingFile8192ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the negative-parameter file map: installs the class vtable, removes every element through FUN_008997e0 and calls _free on the map when bit 0 of param_2 is set. |
