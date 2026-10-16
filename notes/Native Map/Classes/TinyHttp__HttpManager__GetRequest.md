# Class: TinyHttp::HttpManager::GetRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2f330` | `— ⚠reject` | ⚪ | virtual | Virtual slot 2 of GetRequest: logs 'RECEIVED(%d) %s (mod=%d)' with the response code (or -1 when none), the request name at +0xC and the manager's mod id at manager+0x2284. Then, when the handler at (manager+0x2150)+0x1… |
| `00d30e40` | `GetRequest::ScalarDeletingDtor` | ✅ | dtor | Installs the GetRequest vtable, calls the Request destructor, conditionally frees the object, and returns its pointer. |
| `00d31140` | `GetRequest::BuildAndSend ?` | 🟡 | virtual | Splits the URL, formats a GET HTTP/1.1 request with optional User-Agent and Host headers, then passes the bytes to the transport write method. |
