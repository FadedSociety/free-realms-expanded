# Class: SoeUtil::Set<SoeUtil::StrongType<unsigned_int,AppPhysics::SoeUtilSignatureCollisionObjectReference,1>,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b2d0` | `Set::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095e590` | `SetDestructor → SoeUtil::Set<SoeUtil::StrongType<unsigned_int,AppPhysics::SoeUt…` | ✅ | dtor | Installs the set vtable, recursively destroys both tree branches, releases the root node through the virtual cleanup slot, and clears root and count fields. |
| `0095f1e0` | `SetScalarDeletingDtor → SoeUtil::Set<SoeUtil::StrongType<unsigned_int,AppPhysic…` | ✅ | virtual | Destroys the tree root and children, clears set state, and frees the object when the deleting flag bit is set. |
