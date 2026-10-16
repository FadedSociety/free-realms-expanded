# Class: TcpLibrary::TcpConnectionHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2edd0` | `DtorBody` | ✅ | dtor | Destructor body: stores the TcpConnectionHandler vptr at +0 and returns; no member cleanup. |
| `00d2edf0` | `DeleteIfFlagBit0 ? → ScalarDeletingDtor` | 🟡 | virtual | Virtual slot 0 of TcpConnectionHandler: stores the vptr, frees this through _free when bit 0 of the byte argument is set, returns this (ret 4). It does not call a separate dtor body. |
