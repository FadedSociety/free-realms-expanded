# System: lib.tinyhttp

51 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ade900` | `TinyHttp::HttpManagerHandler::~HttpManagerHandler` | TinyHttp::HttpManagerHandler | Base-class destructor body of the TinyHttp handler that ScreenshotManager derives from (its constructor installs this vtable first, see the ScreenshotManager note). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade940` | `TinyHttp::HttpManagerHandler::ScalarDeletingDtor` | TinyHttp::HttpManagerHandler | Heap-delete path for TinyHttp::HttpManagerHandler objects. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00d2f3b0` | `ProcessResult` | TinyHttp::HttpManager::PutRequest | Completion hook for an HTTP PUT: logs the result and passes it to the registered handler. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2f420` | `ProcessResult` | TinyHttp::HttpManager::PostRequest | Completion hook for an HTTP POST: logs the result and passes it to the registered handler. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2f910` | `HttpConnection::OnTerminated` | TinyHttp::HttpManager::HttpConnection | TinyHttp connection failure handling and retry scheduling. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d2fb90` | `HttpConnection::~HttpConnection` | TinyHttp::HttpManager::HttpConnection | Non-deleting cleanup for TinyHttp HttpConnection objects. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d2fd10` | `Request::~Request` | TinyHttp::HttpManager::Request | Base Request cleanup shared by GetRequest, PutRequest, and PostRequest. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d2fdf0` | `PutRequest::~PutRequest` | TinyHttp::HttpManager::PutRequest | Derived PutRequest cleanup in TinyHttp. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d2fed0` | `PostRequest::~PostRequest` | TinyHttp::HttpManager::PostRequest | Derived PostRequest cleanup in TinyHttp. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d302f0` | `HttpConnection::HttpConnection` | TinyHttp::HttpManager::HttpConnection | Creates one pooled TinyHttp connection object. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30420` | `HttpConnection::ScalarDeletingDtor` | TinyHttp::HttpManager::HttpConnection | Vtable slot 0 deleting destructor for HttpConnection. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30440` | `HttpConnection::GiveTime` | TinyHttp::HttpManager::HttpConnection | Periodic timeout and reconnect processing for TinyHttp connections. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30e40` | `GetRequest::ScalarDeletingDtor` | TinyHttp::HttpManager::GetRequest | Deleting destructor for GetRequest objects. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30e70` | `PutRequest::ScalarDeletingDtor` | TinyHttp::HttpManager::PutRequest | Deleting destructor for PutRequest objects. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30e90` | `PostRequest::ScalarDeletingDtor` | TinyHttp::HttpManager::PostRequest | Deleting destructor for PostRequest objects. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |

## 🟢 likely (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2f8c0` | `FailHeadRequestAndDelete` |  | Used by the connection-termination retry-exhaustion path to retire the pending head. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30100` | `SplitUrlAuthorityAndPath` |  | HTTP request builders use it to split each request URL before formatting the request line. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30240` | `ReadLocalFileAndRespond` |  | The URL router uses this path for file:// requests; this is local file handling, not a game packet. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31770` | `RemoveConnection` |  | Removes an expired or otherwise retired connection from the HTTP manager pool. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d317e0` | `CreateConnection` |  | Adds a new connection to the TinyHttp manager's keyed pool. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31ac0` | `Request::Request` | TinyHttp::HttpManager::Request | Base initializer used by the concrete HTTP request types. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31ba0` | `PutRequest::PutRequest` | TinyHttp::HttpManager::PutRequest | Constructs a TinyHttp PUT request object. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31c60` | `PostRequest::PostRequest` | TinyHttp::HttpManager::PostRequest | Constructs a TinyHttp POST request object. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31e50` | `EnqueueGetRequest` |  | Queues an HTTP GET operation for a selected manager connection. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31f10` | `EnqueuePutRequest` |  | Queues an HTTP PUT operation for a selected manager connection. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31fd0` | `EnqueuePostRequest` |  | Queues an HTTP POST operation for a selected manager connection. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d32100` | `RouteGetUrl` |  | Dispatches a GET request to either local file handling or the TinyHttp connection manager. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d32210` | `RoutePutUrl` |  | Dispatches a PUT operation through the TinyHttp manager. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d32330` | `RoutePostUrl` |  | Dispatches a POST operation through the TinyHttp manager. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d32f60` | `HttpManager::Cleanup` |  | Shared HTTP manager cleanup used by social-share and leaderboard-related objects. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d33050` | `HttpManager::HttpManager` |  | Common TinyHttp manager construction used by its concrete manager owners. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |

