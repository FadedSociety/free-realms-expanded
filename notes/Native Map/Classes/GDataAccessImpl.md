# Class: GDataAccessImpl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00428d6c` | `GDataAccessImpl::GDataAccessImpl` | ✅ | ctor | Constructor of the GFC data-access implementation: installs the GDataAccessImpl vtable at +0, zeroes +8, +0xC and +0x10 and stores the argument at +0x14; returns this. |
| `00428d89` | `GDataAccessImpl::ScalarDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a data-access object: runs the destructor chain 00424eca (whose own analysis rewrites the GInterface vtable) and frees the object with _free when the flag argument's low bit is set; r… |
| `0042af23` | `GDataAccessImplCopy` | ✅ | ctor | Copy-style constructor of a data-access object: installs the GDataAccessImpl vtable at +0, copies the source's embedded 32-byte-element vector at source+4 through 00427ac0 and then copies the source dword at +0x14 into … |
| `0042af4a` | `GDataAccessImplCreate` | ✅ | function | Inline factory: allocates 0x34 bytes through 00409351, constructs a data-access object in place (GObjectBase base, param_1 at +0x14, the intermediate 0x175d1a0 table at +0x18, the GDataAccessImpl vtable at +0x1C which t… |
