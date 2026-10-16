# Class: CCallback<ClientSteamImpl,MicroTxnAuthorizationResponse_t,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae13d0` | `MicroTxnCallbackDtor` | ✅ | dtor | Installs the microtransaction callback vtable and unregisters this callback when flag bit 0 at object+4 is set. |
| `00ae13f0` | `MicroTxnCallbackDispatch` | ✅ | virtual | Loads callback function pointer from object+0x10 and tail-jumps to it for event dispatch. |
| `00ae1400` | `MicroTxnCallbackInvoke` | ✅ | virtual | Loads the callback pointer from object+0x10, passes its event argument, and returns after the indirect invocation. |
