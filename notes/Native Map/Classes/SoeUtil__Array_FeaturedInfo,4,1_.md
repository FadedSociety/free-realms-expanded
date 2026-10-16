# Class: SoeUtil::Array<FeaturedInfo,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c84510` | `FeaturedInfoArray4Dtor` | ✅ | dtor | Cleans inline FeaturedInfo storage, restores the zero-inline vtable, destroys elements, and frees heap storage when used. |
| `00c84ca0` | `FeaturedInfoArray4ScalarDeletingDtor` | ✅ | dtor | Calls the inline-array cleanup routine and conditionally frees the object when flag bit 0 is set. |
