# Class: SoeUtil::Array<SoeUtil::IString<char>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071b3e0` | `StringArrayConstructor` | ✅ | ctor | Installs string-array vtable and zeros its data pointer, size, and capacity. |
| `0071b620` | `StringArrayChooseCapacity` | ✅ | virtual | In non-resize mode, if requested count exceeds capacity chooses requested*5/4; otherwise reuses storage while capacity<=requested*4/3, and only then chooses requested*6/5. In resize mode it reuses storage when counts ma… |
| `0071b6e0` | `StringArrayVirtualOperation3` | ✅ | virtual | If the flag is nonzero, calls the second operand vtable entry at byte offset 0x0c. Otherwise swaps backing pointers at offset 4 and swaps the dwords at offsets 8 and 0x0c using xor operations. |
| `0071ba50` | `StringArrayDestructor` | ✅ | dtor | Installs string-array vtable, destroys elements, frees backing storage, and zeros the pointer. |
| `0071bb00` | `StringArrayScalarDeletingDtor` | ✅ | dtor | Runs string-array destructor and frees object when deleting flag bit is set. |
