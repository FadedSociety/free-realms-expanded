# System: platform.win32

41 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0048eaa1` | `ResolveDrawDeviceWin32` |  | Win32 draw-device registry resolution. The analyzed FUN_00546169 body is a return-zero stub, so successful runtime loading of GDraw_GDI.dll is not established. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0049141f` | `ResolveFontDeviceWin32` |  | Win32 font-device registry resolution. The analyzed FUN_00546169 body is a return-zero stub, so successful runtime loading of GDraw_GDI.dll is not established. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00c194b0` | `OpenFreeRealmsWebsite` |  | Asks the launcher to open the Free Realms website. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0048dcf8` | `GetCursorHotspot` |  | Win32 icon/cursor hotspot helper for an object carrying handle, name and fallback coordinate fields. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048ea78` | `DtorVtables1769298` |  | destructor of a two-vtable object in the Win32 driver block; whether it is the draw device the loader FUN_0048EAA1 fills is suggested only by the neighbourhood, so the class is left unnamed. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048ebd6` | `ReleaseGlobalsAndUnregisterIds` |  | Registry and global-interface teardown associated with the draw/font device registry; identifier class meanings are unresolved. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

## 🟡 speculative (33)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00701a20` | `CompareFileContents ?` |  | Opens both paths, compares file sizes, reads matching chunks, compares their bytes, and closes the handles. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00701d20` | `WriteTextWithNewlineOption ?` |  | Writes a C string, optionally converts line feeds to CRLF in chunks, and accumulates the number of bytes written. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00701f60` | `WriteTextToFile ?` |  | Opens the destination in write mode, forwards the supplied text to the text writer, then closes the file. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702140` | `ThunkTo00701f60 ? → WriteIStringToFile` |  | Unconditionally jumps to function 00701f60 without implementing a separate body. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702200` | `ReadFileToString ?` |  | Reads the remainder of an open file into a dynamic string, appends a terminator, optionally converts CRLF pairs, and returns success. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007022d0` | `ReadFileToByteArray ?` |  | Opens a file, sizes a byte array, reads the file contents into its storage, records the actual count, and closes the handle. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702440` | `ReadTextFile ?` |  | Opens a file for reading, loads its text through the string reader, and closes the file handle. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007027f0` | `WindowsTickTime ?` |  | Reads the Windows millisecond tick source and updates the helper's stored time-tracking pair. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `0070290b` | `QueryPerfCounterScaledTime ?` |  | Reads the performance counter and scales its value using the cached counter frequency. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007029b0` | `QueryPerfCounterTime ?` |  | Queries the performance-counter frequency and current counter, then computes a 64-bit scaled time value. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702c40` | `GetFileMetadata ?` |  | Calls GetFileInformationByHandle, fills file size, attribute flags, creation time, and write time, and returns the API result. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702d50` | `FileRead ?` |  | Reads bytes from the active file handle, logs a ReadFile error, stores failure state, and returns whether any bytes were read. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702dd0` | `GetFileSize ?` |  | Calls GetFileSizeEx for the active handle and returns the 64-bit size, or minus one when the handle is invalid. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702e00` | `QueryCurrentFileOffset ?` |  | Calls SetFilePointer with zero displacement and FILE_CURRENT to query the current file offset. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702e40` | `SeekFileAbsolute ?` |  | Combines the supplied low and high offset words and seeks the active file handle to that absolute position. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702e80` | `SeekFileEnd ?` |  | Seeks the active file handle to its end using the supplied offset words. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702eb0` | `IsFileAtEnd ?` |  | Compares the current file position with the file size and returns whether the positions match. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702f40` | `GetFileStateField ?` |  | Returns the dword stored at offset four in the file object's nested data structure. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702f50` | `HasValidFileHandle ?` |  | Returns the file object's valid-handle flag as a boolean result. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702f60` | `GetMetadataIfOpen ?` |  | If the file handle is valid, fills metadata through the metadata helper; otherwise returns false. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00703150` | `FileWrite ?` |  | Writes bytes to the active file, optionally seeks to the end, logs WriteFile errors, and updates the file state. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007034d0` | `CloseFileAndReset ?` |  | Closes the active file handle, resets its marker, releases the embedded string reference, and clears related fields. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007039b0` | `InitializeFileMembers ?` |  | Initializes embedded byte-array and fixed-string members with their vtables, counts, capacities, and empty states. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00703a30` | `DestroyFileMembers ?` |  | Closes an active embedded file handle, releases member strings and arrays, and restores their interface vtables. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00703e10` | `FileOpen ?` |  | Normalizes and converts a path, calls CreateFileW with the selected open mode, stores the handle, and logs open failures. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007040c0` | `MoveFileWithNormalizedPaths ?` |  | Converts both UTF-16 paths, changes forward slashes to backslashes, calls MoveFileW, and returns its result. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704280` | `DeleteFileFromPath ?` |  | Converts the supplied path to UTF-16, calls DeleteFileW, and returns the API result. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704390` | `GetPathFileMetadata ?` |  | Converts a path, opens the file for metadata access, fills its metadata record, and closes the handle. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704490` | `SetPathFileAttributes ?` |  | Converts a path to UTF-16, selects the requested attribute bits, calls SetFileAttributesW, and returns success. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007045c0` | `MoveFilePathWrapper ?` |  | Converts the two supplied path strings and forwards them to the normalized MoveFileW helper. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704750` | `DeleteFilePathWrapper ?` |  | Converts the supplied path and forwards the resulting wide string to the file-deletion helper. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704840` | `GetFileMetadataPathWrapper ?` |  | Converts the supplied path and forwards it to the file-metadata helper. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704940` | `SetFileAttributesPathWrapper ?` |  | Converts the supplied path and forwards it with attribute flags to the file-attribute helper. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0048ee01` | `— ⚠reject` |  | Lazy ensure-and-forward wrapper; slot +0x18 target behavior is not identified. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00701e20` | `— ⚠reject` |  | Opens a file, allocates storage for its contents, reads the bytes, frees the allocation on failure, and stores the resulting pointer. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |

