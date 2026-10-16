# Class: DelayedAssetLoader::FileCreateJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae51e0` | `DelayedAssetLoader::FileCreateJob::Destroy` | ✅ | dtor | Rewrites the job vtable, calls the job cleanup helper and base cleanup, then restores the Job and RefCounted base vtables. |
| `00ae56f0` | `DelayedAssetLoader::FileCreateJob::Ctor` | ✅ | ctor | Initializes the job base, installs the FileCreateJob vtable, constructs a StringFixed<char,512> member, stores two constructor values, clears the trailing byte, and increments the referenced object's count at +0x14. |
| `00ae5790` | `DelayedAssetLoader::FileCreateJob::ScalarDeletingDtor` | ✅ | dtor | Calls the FileCreateJob destructor body, conditionally frees the object when flag bit 0 is set, and returns the original pointer otherwise. |
| `00ae57b0` | `— ⚠reject` | ⚪ | virtual | Searches the stored path for the configured substring, copies a matching suffix into a fixed 8192-character string, calls further helpers with path and job-associated fields, and writes a result byte at +0x274. |
