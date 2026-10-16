# Class: SoeUtil::HashListMap<int,Logging::LoggingFile,8192,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00899b50` | `HashListMapLoggingFile8192ZeroDtor` | ✅ | dtor | Destructor of the zero-parameter file map: installs the derived vtable, removes every element through FUN_008997e0, frees the auxiliary node-pool tree at +0x8014 (children via FUN_008981c0 with alignment 8, root via Fre… |
| `0089a5b0` | `HashListMapLoggingFile8192ZeroScalarDtor` | 🟢 | dtor | Scalar deleting destructor of the zero-parameter file map: runs the full map destructor FUN_00899b50 and calls _free on the map when bit 0 of param_2 is set. |
| `0089a5d0` | `HashListMapLoggingFile8192ZeroCtor` | ✅ | ctor | Constructor of the zero-parameter file map: clears the list head/tail/count at +8/+0xc/+0x10 and the field at +4, zeroes the 8192-bucket table at +0x14 with _memset(...,0x8000), installs the derived vtable, clears the a… |