## 🟡 speculative (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2f040` | `ListUnlink_230_234 ?` |  | With ECX = list header (head at [0], tail at [1], count at [2]) and argument = node, unlinks the node using its prev link at +0x230 and next link at +0x234, clears both node links and decrements the count. Returns (ret … | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2f4b0` | `ListDrainDelete_230_234 ?` |  | With ECX = list header, repeatedly unlinks the head node (links +0x230 prev and +0x234 next, count at header+8) and calls the node's vtable slot 0 with argument 1 (scalar delete) until the list is empty. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2f620` | `ParseIntArg ?` |  | Calls 00d2f160 (AtoiToOutParam) on the string at *param_1 with a local out-int and returns the parsed value (00d2f160 leaves its out-pointer in EAX, so [EAX] is the value). | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2f780` | `PumpStartFreeNodes ?` |  | While the free-list count at +0x2140 is positive and the limits at (+0x2150)+8 and +0xC and the started counter at +0x2280 allow, only when the state at (+0x212C)+0x1C equals 1; takes the head of the free list (header a… | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2f870` | `CompleteActiveHead ?` |  | Unlinks the head of the active list (header at +0x2144) through 00d2f040, calls the node's vtable slot 2, subtracts node[+0x220] from the byte counter at +0x2160, then calls the node's vtable slot 0 with argument 1 (sca… | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d30680` | `HttpConnection::HandleResponse ? ⚠rename` | TinyHttp::HttpManager::HttpConnection | Parses an HTTP status line and response headers, copies fixed-length or chunked body data into the active request, and resets or disconnects on response completion or malformed input. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30d10` | `HttpConnection::ConnectComplete ?` | TinyHttp::HttpManager::HttpConnection | Calls a helper on the connection's transport subobject, obtains a local address string, logs it, then releases the temporary string state. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30e20` | `Request::ScalarDeletingDtor ?` | TinyHttp::HttpManager::Request | Calls the Request destructor, conditionally frees the object when flag bit zero is set, and returns the object pointer. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30eb0` | `InitializeFixedStringsAndLimits ?` |  | Initializes two fixed 64-byte string members and clears or assigns the associated numeric limit and timing fields. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d30f70` | `HttpConnection::ScalarDeletingDtorWrapper ? ⚠rename` | TinyHttp::HttpManager::HttpConnection | Calls the HttpConnection destructor, frees the object when flag bit zero is set, and returns its pointer. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31020` | `CopyHttpSettings ?` |  | Copies scalar settings fields and the two fixed 64-byte strings from the source settings block to the destination. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d310e0` | `EscapeUrlComponent ?` |  | Copies bytes in the accepted printable range except reserved characters and percent-encodes other bytes as percent followed by two lowercase hexadecimal digits. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31140` | `GetRequest::BuildAndSend ?` | TinyHttp::HttpManager::GetRequest | Splits the URL, formats a GET HTTP/1.1 request with optional User-Agent and Host headers, then passes the bytes to the transport write method. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31310` | `PutRequest::BuildAndSend ?` | TinyHttp::HttpManager::PutRequest | Formats a PUT HTTP/1.1 request with Host and optional User-Agent headers, adds the body and its length when present, then writes it to the transport. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31540` | `PostRequest::BuildAndSend ?` | TinyHttp::HttpManager::PostRequest | Formats a POST HTTP/1.1 request with Host and optional User-Agent headers, adds the body and its length when present, then writes it to the transport. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31890` | `HttpManager::GiveTime ? ⚠downgrade` |  | Periodic HTTP manager connection maintenance. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d319f0` | `FindOrCreateConnection ?` |  | Hashes the supplied route and index, searches the 512-bucket connection table for a matching key, and creates a connection when none matches. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31d20` | `RemoveAllConnections ?` |  | Bulk HTTP connection cleanup used during manager teardown or reset. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d31d40` | `SelectConnectionByLoad ?` |  | Chooses a configured connection slot using the active and queued request load values maintained by the manager. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2f330` | `— ⚠reject` | TinyHttp::HttpManager::GetRequest | Completion hook for an HTTP GET: logs the result and passes it to the registered HTTP handler. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

