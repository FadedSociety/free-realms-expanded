# Class: SoeUtil::HashListMap<int,RailCameraDefinition,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bdb20` | `RailCameraDefinitionHashListMap32::InitBuckets` | 🟢 | init | Initializes bucket sentinels, advances a 32-bit pointer by 0x40 elements (0x100 bytes) per step, and stores capacity 0x20. |
| `008f38a0` | `HashListMapRailCameraDefinition_Dtor` | ✅ | dtor | Installs the inline map vtable, unlinks all head nodes, recursively frees the pool tree and block, clears pool fields, installs the dynamic map vtable, then drains remaining nodes. |
| `008f42f0` | `HashListMapRailCameraDefinition_Slot1Adjustor` | ✅ | virtual | Adds 0x114 to ECX and jumps to 008f4240; this is an adjustor thunk, not the target body. |
| `008f4300` | `HashListMapRailCameraDefinition_Slot2Adjustor` | ✅ | virtual | Adds 0x114 to ECX and jumps to 008f4290; this is an adjustor thunk, not the target body. |
