# Class: FlashMovieInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d55690` | `FlashMovieInterface::ScalarDeletingDtor ⚠downgrade` | 🟢 | dtor | Scalar deleting destructor: calls the base teardown FUN_010b48e0 on this, then calls _free(this) when bit 0 of the flags argument is set; returns this in eax and pops the single stack argument with ret 4. |
