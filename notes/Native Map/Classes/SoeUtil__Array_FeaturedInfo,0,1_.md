# Class: SoeUtil::Array<FeaturedInfo,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c844e0` | `FeaturedInfoArrayDtor` | ✅ | dtor | Installs the array vtable, destroys active FeaturedInfo elements, frees backing storage, and clears its pointer. |
| `00c847a0` | `FeaturedInfoArrayScalarDeletingDtor` | ✅ | dtor | Destroys active elements, frees array storage, and conditionally frees the array object when flag bit 0 is set. |
| `00c85300` | `— ⚠reject` | ⚪ | virtual | Releases the source and destination arrays using their element counts, clears temporary storage, then frees the allocation. |
