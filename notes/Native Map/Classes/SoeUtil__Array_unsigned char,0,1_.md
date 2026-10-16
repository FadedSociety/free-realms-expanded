# Class: SoeUtil::Array<unsigned char,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00702f80` | `ArrayAllocate ?` | 🟡 | virtual | Computes a larger capacity for the byte array and allocates backing storage using its configured allocator. |
| `00703020` | `ArrayFreeMemory ?` | 🟡 | virtual | Frees the supplied array allocation through the array memory-release routine. |
| `00703040` | `ArrayAssignOrSwap ?` | 🟡 | virtual | When the flag is set delegates assignment to the source array; otherwise swaps the source and destination storage fields. |
| `00703200` | `ArrayDestructor ?` | 🟡 | dtor | Releases the array's owned memory and performs the base byte-array destruction steps. |
| `007033a0` | `ArrayScalarDeletingDestructor ?` | 🟡 | dtor | Calls the array destructor and conditionally frees the outer object when the deleting flag requests it. |
| `00703790` | `ByteArrayConstructorFromRange ?` | 🟡 | ctor | Initializes a dynamic byte array and copies the supplied bytes into its storage. |
| `00703be0` | `SwapArrayContents ? → InlineArraySwapContents` | 🟡 | virtual | Swaps array storage through a temporary so that each array receives the other's contents. |
