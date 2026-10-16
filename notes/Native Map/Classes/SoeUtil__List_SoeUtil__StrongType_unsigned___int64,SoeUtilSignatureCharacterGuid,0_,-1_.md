# Class: SoeUtil::List<SoeUtil::StrongType<unsigned___int64,SoeUtilSignatureCharacterGuid,0>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008e0c50` | `ListCharacterGuidStrongType_Dtor` | ✅ | dtor | Stores the character-GUID list vtable and jumps to its list cleanup routine. |
| `008e0c70` | `ListCharacterGuid_Allocate` | ✅ | virtual | Initializes the list allocator, uses the thread or global allocator when available, and otherwise allocates 0x10 bytes with alignment 8. |
| `008e0cb0` | `FreeAligned8` | ✅ | virtual | Calls the allocator free helper with the supplied pointer and alignment 8. |
| `008e21b0` | `ListCharacterGuid_ScalarDeletingDtor` | ✅ | dtor | Installs the GUID list vtable, runs list cleanup, and optionally frees the object when flag bit 0 is set. |
| `00a8c0b0` | `ListGuid20Ctor` | ✅ | ctor | Constructs an empty list of 64-bit character-GUID values: stores the base List<StrongType<uint64,SoeUtilSignatureCharacterGuid,0>,-1> vptr at *param_1, clears param_1[1], param_1[2] and param_1[3], stores the List<...,2… |
| `00a8caa0` | `ListGuid20DtorBody` | ✅ | dtor | Destructor body with an EH frame for the 20-capacity list of character GUIDs: sets the ,20 vptr, calls 008d3ee0, zeroes the words at +0x18 and +0x1c, reads the chunk-tree root at this+0x10 and, when it is non-null, free… |
