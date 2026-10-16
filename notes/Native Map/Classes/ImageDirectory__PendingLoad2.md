# Class: ImageDirectory::PendingLoad2

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00989030` | `PendingLoad2_OnAssetReady` | ✅ | virtual | Casts param_2 from Asset to ImageAsset2 using the two RTTI descriptors, calls GFxWrap::ReclaimImageFromAssetLimbo and ReleaseImageReference using this+8, calls the global manager, removes this pending entry through 0098… |
| `00989740` | `PendingLoad2_CreateImageAsset` | ✅ | virtual | Allocates 0x2c bytes using the guarded allocator 00d833a6, returns null if allocation fails, otherwise calls the ImageAsset2 constructor and returns the new object. |
| `009897a0` | `— ⚠reject` | ⚪ | ctor | Installs AssetRequester then PendingLoad2 vtables, initializes the IString subobject vtable and static empty-string pointer, zeroes the string length/reference fields, stores param_2, and clears the pending image pointe… |
| `00989dc0` | `ImageDirectory::PendingLoad2::ScalarDeletingDestructor` | ✅ | dtor | Calls the AssetRequester destructor body, frees this only when deleting flag bit 0 is set, and otherwise returns this. |
| `0098b0d0` | `ImageDirectory::PendingLoad2::ReportUnavailableAsset` | ✅ | virtual | Formats and logs the unavailable asset name, image id, and URL; releases and detaches its image reference, then removes the nested pending-load entry. |
