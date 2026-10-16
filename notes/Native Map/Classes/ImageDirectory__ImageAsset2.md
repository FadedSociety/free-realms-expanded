# Class: ImageDirectory::ImageAsset2

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00989470` | `ImageAsset2_Constructor` | ✅ | ctor | Calls 0071bb80 for base setup, installs ImageAsset2's vtable, zeroes field param_1[10], and returns this. |
| `009894d0` | `ImageAsset2_Destructor` | ✅ | dtor | Installs the ImageAsset2 vtable, releases and detaches the image reference at param_1[10] when nonzero, clears that field, calls 0071bc20 for base teardown, and restores exception state. |
| `00989540` | `ImageAsset2_OnImageBytes` | ✅ | virtual | Initializes a temporary IString and calls Asset vtable slot +8 with ECX=the Asset object and a stack pointer to that IString (raw bytes show the Ghidra stack-cookie expression is not a real argument). It hashes the retu… |
| `00989b70` | `ImageAsset2::ScalarDeletingDestructor` | ✅ | dtor | Calls the ImageAsset2 destructor body; frees this only when deleting flag bit 0 is set, then otherwise returns this. |
| `0098af60` | `ImageAsset2::Method2` | ✅ | virtual | Initializes a temporary IString, obtains a string-producing value through 0071c050, assigns from the returned data pointer plus five bytes, releases the temporary, and returns. |
