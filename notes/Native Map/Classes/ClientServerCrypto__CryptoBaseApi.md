# Class: ClientServerCrypto::CryptoBaseApi

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d33270` | `CryptoBaseApi::CryptoBaseApi` | 🟢 | ctor | Installs the CryptoBaseApi primary and secondary vtables, then calls the shared crypto initialization helper. |
| `00d33320` | `CryptoBaseApi::~CryptoBaseApi ?` | 🟡 | dtor | Releases the child API object through its virtual destructor and runs the shared base cleanup routine. |
| `00d33450` | `CryptoBaseApi::DeletingDtorAdjustor` | ✅ | dtor | Subtracts 0xc0 from ECX and jumps to 00d33490, adjusting the receiver before the shared deleting destructor. |
| `00d33460` | `CryptoBaseApi::DeletingDtorAdjustor` | ✅ | dtor | Subtracts 0xc4 from ECX and jumps to 00d33490, adjusting the receiver before the shared deleting destructor. |
| `00d33490` | `CryptoBaseApi::ScalarDeletingDtor` | ✅ | dtor | Calls the CryptoBaseApi destructor, frees this when flag bit zero is set, and returns this. |
| `00d335d0` | `CryptoBaseApi::ForwardBuffer → CryptoBaseApi::InternalSend` | 🟢 | virtual | Optionally wraps the supplied bytes through PacketUtils::WrapPacket, then passes the selected buffer to a shared transport helper or receiver virtual method. |
| `00d336f0` | `CryptoBaseApi::InternalSendSecure` | ✅ | virtual | Checks provider readiness, wraps or encrypts the supplied buffer, passes the result to the transport helper, and logs when wrapping fails. |
| `00d33900` | `CryptoBaseApi::OnRoutePacket` | ✅ | virtual | Attempts to unwrap the input through the current provider, dispatches decoded bytes to a shared route handler, and falls back to the marker-based wrapper path when needed. |
