# Class: TinyHttp::HttpManager::PostRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2f420` | `ProcessResult` | ✅ | virtual | Virtual slot 2 of PostRequest: logs 'HttpManager::PostRequest::ProcessResult(%s:mod%d) HttpResultCode=%d' with the request name, the mod id and the result code at +0x224, then calls the manager handler's vtable +0xC wit… |
| `00d2fed0` | `PostRequest::~PostRequest` | ✅ | dtor | Releases the reference-counted member and 512-byte string, then calls the base Request destructor. |
| `00d30e90` | `PostRequest::ScalarDeletingDtor` | ✅ | dtor | Calls the PostRequest destructor, conditionally frees the object when flag bit zero is set, and returns the object pointer. |
| `00d31540` | `PostRequest::BuildAndSend ?` | 🟡 | virtual | Formats a POST HTTP/1.1 request with Host and optional User-Agent headers, adds the body and its length when present, then writes it to the transport. |
| `00d31c60` | `PostRequest::PostRequest` | 🟢 | ctor | Initializes the Request base, stores the optional body reference, and copies the supplied request metadata into the derived object. |
