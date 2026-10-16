# Class: BaseClientFileOpener

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b8990` | `BaseClientFileOpener::BaseClientFileOpener` | ✅ | ctor | Writes AppUtil::FileOpener::vftable followed by BaseClientFileOpener::vftable to the object. |
| `008b8a40` | `— ⚠reject` | ⚪ | dtor | Calls FUN_00fbe280 with param_1 in ECX; when param_2 bit 0 is set, calls _free(param_1), otherwise returns param_1 after restoring ExceptionList. |
| `00ae32a0` | `CreateBaseClientFile` | ✅ | virtual | Allocates 0x80 bytes, constructs BaseClientFile when allocation succeeds, and returns the object pointer or null. |
| `00ae34d0` | `BaseClientFileOpener::OpenForRead` | ✅ | virtual | Gets the opener backend, normalizes path separators, checks existence, initializes output metadata when supplied, obtains file size from the shared representation, and falls back to the standard opener if absent. |
