# Class: TinyHttp::HttpManager::PutRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2f3b0` | `ProcessResult` | ✅ | virtual | Virtual slot 2 of PutRequest: logs 'HttpManager::PutRequest::ProcessResult(%s:mod%d) HttpResultCode=%d' with the request name, the mod id and the result code at +0x224, then calls the manager handler's vtable +8 with th… |
| `00d2fdf0` | `PutRequest::~PutRequest` | ✅ | dtor | Releases the reference-counted member and 512-byte string, then calls the base Request destructor. |
| `00d30e70` | `PutRequest::ScalarDeletingDtor` | ✅ | dtor | Calls the PutRequest destructor, conditionally frees the object when flag bit zero is set, and returns the object pointer. |
| `00d31310` | `PutRequest::BuildAndSend ?` | 🟡 | virtual | Formats a PUT HTTP/1.1 request with Host and optional User-Agent headers, adds the body and its length when present, then writes it to the transport. |
| `00d31ba0` | `PutRequest::PutRequest` | 🟢 | ctor | Initializes the Request base, stores the optional body reference, and copies the supplied request metadata into the derived object. |
