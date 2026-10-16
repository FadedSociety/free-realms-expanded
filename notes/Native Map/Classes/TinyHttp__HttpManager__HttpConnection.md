# Class: TinyHttp::HttpManager::HttpConnection

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2f910` | `HttpConnection::OnTerminated` | ✅ | virtual | Logs termination, requeues sent requests, checks retry count, then releases transport state. |
| `00d2fb90` | `HttpConnection::~HttpConnection` | ✅ | dtor | Releases the attached transport object, two fixed strings, and the 8 KB byte array, then restores the TCP base vtable. |
| `00d302f0` | `HttpConnection::HttpConnection` | ✅ | ctor | Installs the connection vtable, initializes the 8 KB buffer and fixed strings, stores constructor inputs, and zeros connection state. |
| `00d30420` | `HttpConnection::ScalarDeletingDtor` | ✅ | dtor | Calls the HttpConnection destructor, frees this when flag bit 0 is set, and returns this. |
| `00d30440` | `HttpConnection::GiveTime` | ✅ | method | Creates a connection when requests are pending, pumps connection work, and disconnects an idle connection after its configured timeout. |
| `00d30680` | `HttpConnection::HandleResponse ? ⚠rename` | 🟡 | virtual | Parses an HTTP status line and response headers, copies fixed-length or chunked body data into the active request, and resets or disconnects on response completion or malformed input. |
| `00d30d10` | `HttpConnection::ConnectComplete ?` | 🟡 | virtual | Calls a helper on the connection's transport subobject, obtains a local address string, logs it, then releases the temporary string state. |
| `00d30f70` | `HttpConnection::ScalarDeletingDtorWrapper ? ⚠rename` | 🟡 | dtor | Calls the HttpConnection destructor, frees the object when flag bit zero is set, and returns its pointer. |
