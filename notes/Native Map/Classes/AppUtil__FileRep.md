# Class: AppUtil::FileRep

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae2d50` | `FileRepCtor` | ✅ | ctor | Calls the base file representation constructor, installs AppUtil::FileRep vtable, and returns this. |
| `00ae2db0` | `FileRepIsComplete` | ✅ | virtual | Calls vtable methods at offsets +0x1c and +0x18 and returns 1 when their 64-bit results are equal, otherwise 0. |
| `00ae2de0` | `FileRep::ScalarDeletingDtor` | ✅ | dtor | Calls the file representation base destructor, optionally frees this when flag bit 0 is set, and returns this otherwise. |
