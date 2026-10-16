# Class: TinyHttp::HttpManager::Request

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2fd10` | `Request::~Request` | ✅ | dtor | Decrements the reference-counted member at +0x228, then releases the fixed 512-byte URL string. |
| `00d30e20` | `Request::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the Request destructor, conditionally frees the object when flag bit zero is set, and returns the object pointer. |
| `00d31ac0` | `Request::Request` | 🟢 | ctor | Stores request inputs, escapes the supplied URL into the request string, and initializes request status, size, and retry state. |
