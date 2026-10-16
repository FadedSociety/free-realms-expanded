# Class: GameRuntimeTerrain::Chunk

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4c330` | `LogUnavailableChunkAsset` | ✅ | virtual | Logs the result code and asset name with the FreeRealms.log format, then sets the chunk status byte at +0x38 to 1. |
| `00d4dc20` | `ChunkDtor` | ✅ | dtor | Cancels and releases job references, frees owned buffers, clears the Tile array, releases reference-counted resources, destroys render-batch arrays, and resets base vtables. |
| `00d4dea0` | `AdjustorThunk_00d4e210` | ✅ | method | Subtracts 8 from ECX, then jumps to 00d4e210; this is an adjustor thunk, not the deleting-destructor body. |
| `00d4e0a0` | `ChunkCtor` | ✅ | ctor | Installs base and Chunk vtables, stores the supplied handler pointer, clears chunk state fields, initializes Tile and render-batch arrays, and stores the handler again in the final field. |
| `00d4e210` | `ChunkScalarDeletingDtor` | ✅ | dtor | Runs Chunk cleanup, conditionally frees the Chunk object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00d4eb30` | `ChunkBuildAssetPaths` | ✅ | virtual | When the alternate-format flag is set, formats a .gck2 filename and passes it with this object to provider vtable slot +0x0C; always formats and passes a .gcnk filename the same way. |
| `00d4ec80` | `HandleChunkAssetResult` | ✅ | virtual | Requires three stored state fields, calls the chunk parser, and when parsing returns false invokes Chunk vtable slot +0x10 with status 3 and the supplied result values. |
