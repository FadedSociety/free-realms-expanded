# Class: GameRuntimeTerrain::DecompressChunkDataJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4c2c0` | `PrepareChunkDecodeBuffers` | ✅ | virtual | Allocates buffers for the stored input and output sizes, passes each buffer and its lengths to 010ebdc0, and clears the job success flag when that call returns -1. |
| `00d4d720` | `DecompressChunkDataJobCtor` | ✅ | ctor | Initializes the job base and its input fields, stores size and owner references, installs the job vtable, and increments the referenced object's count. |
| `00d4d7e0` | `DecompressChunkDataJobDtor` | ✅ | dtor | Decrements its referenced owner count, releases the owner at zero, frees one buffer, frees another allocation with alignment 0x10, then restores base vtables. |
| `00d4d8a0` | `DecompressChunkDataJobScalarDeletingDtor` | ✅ | dtor | Runs job cleanup, frees the job object when flag bit 0 is set, and otherwise returns the object pointer. |
