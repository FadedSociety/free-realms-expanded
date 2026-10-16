# Class: EmbeddedWebBrowserLibrary

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00993cf0` | `EmbeddedWebBrowserLibrary::OnAssetsLoaded` | ✅ | virtual | On the success condition, logs successful completion and stores state 2; otherwise clears the success byte, logs the error code, and stores state 2. |
| `00993e30` | `EmbeddedWebBrowserLibrary::OnAssetLoaded` | ✅ | virtual | Logs asset success when result equals 1; otherwise clears the library success byte and logs the result code, asset name, and disk-write flag. |
| `00993e90` | `EmbeddedWebBrowserLibraryDtor` | ✅ | dtor | Installs the browser-library vtable, invokes cleanup helper 00703350, then performs embedded-browser cleanup through 00ae8460. |
| `00993fb0` | `EmbeddedWebBrowserLibrary::Ctor ⚠downgrade` | 🟢 | ctor | Installs the browser-library vtable, clears the state and callback fields, initializes a large member, and initializes the base logging object. |
| `009940b0` | `EmbeddedWebBrowserLibraryScalarDeletingDtor` | 🟢 | dtor | Calls the embedded-browser library destructor, frees the object when the deleting flag is set, and otherwise returns the object pointer. |
