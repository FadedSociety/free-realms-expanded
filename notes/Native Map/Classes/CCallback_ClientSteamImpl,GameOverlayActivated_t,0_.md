# Class: CCallback<ClientSteamImpl,GameOverlayActivated_t,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae1370` | `OverlayCallbackDtor` | ✅ | dtor | Installs the overlay callback vtable and unregisters this callback when flag bit 0 at object+4 is set. |
| `00ae1390` | `OverlayCallbackDispatch` | ✅ | virtual | Loads callback function pointer from object+0x10 and tail-jumps to it with the first argument as the event pointer. |
| `00ae13a0` | `OverlayCallbackInvoke` | ✅ | virtual | Loads callback pointer from object+0x10, passes its argument, and returns after the indirect call. |
