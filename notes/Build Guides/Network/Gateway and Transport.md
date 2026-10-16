---
type: build-guide
section: Network
updated: 2026-10-12
---

# Gateway and Transport

Part of [[Build Guides]] › Network. Related: [[Command System]], [[OpenSSL and Crypto]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

4762 accepted functions: 1992 confirmed, 1699 likely, 552 speculative, 519 unknown. Network-linked (`wire`): 1157.

Subsystems: `net.packet` (1426), `net.packet.player_update` (280), `net.packet.lobby` (268), `net.udp` (256), `net.packet.vehicle` (233), `net.packet.housing` (175), `net.gateway` (170), `net.packet.minigame` (168), `net.packet.command` (166), `net.packet.encounter` (166), `net.packet.guild` (141), `net.packet.activity` (136), `net.packet.soccer` (128), `net.tcp` (97), `net.packet.group` (73), `net.packet.shipcombat` (72), `net.client` (70), `net.packet.ability` (61), `net.packet.client_update` (50), `net.packet.repeating_activity` (44), `net.packet.combat` (42), `net.packet.broker` (41), `net.ssl` (39), `net.packet.inventory` (33), `net.packet.reward` (28), `net.packet.ui` (26), `lib.ssl` (26), `net.packet.promotions` (25), `net.packet.rating` (24), `net.packet.social_share` (24), `net.packet.wall_of_data` (23), `net.packet.battlemages` (23), `net.packet.player_title` (22), `net.packet.quickchat` (22), `net.packet.fotomat` (21), `net.packet.ingamepurchase` (21), `net.packet.lobby_game_def` (20), `net.packet.reference_data` (17), `net.packet.quest` (15), `net.packet.namechange` (14), `net.packet.inspect` (12), `net.packet.report` (11), `net.packet.job_customization` (11), `net.packet.client_path` (7), `net.packet.objective` (7), `net.packet.point_of_interest` (7), `net.packet.loyalty` (6), `net.compression` (4), `net.packet.mount` (3), `net.packet.matchmaking` (3), `net.packet.factory` (3), `net` (1), `net.dns` (1)

_Large or library guide: only confirmed and likely functions are listed (3691 of 4762)._

### `lib.ssl`

**HttpLibrary::HttpConnectionAdapterT<SslLibrary::SslConnectionFilter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5fa90` | HttpConnectionAdapterSendThunk | likely | Loads ECX from this+4 and tail-jumps to 00f92220 (which loads ECX from the held pointer and jumps on to the SSL filter send path); vtable slot 0, not a destruc… |

**SslConnectionImpl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f924b0` | SslConnectionImpl::DoHostCheck | confirmed | Fetches the peer certificate from the SSL object at this+0x2020 and reads its subject commonName (NID 0xd) into a 256-byte stack buffer. Skips leading '*' char… |

**SslLibrary::LogHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5f270` | ScalarDeletingDtorBody | confirmed | Stores SslLibrary::LogHandler::vftable into the object and, when bit 0 of the flag argument is set, calls _free on the object; returns the object pointer. Live… |

**SslLibrary::MyDataMap** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f92b90` | SslLibrary::MyDataMap::Append | confirmed | Under the guard at 0x1cb4190 (00fc45f0 enter, 00fc4610 leave), lazily initialises the global Array<DataPair> at 0x1cb4220 and atexit cleanup, grows it through … |
| `00f92c60` | SslLibrary::MyDataMap::Remove | confirmed | Under the same guard, scans the global pair array for the first entry whose key equals param_1 and erases that entry through 00f92a10(index,1). Does nothing wh… |
| `00f930c0` | SslLibrary::MyDataMap::Find | confirmed | Under the guard at 0x1cb4190 (GuardEnter 00fc45f0, GuardLeave 00fc4610), lazily initialises the global Array<DataPair> at 0x1cb4220, then scans its entries for… |

**SslLibrary::SslConnectionFilterImpl** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f92660` | SslLibrary::SslConnectionFilterImpl::DoAuthenticate | confirmed | Calls 01192510 on the SSL object at this+0x2020. Result 1: when the byte at this+0x2034 is set, runs DoHostCheck (00f924b0) and fails with 00f91980(0x23) if it… |
| `00f928f0` | SslLibrary::SslConnectionFilterImpl::SslConnectionFilterImpl | confirmed | Stores the SslConnectionFilterImpl vtable at +0, calls 00f93bc0 on this, stores the ReportHandlerAdapter vtable at +0x14, zeroes the word fields at +8..+0x18 a… |
| `00f92d20` | SslLibrary::SslConnectionFilterImpl::~SslConnectionFilterIm… | confirmed | Stores the SslConnectionFilterImpl vtable, removes the SSL handle at +0x2020 from the data map (00f92c60) and calls 01191b90 on it, calls 01190530 on the objec… |
| `00f92e20` | SslLibrary::SslConnectionFilterImpl::ScalarDeletingDtor | confirmed | Vtable slot [0] of SslConnectionFilterImpl: calls the destructor body 00f92d20 on this, then calls _free(this) when bit 0 of the argument byte is set. Returns … |
| `00f92e40` | SslLibrary::SslConnectionFilterImpl::ConfigAsServer | confirmed | Thiscall with the certificate file (param_2) and key file (param_3). Returns 0 with a log line when either is null. Otherwise creates an SSL context (method fr… |
| `00f93030` | SslLibrary::SslConnectionFilterImpl::AttachClient | confirmed | Thiscall(host, transport, addRef flag). Returns 0 with a log line when the client-configured flag at this+0x2034 is clear, when the host string is null or empt… |
| `00f936e0` | SslLibrary::SslConnectionFilterImpl::ConfigAsClient | confirmed | Thiscall(certificate file, certificate path, verify flags). Returns 0 with a log line when both the file and path are null. Otherwise creates an SSL context (m… |
| `00f93840` | SslLibrary::SslConnectionFilterImpl::ConfigClient | confirmed | Thiscall(certificate file, certificate path, handler, verify flags). Logs and returns 0 when the handler argument is null. Otherwise calls ConfigAsClient 00f93… |
| `00f92720` | SslLibrary::SslConnectionFilterImpl::DispatchByState | likely | ECX points to the holder of the filter pointer: loads the filter object from it. Returns AL=0 when the word at object+8 is null or the state at object+0x2048 i… |
| `00f92a80` | SslLibrary::SslConnectionFilterImpl::Create | likely | Allocates 0x2050 bytes through 00d833a6 (operator new). On success runs the constructor 00f928f0 on the block and stores the pointer in *ECX; on failure stores… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f931b0` | SslLibrary::VerifyCallback | confirmed | Certificate verify callback with (preverify_ok, X509_STORE_CTX*). Gets the SSL handle from the store context, looks it up with 00f930c0 and, when found, reads … |
| `00f93ad0` | SslGlobalRefAcquire | likely | Under the guard at 0x1cb4ba8 (GuardEnter 00fc45f0): when the global reference count at 0x1cb4ba0 is zero, registers four OpenSSL callbacks through 011b51f0 (00… |

### `net`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a8030` | EstablishCSC | confirmed | Connection-establishment sequence with four failure logs: fails with 'CSC - Gateway is NULL.' when this+0x280 is null; fails with 'CSC - already established.' … |

### `net.client`

**BaseClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00904850` | BaseClient::IdleKick | confirmed | Thiscall with a byte argument param_2. Sets the byte at this+0xc08 to 1, looks up the label 'ErrorDisconnected' when param_2 is zero or 'ErrorLabelIdle' otherw… |
| `009049a0` | BaseClient::CheckIdleTimeout | likely | Virtual, fastcall on this. Calls FUN_00ad3380 and, when nonzero, stores the 64-bit clock from FUN_00702670 at this+0xc00/+0xc04. Stores the clock minus the val… |

**ClientGatewayApi** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acc810` | ClientGatewayApi::ClientGatewayApi | confirmed | Constructor: registers the API name 'CGAPI_527' by running the crypto/API base constructor FUN_00d33270 (ClientServerCrypto::CryptoBaseApi) with the caller's t… |
| `00acc8f0` | ClientGatewayApi::Destructor | confirmed | 92-byte destructor body inside an SEH frame: reinstalls the three ClientGatewayApi vtables and calls FUN_00d33320, the ClientServerCrypto::CryptoBaseApi destru… |
| `00acc970` | ClientGatewayApi::OnFailed | confirmed | Virtual slot 15 of ClientGatewayApi: prints 'ClientGatewayApi::OnFailed...\n' with _wprintf and, when the callback object at this+0xbf0 exists, calls its virtu… |
| `00acc9a0` | ClientGatewayApi::OnConnect | confirmed | Virtual slot 16 of ClientGatewayApi: prints 'ClientGatewayApi::OnConnect...\n' with _wprintf and, when the callback object at this+0xbf0 exists, calls its virt… |
| `00acc9d0` | ClientGatewayApi::OnDisconnect | confirmed | Virtual slot 17 of ClientGatewayApi: prints 'ClientGatewayApi::OnDisconnect...\n' with _wprintf and, when the callback object at this+0xbf0 exists, calls its v… |
| `00accae0` | ClientGatewayApi::ScalarDeletingDtor | confirmed | 27-byte scalar deleting destructor: calls the destructor body 00acc8f0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00acc950` | ClientGatewayApi::SetApiKind2 | likely | 17-byte wrapper (bytes 8b 44 24 04 6a 00 50 6a 02 e8 ..): pushes 0, the single argument and the constant 2 and calls FUN_00d333c0 with this (ecx) unchanged, re… |
| `00acca00` | ClientGatewayApi::ScalarDeletingDtorAdjustor_c4 | likely | 11-byte adjustor thunk (bytes 81 e9 c4 00 00 00 e9 ..): subtracts 0xc4 from this and jumps to the scalar deleting destructor 00accae0; it is slot 0 of the seco… |
| `00acca10` | ClientGatewayApi::ScalarDeletingDtorAdjustor_c0 | likely | 11-byte adjustor thunk (bytes 81 e9 c0 00 00 00 e9 ..): subtracts 0xc0 from this and jumps to the scalar deleting destructor 00accae0; it is slot 0 of the othe… |

**ClientServerCore::BaseApi** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d5c80` | ClientServerCore::BaseApi::InitDefaults | confirmed | Field-initialisation body called at the end of the BaseApi constructor 010d6a90 (thiscall, one stack argument for the owner). Stores this+0xc4 (subobject addre… |
| `010d5d40` | ClientServerCore::BaseApi::DisconnectRequest | confirmed | Virtual, slot [8] of BaseApi and the sibling API vtables. Logs 'BaseApi disconnect request (%s)' through 00704dd0 when the log flag is set, clears the byte at … |
| `010d65d0` | ClientServerCore::BaseApi::~BaseApi | confirmed | Destructor body. Stores ClientServerCore::BaseApi::vftable at this, +0xc0 and +0xc4; logs 'BaseApi destructed (%s)' when the owner log flag is set; when the tr… |
| `010d6820` | ClientServerCore::BaseApi::EstablishConnection | confirmed | Thiscall with two stack arguments (address, timeout). Clears the counters at +0x37c and +0x380, releases the name string at +0x32c, stores the 64-bit tick from… |
| `010d6950` | ClientServerCore::BaseApi::AutoReconnectTick | confirmed | Thiscall with one stack argument. When the byte at +0x1f5 is clear and the byte at +0x324 is set and the count at +0x210 is positive, it measures elapsed time … |
| `010d6a70` | ClientServerCore::BaseApi::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the BaseApi destructor body 010d65d0 on this, then when bit 0 of the argument is set calls _free on the object, and returns t… |
| `010d6a90` | ClientServerCore::BaseApi::BaseApi | confirmed | Thiscall constructor with three stack arguments (owner, default port, and a fourth argument passed to 010d90b0). It runs the base constructor 010d7a70, stores … |
| `010d6bc0` | ClientServerCore::BaseApi::ConnectStoredAddress | confirmed | Virtual, slot [7] of BaseApi and the sibling API vtables. Stores the argument byte at this+0x324 (the byte that DisconnectRequest clears and AutoReconnectTick … |
| `010d6be0` | ClientServerCore::BaseApi::ConnectRequest | confirmed | Thiscall with the address string, the timeout and the auto-reconnect argument. When the owner at +0x1f0 has its log flag set it logs 'BaseApi connect request (… |
| `010d6c50` | ClientServerCore::BaseApi::OnRoutePacket | confirmed | Virtual at slot +0xc0[1] of BaseApi. Calls the hook object at +0x2c4 through its vtable slot +4 with the length argument, takes a 64-bit tick, calls 010d9800 (… |
| `010d7270` | ClientServerCore::BaseApi::OnConnectionTerminated | confirmed | Virtual at slot +0xc0[3] of BaseApi (the handler subobject, owner at this+0x130). It refreshes the status text at this+0x268 through 010d6df0 and copies the dw… |
| `010d5d90` | ClientServerCore::BaseApi::ServiceUntilTimeout | likely | Virtual, slot [12] of BaseApi. Reads the 64-bit tick (007027f0) and loops while the state probe at slot +0x28 is true: each pass compares the elapsed milliseco… |
| `010d6ef0` | ClientServerCore::BaseApi::PublishUdpStats | likely | Thiscall with the sink pointer and a prefix string. When the owner at +0x1f0 is present it formats 'PREFIX/Udp/<metric>' names with 00402fd0 for incoming and o… |
| `010d7650` | ClientServerCore::BaseApi::BaseApiVariant | likely | Thiscall constructor with six or more stack arguments. Runs the base constructor 010d7a70 (UdpCompressionHandler), installs the UdpConnectionHandler, RpcManage… |

**ClientServerCore::RpcManagerHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d5800` | ClientServerCore::RpcManagerHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores ClientServerCore::RpcManagerHandler::vftable at this, calls _free (00d82463) on this when bit 0 of the argument is set, and … |
| `010d57f0` | ClientServerCore::RpcManagerHandler::~RpcManagerHandler | likely | Non-deleting destructor body. Stores ClientServerCore::RpcManagerHandler::vftable at this and returns; it frees nothing. |

**ClientServerCrypto::CryptoBaseApi** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d33450` | CryptoBaseApi::DeletingDtorAdjustor | confirmed | Subtracts 0xc0 from ECX and jumps to 00d33490, adjusting the receiver before the shared deleting destructor. |
| `00d33460` | CryptoBaseApi::DeletingDtorAdjustor | confirmed | Subtracts 0xc4 from ECX and jumps to 00d33490, adjusting the receiver before the shared deleting destructor. |
| `00d33490` | CryptoBaseApi::ScalarDeletingDtor | confirmed | Calls the CryptoBaseApi destructor, frees this when flag bit zero is set, and returns this. |
| `00d336f0` | CryptoBaseApi::InternalSendSecure | confirmed | Checks provider readiness, wraps or encrypts the supplied buffer, passes the result to the transport helper, and logs when wrapping fails. |
| `00d33900` | CryptoBaseApi::OnRoutePacket | confirmed | Attempts to unwrap the input through the current provider, dispatches decoded bytes to a shared route handler, and falls back to the marker-based wrapper path … |
| `00d33270` | CryptoBaseApi::CryptoBaseApi | likely | Installs the CryptoBaseApi primary and secondary vtables, then calls the shared crypto initialization helper. |
| `00d335d0` | CryptoBaseApi::ForwardBuffer | likely | Optionally wraps the supplied bytes through PacketUtils::WrapPacket, then passes the selected buffer to a shared transport helper or receiver virtual method. |

**PlayClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090d480` | PlayClient::OnDisconnect | confirmed | Displays ErrorDisconnected and ErrorLabelOops, invokes 00904560, and writes the disconnect sequence to NetInfo.log. |
| `0090d990` | PlayClient::HandleWorldShutdownNotice 🌐 | confirmed | Parses shutdown time and reason fields, chooses immediate or timed shutdown localization, formats remaining hours/minutes/seconds, and submits the notice to th… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00904170` | LoadSecurityRootStore | confirmed | Fastcall on this. Returns 1 if this+0x904 is already set. Once (flag DAT_01b3c264), loads './Resources/Security/RootStore' through FUN_00f90eb0 into DAT_01be48… |
| `0093bec0` | SendOneTimeSessionRequest 🌐 | confirmed | Constructs OneTimeSessionRequestPacket, copies a global dword into its payload, and sends through the tunneled client-world path when the gateway exists. |
| `0093bf50` | RequestChinaSnsSessionTicket 🌐 | confirmed | Logs the China SNS ticket request, constructs OneTimeSessionRequestPacket with a global value, and sends through tunneled client-world transport when available. |
| `0093d740` | BuildAndSendClientMetrics 🌐 | confirmed | Accumulates elapsed-time counters, refreshes timing and activity fields after 15000 ms, and sends the copied structure through the tunneled client-packet path. |
| `00d33b10` | WrapPacket | confirmed | On the marker path, processes the trailing wrapper marker; on the encryption path, invokes the provider, appends the marker to the transformed bytes, and clear… |
| `00904560` | OnDisconnectTeardown | likely | Thiscall with two stack args (param_2, param_3). If the byte at this+0x8e is nonzero it calls FUN_00ad3740 for this+0x104 and this+0x108 when set, FUN_009aae90… |
| `00bf92d0` | TunneledClientPacketBuild | likely | Initializes a packet writer through 008d4a40 with flag 0x10000000, copies a byte run (minimum 2) from the 16-bit value at this+0x4 through 007cc400, constructs… |
| `00d30040` | InitializeLimitsForMode | likely | Initializes fixed-string state and numeric limits, then overrides selected fields for modes 0 through 3. |
| `00d333c0` | ReplaceApiProvider | likely | Destroys the current provider when present, creates a provider from the supplied kind, stores it, and calls its initialization virtual with the remaining argum… |
| `0108d3a0` | HttpStatusToError | likely | Switch on the value in EAX (an HTTP status): 400 returns -10, 401 (0x191) returns -12, 403 (0x193) returns -13, 500 returns -14, 501 (0x1f5) returns -15, 503 (… |
| `010d63e0` | BaseApiConnectionCompletedCallback | likely | Callback on a handler object: when the object at this+0x130 has its log flag at +0x2a8 set it logs 'BaseApi connection completed (%s) address=%s' through 00704… |

### `net.compression`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ebc10` | CompressMagicFrame | confirmed | Returns at once when the output capacity param_4 is below 12. When param_6 is nonzero it writes the bytes a1 b2 c3 d4, then param_2 as a big-endian dword, and … |
| `010ebd70` | ReadMagicFrameLength | confirmed | If the input length is above 8 and bytes 0..3 are a1 b2 c3 d4 (code tests == -0x5e4d3c2c), it returns the big-endian dword at bytes 4..7 (the declared output l… |
| `010ebdc0` | InflateMagicFrame | confirmed | Does nothing when the input length is not positive. Reads the header with ReadMagicFrameLength (010ebd70). When a header is present and the declared length is … |
| `010ebee0` | InflateMagicFrameToBuffer | confirmed | param_3 is a buffer object: data pointer at +4, length at +8, capacity at +0xc. When the input has an A1B2C3D4 header with declared length L (010ebd70 >= 0), i… |

### `net.dns`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fc86f0` | ResolveHostIPv4String | confirmed | Plain function, no arguments, returns nothing. Calls WSAStartup(0x101), converts a 256-char host string produced by 006ffcd0 to wide, calls GetAddrInfoW with h… |

### `net.gateway`

**BaseClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fecb0` | ConnectGateway | confirmed | fastcall (ECX=client, EH frame and /GS cookie). Logs the local client address ("Local client address is %s"). Allocates an 0xbf8-byte gateway connection object… |
| `008ff000` | BaseClient::ShowOrLogError | likely | thiscall with a flag byte, a dword code and a message pointer (ret 0xc in the pseudocode's view). Replaces a null message with the string "Unknown error". If t… |

**ClientGateway::BasePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c04670` | WrapInGamePurchasePacketForGateway 🌐 | confirmed | Serializes the in-game purchase packet into a temporary buffer, constructs a tunneled client world packet, adds param_2, and destroys the temporary packet writ… |

**Engine::FRTWNetworkService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013df1e0` | Engine::FRTWNetworkService::Ctor | confirmed | Calls the NetworkServiceBase constructor 013df2c0 on this, stores the Engine::FRTWNetworkService vftable 018e25f8 at [this], clears the byte at [this+8], and r… |
| `013df240` | Engine::FRTWNetworkService::Dtor | confirmed | Stores the FRTW network service vftable 018e25f8 at [this], clears the byte at [this+8], then calls the NetworkServiceBase destructor body 013df320. |
| `013df2a0` | Engine::FRTWNetworkService::ScalarDeletingDtor | confirmed | Calls the destructor body 013df240, then frees this through _free (00d82463) only when bit 0 of the flag argument is set, and returns this. |

**Engine::NetworkServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013df2c0` | Engine::NetworkServiceBase::Ctor | confirmed | Calls the base constructor 01230d10 on this, stores the Engine::NetworkServiceBase vftable 018e2618 at [this], and returns this. |
| `013df320` | Engine::NetworkServiceBase::Dtor | confirmed | Stores the NetworkServiceBase vftable 018e2618 at [this] and then calls the base destructor body 01230d30; no return value. |
| `013df4a0` | Engine::NetworkServiceBase::ScalarDeletingDtor | confirmed | Stores the NetworkServiceBase vftable 018e2618 at [this], calls the base destructor body 01230d30, then frees this through _free (00d82463) only when bit 0 of … |

**GatewayCommand** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d6ab0` | GatewayCommand::GatewayCommand | confirmed | Constructor body: calls 0140c090 (the base-chain setup shared with GuildCommand), installs GatewayCommand::vftable and returns this. |
| `014d6b30` | GatewayCommand::GetName | confirmed | Returns the address of the string literal NetworkCommand (mov eax, imm32; ret), not GatewayCommand. |
| `014d6b70` | GatewayCommand::DtorBody | confirmed | Sets the vtable to GatewayCommand::vftable and calls 0140c160 (the destructor chain shared with GuildCommand::DtorBody) and returns. |
| `014d6bd0` | GatewayCommand::Clone | confirmed | Allocates 4 bytes (00d833a6). When that succeeds it constructs the new object with 014d6ab0. It then calls virtual slot [18] (0x48) on this with the new pointe… |
| `014d6c40` | GatewayCommand::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GatewayCommand (00d8d382); on failure it reports the clone assertion (012f5a60, line 0x1f). It then calls the… |
| `014d6c90` | GatewayCommand::Deserialize | confirmed | Reads the class header through vtable slot 4 (0x10) with an out-pointer, calls 0140c2a0 on the buffer and returns 0 when either step fails. Otherwise it calls … |
| `014d6cd0` | GatewayCommand::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer (header write), calls 013c26e0 (no-op returning 1) with Starting GatewayCommand, calls 0140c2d0 on the buffer, calls… |
| `014d6da0` | GatewayCommand::ScalarDeletingDtor | confirmed | Sets the vtable to GatewayCommand::vftable, calls 0140c160 (the destructor chain), then calls _free(this) only when bit 0 of the flag argument is set. Returns … |
| `014d6e00` | GatewayCommand::TraceFields | confirmed | Appends the text Gateway Command and a newline (01241650, length 0x10) to the output string, then calls the stub 0140c290 on it. It does not print the account … |
| `014d6e30` | GatewayCommand::TraceName | confirmed | Appends the 15-character text Gateway Command (01241650, length 0xf) to the output string and returns. |
| `014d6b40` | GatewayCommand::GetSharedDword | likely | Returns the dword in global DAT_01cbe244 (mov eax, [0x01cbe244]; ret). |
| `014d6b50` | GatewayCommand::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe244 (ret 4). |
| `014d6d20` | GatewayCommand::FactoryCreate | likely | Allocates 4 bytes (00d833a6) and, when the allocation is non-null, constructs a GatewayCommand with 014d6ab0. Returns the object, or 0 when the allocation fail… |
| `014d6d80` | GatewayCommand::RegisterFactory | likely | Pushes the creator 014d6d20 and the classID 0x19d and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GatewayCommand_GetConnectionServer** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ee700` | GatewayCommand_GetConnectionServer::GetClassName | confirmed | Virtual slot 11. Returns the address of the C string GatewayCommand_GetConnectionServer (6-byte body, mov eax, imm32; ret). |
| `013eea50` | GatewayCommand_GetConnectionServer::CloneInto | confirmed | Virtual slot 18. Dynamic-casts its target from NetworkCommand to GatewayCommand_GetConnectionServer (assert clone at line 0x74 on failure, source GatewayComman… |
| `013eeae0` | GatewayCommand_GetConnectionServer::DumpFields | confirmed | Virtual slot 19. Appends Get Connection Server Gateway Command, then calls 014d6e00 on its argument, then formats and appends the lines SessionID: %s (string a… |
| `013eeca0` | GatewayCommand_GetConnectionServer::AppendTitle | confirmed | Virtual slot 20. Calls 01241650 with the 37-character text Get Connection Server Gateway Command and returns. |
| `013ee5e0` | GatewayCommand_GetConnectionServer::WriteFields | likely | Virtual slot 10, thiscall with one argument. Calls this->vtable +8 with the argument, calls 014d6cd0, writes three strings with 013c2860 (fields at this+4, +0x… |
| `013ee650` | GatewayCommand_GetConnectionServer::GatewayCommand_GetConne… | likely | Constructor body: calls 014d6ab0 for base initialisation, stores the GatewayCommand_GetConnectionServer vftable at *this, initialises three empty std::string f… |
| `013ee740` | GatewayCommand_GetConnectionServer::~GatewayCommand_GetConn… | likely | Destructor body under an EH frame: stores the GatewayCommand_GetConnectionServer vftable at *this, frees the three std::string buffers at this+0x40, +0x24 and … |
| `013ee8f0` | GatewayCommand_GetConnectionServer::ScalarDeletingDtor | likely | Virtual slot 0 (bytes 56 8B F1 E8 .. test byte [esp+8],1 .. C2 04 00): calls the destructor body 013ee740, frees this when flag bit 0 is set, returns this. |

**HttpLibrary::HttpConnectionHandlerAdapter** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f992e0` | HttpConnectionHandlerAdapter::Ctor | confirmed | Thiscall constructor: stores HttpLibrary::HttpConnectionHandlerAdapter::vftable at +0, allocates 400 (0x190) bytes through operator new (00d833a6); on success … |
| `00f99670` | HttpConnectionHandlerAdapter::DtorBody | confirmed | Stores HttpLibrary::HttpConnectionHandlerAdapter::vftable at +0, calls 00f99220 on the reader at +4 and frees it through _free when non-null, then stores TcpLi… |
| `00f99810` | ConsumeReceived | confirmed | Virtual slot 1 of the adapter, thiscall with (data pointer, length): while length is non-zero, if the reader's +0x188 header-complete byte is clear it calls 00… |
| `00f998d0` | HttpConnectionHandlerAdapter::ScalarDeletingDtor | confirmed | Virtual slot 0 of the adapter: calls the dtor body 00f99670 on this, frees this through _free when bit 0 of the byte argument is set, and returns this (ret 4). |
| `00f98f50` | HttpConnectionHandlerAdapter_NotifyStateOne | likely | Virtual slot [2] of HttpLibrary::HttpConnectionHandlerAdapter (vtable 01879cd8): when the sink object at this+8 is non-null, calls its vtable slot +8 with the … |
| `00f98f90` | HttpConnectionHandlerAdapter_NotifyStateTwo | likely | Virtual slot [3] of HttpLibrary::HttpConnectionHandlerAdapter (vtable 01879cd8): when the sink object at this+8 is non-null, calls its vtable slot +0xc with th… |

**NetworkService** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f2730` | NetworkService::DtorBody | confirmed | Fastcall (ECX = this): stores NetworkService::vftable at +0, runs the logout 012f2650, closes through 012f0d30(1), calls WSACleanup, clears the singleton DAT_0… |
| `012f27f0` | NetworkService::ScalarDeletingDtor | confirmed | Thiscall with a flag byte: calls the destructor body 012f2730 and frees this only when flag bit 0 is set, then returns this (ret 4). |

**TcpLibrary::TcpConnectionHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d2edd0` | DtorBody | confirmed | Destructor body: stores the TcpConnectionHandler vptr at +0 and returns; no member cleanup. |

**(no class)** (83)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00935680` | BuildTunneledClientPacket 🌐 | confirmed | Allocates a large temporary buffer, serializes the supplied packet through FUN_00922570, constructs PacketTunneledClientPacket, copies the serialized length an… |
| `009c9410` | SendTunneledClientPacket 🌐 | confirmed | Serializes the supplied packet into a stack buffer, constructs PacketTunneledClientPacket, copies serialized bytes and flag into it, sends through the gateway … |
| `009e7c00` | GatewaySend_U16PairAndPayload 🌐 | confirmed | Serialises the two-u16 record header and tail payload into a local byte-array writer, constructs a type-5 PacketTunneledClientPacket, and passes that packet wi… |
| `009e7d70` | GatewaySend_FoundationRecord 🌐 | confirmed | Serialises the foundation record into a local byte-array writer, constructs a type-5 PacketTunneledClientPacket, and passes that packet with the caller type an… |
| `009e7e80` | GatewaySend_ToolRecord 🌐 | confirmed | Serialises the tool record into a local byte-array writer, constructs a type-5 PacketTunneledClientPacket, and passes that packet with the caller type and zero… |
| `009e7f90` | GatewaySend_HeaderOnly 🌐 | confirmed | Serialises the two-u16 header into a local byte-array writer, constructs a type-5 PacketTunneledClientPacket, and passes that packet with the caller type and z… |
| `009e80d0` | GatewaySend_CustomRecord 🌐 | confirmed | Serialises the record through FUN_0090ac70 into a local byte-array writer, constructs a type-5 PacketTunneledClientPacket, and passes that packet with the call… |
| `00a3dca0` | SendTunneledPacket 🌐 | confirmed | Initializes a 0x2048-byte stack writer with capacity argument 0x10000000, serializes the caller-supplied packet object through 00a3da40, constructs a local Cli… |
| `00acdf30` | SendPacketLogin 🌐 | confirmed | Constructs PacketLogin, assigns three string members and version 1.910.1.530630, calls the login sender, destroys the temporary object, and returns its result … |
| `00ace250` | SendClientIsReadyTunnel 🌐 | confirmed | Writes the inner packet's 16-bit id, constructs an outer tunneled client packet, submits its serialized bytes through 00925640, and frees the scratch buffer. |
| `00ace390` | SendClientIsReady 🌐 | confirmed | Constructs PacketClientIsReady, passes it to the tunnel sender with flag 1, and returns after that call. |
| `00be77b0` | SendTunneledClientWorldPacket 🌐 | confirmed | Initializes a stack packet writer through 008d4a40 (argument 0x10000000), serialises the dwords at the source object's +0x10 and +0x14 into it through 00be7010… |
| `00be78c0` | SendTunneledClientPacket 🌐 | confirmed | Initializes a stack packet writer through 008d4a40 (argument 0x10000000), serialises the dwords at the source object's +0x10 and +0x14 into it through 00be7010… |
| `00c22ad0` | WrapItemActionBarAssign 🌐 | confirmed | Serializes the InventoryPacketItemActionBarAssign body through 00c20a30, wraps it in ClientGateway::BasePacket, and cleans up the temporary writer. |
| `00cdbec0` | SerializeAndSend_cda880 🌐 | confirmed | Initializes a packet writer, serializes through 00cda880, constructs a local tunneled packet with ECX pointing to its stack object, sends through 00925640, the… |
| `00cdbfd0` | SerializeAndSend_cda990 🌐 | confirmed | Initializes a packet writer, serializes through 00cda990, constructs a local tunneled packet with ECX pointing to its stack object, sends through 00925640, the… |
| `00cdc0e0` | SerializeAndSend_cdaaa0 🌐 | confirmed | Initializes a packet writer, serializes through 00cdaaa0, constructs a local tunneled packet with ECX pointing to its stack object, sends through 00925640, the… |
| `00cdc1f0` | SerializeAndSend_cdab10 🌐 | confirmed | Initializes a packet writer, serializes through 00cdab10, constructs a local tunneled packet with ECX pointing to its stack object, sends through 00925640, the… |
| `00cdc390` | SerializeAndSend_cdb000 🌐 | confirmed | Initializes a packet writer, serializes through 00cdb000, constructs a local tunneled packet with ECX pointing to its stack object, sends through 00925640, the… |
| `00cde880` | SerializeAndSend_ImpulsePayload 🌐 | confirmed | Initializes the packet writer, serializes through 00cde6f0, constructs a local tunneled packet with ECX pointing to its stack object, sends through 00925640, t… |
| `00cffa70` | SerializeAndSendGatewayPacket 🌐 | confirmed | Initializes a packet writer, serializes the supplied packet, constructs a PacketTunneledClientPacket envelope, passes it to gateway helper 00925640, then destr… |
| `012f0820` | ConnectSocketNonBlocking | confirmed | Fastcall (ECX = connection object): builds a sockaddr from the object: family from the address record at this+0x48 (+8), the address bytes copied from the reco… |
| `00925740` | GatewaySendViaSerializer_009135c0 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 009135c0, constructs a PacketTunneledClientPacket envelope (0089adc0) from… |
| `00925850` | GatewaySendViaSerializer_00913620 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 00913620, constructs a PacketTunneledClientPacket envelope (0089adc0) from… |
| `00925960` | GatewaySendViaSerializer_009136e0 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 009136e0, constructs a PacketTunneledClientPacket envelope (0089adc0) from… |
| `00925a70` | GatewaySendViaSerializer_00913740 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 00913740, constructs a PacketTunneledClientPacket envelope (0089adc0) from… |
| `00925b80` | GatewaySendViaSerializer_009137a0 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 009137a0, constructs a PacketTunneledClientPacket envelope (0089adc0) from… |
| `00925c90` | GatewaySendViaSerializer_0090ac70 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 0090ac70 (two u16 fields), constructs a PacketTunneledClientPacket envelop… |
| `00925da0` | GatewaySendViaSerializer_009138e0 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 009138e0, constructs a PacketTunneledClientPacket envelope (0089adc0) from… |
| `00925ec0` | GatewaySendViaSerializer_0090ac70_Pair 🌐 | likely | Builds a 0x10000000-capacity writer (008d4a40), serializes the source object through 0090ac70, calls 008fb810 on the sub-objects at param_1+0x10 and param_1+0x… |
| `00acd6d0` | SerializeLoginPacketFields 🌐 | likely | Writes the 16-bit field at +4, then passes members at +8, +0x18, +0x20, and +0x30 to serialization helpers. |
| `00acd7a0` | SerializeHeaderAndBytes | likely | Writes the 16-bit field at +4 and byte at +8, then serializes the length/data members at +0x0c and +0x10. |
| `00acda20` | SendSerializedBuffer | likely | Initializes a temporary packet writer, prepends the supplied 16-bit value, then calls receiver vtable slot +0x50 or +0x54 with the resulting data and length; f… |
| `00acdb60` | SendPacketLogin 🌐 | likely | Initializes a temporary packet writer, serializes through 00acd6d0, dispatches through receiver slot +0x50 or +0x54, then frees the buffer. |
| `00acdc60` | SendSerializedBufferWithHeader | likely | Initializes a temporary packet writer, writes the supplied 16-bit field, dispatches its bytes through receiver slot +0x50 or +0x54, then frees the buffer. |
| `00acdda0` | SendGatewayPacketBytes | likely | Initializes a temporary writer, serializes through 00acd7a0, dispatches via receiver slot +0x50 or +0x54, and frees the writer buffer. |
| `00acdea0` | SendPacketLogout | likely | Checks receiver virtual slot +0x28; when true, constructs PacketLogout, serializes it, then calls receiver slot +0x30. |
| `00acdff0` | SendOnlineStatusRequest | likely | Constructs PacketOnlineStatusRequest, serializes its 16-bit id through 00acdc60, and sends it. |
| `00b1e4e0` | SendTunneledPacketFromRecord 🌐 | likely | Serializes the record at param_1 into a local 8 KiB buffer: FUN_00a0de80 writes its two u16 header fields, then FUN_008fd1a0 writes the u64 at record+0x10. Con… |
| `00b40b90` | BuildTunneledClientPayload | likely | Builds a tunneled client packet around the payload produced by 00b3ab20, copies the source reader's current data pointer and remaining length into the packet, … |
| `00b40da0` | BuildTunneledWorldPayload | likely | Builds a tunneled client-world packet from 00b36a10, copies its current data pointer and remaining length, stores the supplied byte flag, then releases the tem… |
| `00b40eb0` | BuildTunneledWorldPayloadWithString | likely | Builds a tunneled world packet from 00b36a10, serializes the string-like object at param_1+0x18 with 00911c30, copies the payload pointer and remaining length,… |
| `00b40fd0` | BuildTunneledWorldPayloadWithTwoStrings | likely | Builds a tunneled world packet from 00b36a10, serializes string-like fields at param_1+0x18 and +0x34, copies the payload pointer and remaining length, stores … |
| `00b41240` | BuildTunneledWorldPayloadWithOneString | likely | Builds a tunneled world packet from 00b36a10, serializes the string-like field at param_1+0x18, copies the payload pointer and remaining length, stores the cal… |
| `00b41360` | BuildTunneledWorldPayload | likely | Builds a tunneled world packet from 00b36a10, copies the payload pointer and remaining length, stores the caller-supplied flag byte, then releases the temporar… |
| `00bb11b0` | WriteU16U8AndU32 | likely | Serialises a source struct into the 8KB ClientGateway packet writer passed in param_2: first calls 00bb0e90 (which appends the u16 from source+4 and the byte f… |
| `00bb12e0` | WriteU32ListAndNested | likely | Serialises a source struct: appends the dword at source+0 with the writer append idiom, delegates to 00bb1260 for source+4 (dword at source+0x10 plus the node … |
| `00bb1350` | GatewaySend_Struct0fd0 🌐 | likely | Builds an 8KB writer (008d4a40 with 0x10000000), serialises the caller's 20-byte source struct into it through 00bb0fd0, constructs the tunneled packet object … |
| `00bb1460` | GatewaySend_Struct1070 🌐 | likely | Same build-and-send sequence as 00bb1350 but serialises the source through 00bb1070: creates the 8KB writer, writes the source fields, constructs the tunneled … |
| `00bb1570` | GatewaySend_Struct1110 🌐 | likely | Variant of the tunnel send sequence using 00bb1110 as the source-field writer (u16+byte+two dwords): builds the 8KB writer, serialises, constructs the tunneled… |
| `00bb1680` | GatewaySend_Struct11b0 🌐 | likely | Variant of the tunnel send sequence using 00bb11b0 as the source-field writer (u16+byte+u32): builds the 8KB writer, serialises, constructs the tunneled packet… |
| `00bb1820` | WriteHandleU32AndList | likely | Serialises a source struct into the packet writer (param_2): writes the u16 at source+4 and the byte at source+8 (00bb0e90), the 8-byte actor handle at source+… |
| `00bb1ba0` | GatewaySend_Struct1820 🌐 | likely | Variant of the tunnel send sequence using 00bb1820 as source writer (u16+byte, 8-byte handle at source+0xa8, dword/list block, dword at source+0xb0): builds th… |
| `00bd1a10` | SendTunneledGameTimeSync 🌐 | likely | Takes the packet object and a flag argument (passed as 1 by 00bd1b40). Initialises a 0x10000000-capacity writer through 008D4A40, serialises the packet with 00… |
| `00c22460` | WrapSerializedPacketBody | likely | Serializes the supplied object into a temporary writer, wraps its buffer and length in a ClientGateway::BasePacket object with the supplied selector, then dest… |
| `00c22570` | WrapSerializedPacketBody | likely | Serializes a leading U16 pair and fields beginning at source+0x10, wraps the resulting buffer in ClientGateway::BasePacket with the supplied selector, then cle… |
| `00c22690` | WrapSerializedWorldPacketBody | likely | Serializes fields through 00c21310, wraps the body using the world-packet constructor, sets the supplied selector, and destroys temporary storage. |
| `00c227a0` | WrapSerializedWorldPacketBody | likely | Serializes fields through 00c20930, wraps the body using the world-packet constructor, sets the supplied selector, and destroys temporary storage. |
| `00c228b0` | WrapSerializedWorldPacketBody | likely | Serializes fields through 00c20990, wraps the body using the world-packet constructor, sets the supplied selector, and destroys temporary storage. |
| `00c229c0` | WrapSerializedWorldPacketBody | likely | Serializes a leading U16 pair, wraps the bytes using the world-packet constructor, sets the supplied selector, and destroys temporary storage. |
| `00c22be0` | WrapSerializedPacketBody | likely | Serializes a U16 pair and U32 field via 00c20ad0, wraps the body in ClientGateway::BasePacket, sets the supplied selector, and releases temporary storage. |
| `00c22cf0` | WrapSerializedPacketBody | likely | Serializes a U16 pair and an additional source structure through 00911c30, wraps the resulting bytes in ClientGateway::BasePacket, and cleans up. |
| `00c22e10` | WrapSerializedPacketBody | likely | Serializes a U16 pair and an additional source structure through 00911c30, wraps the resulting bytes in ClientGateway::BasePacket, and cleans up. |
| `00c22f30` | WrapSerializedPacketBody | likely | Serializes a U16 pair, wraps the bytes in ClientGateway::BasePacket with the supplied selector, and destroys temporary storage. |
| `00c23150` | WrapSerializedPacketBody | likely | Serializes a U16 pair, wraps the bytes in ClientGateway::BasePacket with the supplied selector, and destroys temporary storage. |
| `00c23260` | WrapSerializedPacketBody | likely | Serializes fields through 00c21430, wraps them in ClientGateway::BasePacket with the supplied selector, and destroys temporary storage. |
| `00c23370` | WrapSerializedPacketBody | likely | Serializes fields through 00c214d0, wraps them in ClientGateway::BasePacket with the supplied selector, and destroys temporary storage. |
| `00c23480` | WrapSerializedPacketBody | likely | Serializes fields through 00c21570, wraps them in ClientGateway::BasePacket with the supplied selector, and destroys temporary storage. |
| `00c23ae0` | WrapLocalPairInWorldPacket | likely | Obtains two values from a local interface, appends two one-byte fields, conditionally calls 00c22690 with selector one, then destroys a temporary through 00c14… |
| `00c24670` | WrapSerializedWorldPacketBody | likely | Serializes the leading fields and structured payload through 00a0e8f0, then wraps them in a tunneled world-packet BasePacket using the supplied selector. |
| `00c24790` | WrapSerializedWorldPacketBody | likely | Serializes fields through 00c22400 and wraps the resulting body in a tunneled world-packet BasePacket with the supplied selector. |
| `00f99d70` | ParseHttpUrl | likely | Cdecl-style with (url string, a host string object, out port long, a path string object, out secure byte): measures the URL with a loop, requires a case-insens… |
| `012f05e0` | NetBufferAppend | likely | Stdcall with four stack args (ret 0x10): arg1 = int** buffer, arg2 = int* length, arg3 = source bytes, arg4 = count n. If the buffer pointer is null it mallocs… |
| `012f0640` | SocketRecvAppend | likely | Fastcall (ECX = connection object): ensures the receive buffer at this+0x50 has room for 0x1000 more bytes beyond the used length at this+0x54 (malloc or reall… |
| `012f06b0` | SocketFlushBufferA | likely | Fastcall (ECX = connection object). Returns 0 when buffer A (+0x58) is empty. Otherwise send()s the length at +0x5c bytes on the socket at +0x44. A nonnegative… |
| `012f0720` | SocketPollNoWait | likely | Fastcall (ECX = connection object). Runs select() with a zero timeout on the socket at +0x44 (read, write and exception sets). Returns 3 when nothing is ready.… |
| `012f0d30` | CloseConnectionSocket | likely | Thiscall with one char argument (keep second buffer flag). Calls 012d46a0 and 012d0900, then switches on the mode at this+0x68: modes 0 and 7 return immediatel… |
| `012f0de0` | DisconnectPostEvents | likely | One char argument: calls 012f0d30 with flag 1 (close socket and buffers), then 012f0c30(7) (mode 7), then 012d46a0. If the argument is zero it allocates a 0x14… |
| `012f1b80` | LoadHostSvrConfig | likely | Fastcall (ECX = NetworkService object). Opens the file host.svr for reading (fopen 'r'; returns when it cannot be opened). Reads it line by line into a 1024-by… |
| `012f1d90` | NetworkServiceGetOrCreate | likely | No arguments: if the global DAT_01cbd970 is null it allocates 0x74 bytes (00d833a6), constructs a NetworkService with 012f1890 (the NetworkService constructor)… |
| `012f2330` | NetworkServiceConnectDefault | likely | Thiscall-style (ECX = NetworkService object, fastcall with no args). Clears the flag at +0x4d and the resolved flag at +0x4c and the hostent at +0x48, closes t… |
| `012f23e0` | PumpConnectedIO | likely | Fastcall (ECX = connection object). Zero-timeout poll (012f0720): 2 closes through 012f0de0(0) and returns its result; 3 (idle) returns 1. Otherwise flushes bu… |
| `012f2470` | NetworkServiceTickByMode | likely | Fastcall (ECX = connection object). Switches on the mode at +0x68 and returns the result of: mode 0 -> 012f1e00 (ConnectStep); modes 1 and 4 -> 012f0ef0(0) (Se… |

### `net.packet`

**AchievementAddPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bab710` | AchievementAddPacket::AchievementAddPacket | confirmed | Calls the base achievement constructor with subtype 2, installs AchievementAddPacket::vftable, initializes an embedded object at +0x10 through FUN_0091c9f0, an… |
| `00bab770` | AchievementAddPacket::~AchievementAddPacket | confirmed | Calls the base packet cleanup, conditionally frees this object when flag bit 0 is set, and otherwise returns its pointer. |

**AchievementCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ba98c0` | AchievementCompletePacket::AchievementCompletePacket | confirmed | Calls the base achievement constructor with subtype 3, installs its packet vtable, stores two dwords and one byte from arguments, then returns this. |
| `00baa300` | AchievementCompletePacket::~AchievementCompletePacket | confirmed | Restores BasePacket::vftable and frees the object when flag bit 0 is set. |

**AchievementInitializePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bad510` | AchievementInitializePacket_Ctor | confirmed | Constructor: calls 00ba97d0 with argument 4 on ECX (base constructor, not resolved here), stores the AchievementInitializePacket vtable (01824bf4) and construc… |
| `00bad8d0` | AchievementInitializePacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00bad570 on ECX, then _free when the delete flag bit 0 is set, and returns the object p… |
| `00bad570` | AchievementInitializePacket_DtorBody | likely | Destructor body: calls 00bad2d0 (List<AchievementData,32> destructor body) on the member at this+0xc, then stores the BasePacket vtable (018011ec) into the obj… |

**AchievementObjectiveActivatedPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00baa360` | AchievementObjectiveActivatedPacket::~AchievementObjectiveA… | confirmed | Restores BasePacket::vftable and frees the object when flag bit 0 is set. |

**AchievementObjectiveAddedPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bab800` | AchievementObjectiveAddedPacket::~AchievementObjectiveAdded… | confirmed | Calls the base cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns this. |

**AchievementObjectiveCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00baa900` | AchievementObjectiveCompletePacket::AchievementObjectiveCom… | confirmed | Calls the base achievement constructor with subtype 8, stores three dwords, zeroes another dword, stores a byte, and returns this. |
| `00baa980` | AchievementObjectiveCompletePacket::~AchievementObjectiveCo… | confirmed | Restores BasePacket::vftable and frees the object when flag bit 0 is set. |

**AchievementObjectiveUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ba9b70` | AchievementObjectiveUpdatePacket::AchievementObjectiveUpdat… | confirmed | Calls the base constructor with subtype 7, stores four dwords at fields beginning +0xc, and returns this. |
| `00baa3c0` | AchievementObjectiveUpdatePacket::~AchievementObjectiveUpda… | confirmed | Restores BasePacket::vftable and conditionally frees the object when flag bit 0 is set. |

**AcquaintanceOnlinePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a12160` | AcquaintanceOnlinePacket_Ctor 🌐 | likely | Constructs the packet by calling FUN_00a10210(3) (acquaintance base with sub-opcode 3), installs AcquaintanceOnlinePacket::vftable, stores param_2 and param_3 … |
| `00a121d0` | AcquaintanceOnlinePacket_ScalarDeletingDtor | likely | Scalar deleting destructor: stores BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |

**AddAcquaintancePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a19170` | AddAcquaintancePacket::Ctor | likely | Constructs the acquaintance base packet, installs AddAcquaintancePacket vftable, initializes a pair field, and conditionally copies the supplied record fields … |
| `00a19220` | AddAcquaintancePacket::ScalarDeletingDtor | likely | Calls the BasePacket destructor 00a150f0, conditionally frees this object, and otherwise returns it. |

**AdminCommandPacketShipCombatCommand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a597f0` | AdminCommandPacketShipCombatCommand::ScalarDeletingDtor | confirmed | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |

**AdminHousingServerVersionPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0500` | AdminHousingServerVersionPacketScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |

**AdventurersJournalInfoPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a42790` | AdventurersJournalInfoPacket::AdventurersJournalInfoPacket 🌐 | confirmed | Constructor of the journal info message: runs the family base constructor FUN_00a426c0(1) (opcode 0xd1, sub-opcode 1 at +8), installs AdventurersJournalInfoPac… |
| `00a43170` | AdventurersJournalInfoPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of the journal info packet: installs BasePacket::vftable and frees the packet with _free when bit 0 of the delete fl… |
| `00a42800` | AdventurersJournalInfoPacket::~AdventurersJournalInfoPacket | likely | Destructor of the journal info packet: reinstalls BasePacket::vftable at +0, i.e. the packet owns nothing beyond its header and one payload dword. |

**AdventurersJournalQuestUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a42850` | AdventurersJournalQuestUpdatePacket::AdventurersJournalQues… 🌐 | confirmed | Constructor of the journal quest-update message: runs the family base constructor FUN_00a426c0(2) (opcode 0xd1, sub-opcode 2), installs AdventurersJournalQuest… |
| `00a431d0` | AdventurersJournalQuestUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of the journal quest-update packet: installs BasePacket::vftable and frees the packet with _free when bit 0 of the d… |
| `00a428c0` | AdventurersJournalQuestUpdatePacket::~AdventurersJournalQue… | likely | Destructor of the journal quest-update packet: reinstalls BasePacket::vftable at +0. |

**AnnouncementAdminSendAll** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3e100` | AnnouncementAdminSendAll::AnnouncementAdminSendAll | confirmed | Constructor of the sub-opcode 3 announcement packet: calls the family constructor 00a3df90 with argument 3 (packet id 0xc1, sub-opcode 3) and installs Announce… |
| `00a3e610` | AnnouncementAdminSendAll::ScalarDeletingDestructor | confirmed | Scalar deleting destructor of the sub-opcode 3 announcement packet: installs BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set. |

**AnnouncementBasePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3df30` | AnnouncementBasePacket::AnnouncementBasePacket | confirmed | Base constructor of the announcement packet family: installs BasePacket::vftable, stores the packet id 0xc1 (193) in the id slot +4, installs AnnouncementBaseP… |
| `00a3df90` | AnnouncementBasePacket::AnnouncementBasePacketSub | confirmed | Family constructor taking the sub-opcode: installs BasePacket::vftable, stores packet id 0xc1 (193) at +4, installs AnnouncementBasePacket::vftable and stores … |
| `00a3e550` | AnnouncementBasePacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor of the announcement family base: under an SEH frame installs BasePacket::vftable at +0 and calls _free(this) when bit 0 of the flag … |

**AnnouncementDataRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3e050` | AnnouncementDataRequestPacket::AnnouncementDataRequestPacket | confirmed | Constructor of the sub-opcode 1 announcement packet: calls the family constructor 00a3df90 with argument 1 (so packet id 0xc1 and sub-opcode 1 are stored) and … |
| `00a3e5b0` | AnnouncementDataRequestPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor of the sub-opcode 1 announcement packet: installs BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set. |

**AnnouncementDataSendPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a40460` | AnnouncementDataSendPacket::AnnouncementDataSendPacket 🌐 | confirmed | Constructor: runs the announcement base constructor FUN_00a3df90(2) (which installs BasePacket and AnnouncementBasePacket vtables, sets the base opcode 0xc1 at… |
| `00a404d0` | AnnouncementDataSendPacket::~AnnouncementDataSendPacket | confirmed | Installs AnnouncementDataSendPacket::vftable, removes list-head nodes through FUN_00a3f6d0 until the +0x10 head is empty, reinstalls the embedded list vtable a… |
| `00a40570` | AnnouncementDataSendPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of AnnouncementDataSendPacket: calls the destructor body 00a404d0 and frees the packet with _free when bit 0 of the … |

**AnnouncementDataSendPacket::AnnouncementPacketData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3eaf0` | EmbeddedStrings3_Dtor_18_28_44 | likely | Destructor of a record holding three SoeUtil::IString fields at +0x18, +0x28 and +0x44: each field gets the base IString vtable back and, when its length (+0x2… |
| `00a3ecc0` | EmbeddedStrings3_Ctor_18_28_44 | likely | Constructor of the three string fields at +0x18, +0x28 and +0x44: installs SoeUtil::IString<char>::vftable for each, points the buffers at the shared static em… |
| `00a3f0a0` | ScalarDelete_a3f0a0 | likely | Scalar deleting destructor under an SEH frame: calls the row destructor 00a3eaf0 and then calls _free(this) when bit 0 of the flag argument is set. |

**BaseAchievementPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ba97d0` | BaseAchievementPacket::BaseAchievementPacket | confirmed | Stores BasePacket::vftable, writes 0x96 at +4, installs BaseAchievementPacket::vftable, and stores the supplied subtype at +8. |
| `00baa2a0` | BaseAchievementPacket::~BaseAchievementPacket | confirmed | Restores BasePacket::vftable and frees the object only when flag bit 0 is set. |

**BaseAdventurersJournalPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a426c0` | BaseAdventurersJournalPacket::BaseAdventurersJournalPacket 🌐 | confirmed | Base constructor of the Adventurer's Journal packet family: installs BasePacket::vftable, stores the family opcode 0xd1 at +4, installs BaseAdventurersJournalP… |
| `00a43110` | BaseAdventurersJournalPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of the journal family base packet: installs BasePacket::vftable and frees the object with _free when bit 0 of the de… |
| `00a42730` | BaseAdventurersJournalPacket::~BaseAdventurersJournalPacket | likely | Base-part teardown of journal packets: reinstalls BasePacket::vftable at +0 and returns. |

**BaseBroadcastPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d1b30` | BaseBroadcastPacket::~BaseBroadcastPacket | confirmed | Destructor body of the broadcast-packet base: rewrites its own vtable to BaseBroadcastPacket::vftable, runs the SoeUtil::IStringFixed<char,256> destructor body… |
| `008d1bb0` | BaseBroadcastPacket::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (slot [0] of the broadcast-packet vtable): runs BaseBroadcastPacket::~BaseBroadcastPacket (008d1b30) and frees the object throu… |
| `008dd380` | BaseBroadcastPacket::BaseBroadcastPacket 🌐 | confirmed | Base packet constructor: installs BasePacket::vftable, stores family opcode 0x4d (77) at +4, installs BaseBroadcastPacket::vftable, clears sub-opcode +8, calls… |
| `008dd400` | BaseBroadcastPacket::BaseBroadcastPacket_SubOpcode 🌐 | confirmed | Base packet constructor taking a sub-opcode: installs BasePacket::vftable, stores family opcode 0x4d (77) at +4, installs BaseBroadcastPacket::vftable, stores … |

**BaseBugSubmissionPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f8870` | BaseBugSubmissionPacket::Dtor | confirmed | Resets the object's vtable first to BaseBugSubmissionPacket and then to BasePacket. |
| `009f88c0` | BaseBugSubmissionPacket::ScalarDeletingDtor | confirmed | Sets the derived and base packet vtables, conditionally frees the object when flag bit 0 is set, and returns its pointer otherwise. |
| `009f8800` | BaseBugSubmissionPacket::Ctor | likely | Installs BasePacket then BaseBugSubmissionPacket vtables, writes packet identifier 0x53 at +4, and stores the argument at +8. |

**BaseChatPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0d9a0` | BaseChatPacket::Ctor | confirmed | Installs BasePacket and BaseChatPacket vtables, sets packet type 15, and initializes the chat payload mode to zero. |
| `00b0da00` | BaseChatPacket::CtorWithMode | confirmed | Installs BasePacket and BaseChatPacket vtables, sets packet type 15, and stores the supplied mode in field 2. |
| `00b0da70` | BaseChatPacket::Dtor | confirmed | Restores BaseChatPacket and then BasePacket vtable pointers on the object. |
| `00b0dac0` | BaseChatPacket::ScalarDeletingDtor | confirmed | Restores BaseChatPacket and BasePacket vtables and frees the object when the low flag bit is set. |
| `00b0dba0` | BaseChatPacket::Dtor | confirmed | Restores BaseChatPacket and BasePacket vtable pointers on the object. |
| `00b0f1e0` | BaseChatPacket::DtorWithString | confirmed | Destroys a fixed 128-byte string and two common subobjects, then restores BaseChatPacket and BasePacket vtables. |

**BaseClaimPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ff350` | BaseClaimPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor inside an SEH frame: stores BaseClaimPacket::vftable, then BasePacket::vftable, then _free(this) when bit 0 of the flag is set. |
| `009ff290` | BaseClaimPacket::BaseClaimPacket | likely | Family constructor: stores BasePacket::vftable, writes the base opcode 0x6c (108) into the packet id slot param_1[1], stores BaseClaimPacket::vftable and then … |

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00923210` | BaseClient::HandleLobbyPacket | confirmed | Virtual slot [72] (shared by PlayClient and BaseClient): reads the lobby sub-opcode header through 008d90b0 from the bounded reader at param_2/param_3 and swit… |

**BaseClientUpdatePacket** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d120` | BaseClientUpdatePacketCtor 🌐 | confirmed | Constructor of BaseClientUpdatePacket for sub-opcode 0: installs BasePacket::vftable, stores the family opcode 0x26 (38) at +4, installs BaseClientUpdatePacket… |
| `0089d180` | BaseClientUpdatePacketCtorWithSubOpcode 🌐 | confirmed | Constructor of BaseClientUpdatePacket with an explicit sub-opcode: installs BasePacket::vftable, stores the family opcode 0x26 (38) at +4, installs BaseClientU… |
| `0089d1f0` | ClientUpdateBaseDtor_89d1f0 | confirmed | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; cited by 85 EH unwind funclets. |
| `0089d240` | BaseClientUpdatePacketScalarDtor | confirmed | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
| `0089d310` | ClientUpdateBaseDtor_89d310 | confirmed | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing a hitpoints update. |
| `0089d3c0` | ClientUpdateBaseDtor_89d3c0 | confirmed | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing a mana update. |
| `0089d590` | ClientUpdateBaseDtor_89d590 | confirmed | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used for the update packet between ItemAdd and Storage. |
| `0089d670` | ClientUpdateBaseDtor_89d670 | confirmed | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing the storage update. |
| `0089d750` | ClientUpdateBaseDtor_89d750 | confirmed | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing the item-delete update. |

**BaseCommandPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9af0` | BaseCommandPacket::~BaseCommandPacket | likely | Trivial destructor body: rewrites the vtables to BaseCommandPacket::vftable then BasePacket::vftable. Called by four EH unwind funclets; it is also the address… |

**BaseFirstTimeEventPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c880b0` | BaseFirstTimeEventPacketCtor | confirmed | Installs BasePacket then BaseFirstTimeEventPacket vtables, stores family opcode 0x6b, and saves the supplied subtype. |
| `00c885d0` | BaseFirstTimeEventPacketScalarDeletingDtor | confirmed | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**BaseFotomatPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad31f0` | BaseFotomatPacket::Dtor | confirmed | Restores BaseFotomatPacket vtable then BasePacket vtable and returns. |

**BaseFriendPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7d870` | BaseFriendPacket::Construct | confirmed | Installs BasePacket then BaseFriendPacket vtables, stores family id 0x4a and the supplied friend sub-opcode at object+8. |
| `00c7dec0` | BaseFriendPacket::DeletingDtor | confirmed | Restores BasePacket::vftable, frees the object if flag bit 0 is set, and otherwise returns the original pointer. |

**BaseIgnorePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7d9e0` | BaseIgnorePacket::Construct | confirmed | Installs BasePacket then BaseIgnorePacket vtables, stores family id 0x70 and the supplied ignore sub-opcode at object+8. |
| `00c7e0c0` | BaseIgnorePacket::DeletingDtor | confirmed | Restores BasePacket::vftable, conditionally frees the object when flag bit 0 is set, and otherwise returns the pointer. |

**BaseInspectPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b76b0` | BaseInspectPacket::ScalarDeletingDestructor | likely | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**BaseInventoryPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a10400` | BaseInventoryPacket_Ctor 🌐 | confirmed | Constructor: stores BasePacket::vftable, writes the family opcode 0x2a (42) into the packet opcode field at +4, installs BaseInventoryPacket::vftable and store… |
| `00a10600` | BaseInventoryPacket_Dtor | likely | Destructor body: stores BaseInventoryPacket::vftable then BasePacket::vftable; no fields released. |
| `00a12380` | BaseInventoryPacket_ScalarDeletingDtor | likely | Scalar deleting destructor: stores BaseInventoryPacket::vftable then BasePacket::vftable and frees the object when bit 0 of param_2 is set. |

**BasePacket** (90)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089af50` | BasePacketDtor_89af50 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; cited by 2316 EH unwind funclets, making it the single most referenced cleanup stub in this shar… |
| `0089af60` | BasePacketScalarDtor | confirmed | Scalar deleting destructor of BasePacket: stores the class vtable at +0 and calls _free when bit 0 of param_2 is set. |
| `0089b370` | BasePacketDtor_89b370 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; cited by 8 EH unwind funclets. |
| `0089b3c0` | BasePacketDtor_89b3c0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; cited by 1 EH unwind funclet. |
| `0089b410` | BasePacketDtor_89b410 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; cited by 3 EH unwind funclets. |
| `0089b4c0` | BasePacketDtor_89b4c0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089b690` | BasePacketDtor_89b690 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089b750` | BasePacketDtor_89b750 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089b7b0` | BasePacketDtor_89b7b0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called from the packet factory FUN_009471a0 and the timing manager 00942c80, cited by 2 EH unwin… |
| `0089b890` | BasePacketDtor_89b890 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; cited by 1 EH unwind funclet. |
| `0089b9a0` | BasePacketDtor_89b9a0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089ba60` | BasePacketDtor_89ba60 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089bb10` | BasePacketDtor_89bb10 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089bbd0` | BasePacketDtor_89bbd0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; cited by 1 EH unwind funclet. |
| `0089bc90` | BasePacketDtor_89bc90 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089bd40` | BasePacketDtor_89bd40 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089bdf0` | BasePacketDtor_89bdf0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089bea0` | BasePacketDtor_89bea0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089bf50` | BasePacketDtor_89bf50 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089c000` | BasePacketDtor_89c000 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the initialization-details path 00938a50 and cited by 1 EH unwind funclet. |
| `0089c0b0` | BasePacketDtor_89c0b0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089c160` | BasePacketDtor_89c160 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089c1b0` | BasePacketDtor_89c1b0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089c260` | BasePacketDtor_89c260 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089c330` | BasePacketDtor_89c330 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and cited by 1 EH unwind funclet. |
| `0089c3f0` | BasePacketDtor_89c3f0 | confirmed | Trivial destructor body resetting the BasePacket vtable at +0; called by the packet factory FUN_009471a0 and by 00cece40, cited by 2 EH unwind funclets. |
| `008eba90` | BasePacketDtorWithVectorList | confirmed | Destroys embedded Vector4 list and restores BasePacket vtable. |
| `0090bca0` | BasePacketSubobjectCleanup | confirmed | Calls 0090ba10 to destroy the Loggable subobject and then restores BasePacket::vftable. |
| `009531b0` | BasePacket::DtorBody | confirmed | Fastcall dtor body (ECX = this, EH frame): destroys five members through 00703350 in unwind order, then stores BasePacket::vftable at [this]. |
| `009c6ca0` | BasePacketDtor | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `009c6d50` | BasePacketDtorAlt | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `00ad9690` | BasePacket::~BasePacket | confirmed | Destructor body that only resets the object's vtable pointer to BasePacket::vftable and returns (67 bytes including the SEH prolog); called from 33 EH unwind f… |
| `00ad9990` | BasePacket::~BasePacket | confirmed | Destructor body that resets the vtable pointer to BasePacket::vftable and returns; reached from two EH unwind funclets only. |
| `00ad9ac0` | BasePacket::~BasePacket | confirmed | Destructor body that only rewrites the vtable pointer to BasePacket::vftable and returns; called from 18 EH unwind funclets. |
| `00ad9b10` | BasePacket::~BasePacket | confirmed | Destructor body that sets the vtable pointer to BasePacket::vftable and returns (bytes c7 01 ec 11 80 01, SEH prolog); a single EH unwind funclet calls it. |
| `00ad9bc0` | BasePacket::~BasePacket | confirmed | Destructor body that rewrites the vtable pointer to BasePacket::vftable and returns; a single EH unwind funclet calls it. |
| `00ad9c10` | BasePacket::~BasePacket | confirmed | Destructor body that rewrites the vtable pointer to BasePacket::vftable and returns; a single EH unwind funclet calls it. |
| `00adcdd0` | BasePacket::~BasePacket | confirmed | Destructor body that only rewrites the vtable pointer to BasePacket::vftable and returns; called from 16 EH unwind funclets. |
| `00adce80` | BasePacket::~BasePacket | confirmed | Destructor body that rewrites the vtable pointer to BasePacket::vftable and returns; a single EH unwind funclet calls it. |
| `00adcf30` | BasePacket::~BasePacket | confirmed | Destructor body that rewrites the vtable pointer to BasePacket::vftable and returns; a single EH unwind funclet calls it. |
| `00adcfe0` | BasePacket::~BasePacket | confirmed | Destructor body that rewrites the vtable pointer to BasePacket::vftable and returns; a single EH unwind funclet calls it. |
| `00b1f940` | BasePacketVftableCleanup_00b1f940 | confirmed | SEH-framed cleanup stores BasePacket::vftable at [param_1]; its bytes include handler prologue, ExceptionList save, vtable store and epilogue. Reached from EH … |
| `00b1fad0` | BasePacketVftableCleanup_00b1fad0 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] under its own SEH frame; reached only from an EH unwind funclet. |
| `00b1fd00` | BasePacketVftableCleanup_00b1fd00 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] under its own SEH frame; reached only from an EH unwind funclet. |
| `00b200b0` | BasePacketVftableCleanup_00b200b0 | confirmed | SEH-framed cleanup stores BasePacket::vftable at [param_1], restores the saved exception chain, then returns. The full function span is 67 bytes. |
| `00b20290` | BasePacketVftableCleanup_00b20290 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] inside its own SEH frame; called from the real function 00b48700 and from an EH unwind funclet, so it is ano… |
| `00b202e0` | BasePacketVftableCleanup_00b202e0 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] inside its own SEH frame; reached only from an EH unwind funclet. |
| `00b20330` | BasePacketVftableCleanup_00b20330 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] inside its own SEH frame; reached only from an EH unwind funclet. |
| `00b20380` | BasePacketVftableCleanup_00b20380 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] inside its own SEH frame; reached only from an EH unwind funclet. |
| `00b203d0` | BasePacketVftableCleanup_00b203d0 | confirmed | Stores BasePacket::vftable (018011ec) at [param_1] inside its own SEH frame; called from the real function 00b48370 and from an EH unwind funclet. |
| `00ba9840` | BasePacket::~BasePacket | confirmed | Replaces the object's first word with BasePacket::vftable and returns. |
| `00ba9930` | BasePacket::~BasePacket | confirmed | Stores BasePacket::vftable at the object's first word and returns. |
| `00ba9ad0` | BasePacket::~BasePacket | confirmed | Stores BasePacket::vftable at the object's first word and returns. |
| `00ba9bf0` | BasePacket::~BasePacket | confirmed | Stores BasePacket::vftable at the object's first word and returns. |
| `00ba9c80` | BasePacket::~BasePacket | confirmed | Stores BasePacket::vftable at the object's first word and returns. |
| `00bd12b0` | BasePacket::DtorBody | confirmed | Destructor body: stores the BasePacket vftable at this+0 and returns. No free; it runs from an EH unwind funclet. |
| `00bf7d30` | BasePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor with the base body inlined: stores BasePacket::vftable into the object, frees it through _free when bit 0 of the stack argument is s… |
| `00bff3f0` | BasePacket::DtorBody | confirmed | Replaces the packet object's vtable pointer with BasePacket::vftable and returns. |
| `00c810b0` | BasePacket::Dtor | confirmed | Removes all FriendData nodes from the embedded list, restores its list vtable, then restores BasePacket::vftable. |
| `00c88120` | BasePacketDtor | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `00c881f0` | BasePacketDtor | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `00c882b0` | BasePacketDtor | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `00c88390` | BasePacketDtor | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `00c88450` | BasePacketDtor | confirmed | Restores the BasePacket vtable at object offset zero and returns. |
| `00c8a320` | BasePacket::Dtor | confirmed | Restores the BasePacket vtable in the object; the body performs no other cleanup. |
| `00c8a3e0` | BasePacket::Dtor | confirmed | Restores the BasePacket vtable in the object and returns. |
| `00c8a4b0` | BasePacket::Dtor | confirmed | Restores the BasePacket vtable and returns without other cleanup. |
| `00c8b710` | BasePacket::Dtor | confirmed | Destroys embedded string state through FUN_00909b90, then restores the BasePacket vtable. |
| `00ccdbe0` | BasePacket::Dtor | confirmed | Restores the BasePacket vtable at the start of the object and returns without releasing additional fields. |
| `00cff840` | BasePacket::~BasePacket | confirmed | Restores the BasePacket vtable at the supplied object and returns. |
| `0089fa30` | BasePacket::~BasePacket | likely | Root destructor body: writes BasePacket::vftable into the object and returns. No field is read or released; the 67 bytes are EH prologue and frame setup around… |
| `0089fa80` | BasePacket::~BasePacket | likely | Root destructor body: writes BasePacket::vftable into the object and returns; nothing else is read, stored or called. |
| `008a0040` | BasePacket::~BasePacket | likely | Root destructor body: writes BasePacket::vftable into this and returns; no other field or call. Exactly one real caller, the EH unwind funclet at 01591170. |
| `00953270` | BasePacket::DtorBodyPartial | likely | Fastcall dtor body (ECX = this, EH frame): destroys two members through 00703350 and then stores BasePacket::vftable at [this]. |
| `009b31c0` | BasePacket::Dtor | likely | Releases two reference-counted string-like members through FUN_0103d010, invokes FUN_00728070 cleanup, then restores BasePacket vtable. |
| `00a10280` | BasePacket_Ctor | likely | Base constructor: stores BasePacket::vftable into this and nothing else. |
| `00a102e0` | BasePacket_Ctor2 | likely | Second byte-identical copy of the BasePacket base constructor: stores BasePacket::vftable and returns. |
| `00a120f0` | BasePacket_ScalarDeletingDtor2 | likely | Scalar deleting destructor for a plain BasePacket: stores BasePacket::vftable, then frees the object with _free when bit 0 of param_2 is set. |
| `00a16450` | BasePacket_ScalarDeletingDtor | likely | Scalar deleting destructor: runs the base packet member teardown FUN_008ca450, which resets the IString at +8 and releases its buffer, then frees the object wi… |
| `00a3e160` | BasePacket_DtorBody_00a3e160 | likely | Destructor body: rewrites BasePacket::vftable at +0 and returns, with no member release. |
| `00a8f980` | BasePacketStringListDestructor | likely | Destroys the owned string list, restores the IString vtable and releases its owned buffer, then restores the BasePacket vtable. |
| `00b43290` | BasePacket_DtorBody | likely | Calls 00b42b90 to destroy the embedded bundle-content request list and then restores BasePacket::vftable on the enclosing object. |
| `00bafed0` | BasePacket_DtorBody | likely | Destructor body: stores the BasePacket vtable (018011ec) into the object and returns; called from two EH unwind funclets. |
| `00c7d8e0` | BasePacket::Destruct | likely | Replaces the object's vtable pointer with BasePacket::vftable and returns; the function has many EH-cleanup callers. |
| `00c7d940` | BasePacket::Destruct | likely | Replaces the object's vtable pointer with BasePacket::vftable and returns; it is also called from an EH unwind funclet. |
| `00c7d990` | BasePacket::Destruct | likely | Replaces the object's vtable pointer with BasePacket::vftable and returns; one constructor unwind path calls it. |
| `00c7da50` | BasePacket::Destruct | likely | Replaces the object's vtable pointer with BasePacket::vftable and returns; EH cleanup calls reference this function. |
| `00c7dab0` | BasePacket::Destruct | likely | Replaces the object's vtable pointer with BasePacket::vftable and returns; caller 00c81250 uses it in cleanup. |
| `00c7edd0` | BasePacket::StringMembersDtor | likely | Calls FUN_0070a790 on embedded member storage, then restores BasePacket vtable and EH state. |
| `00c7ee30` | BasePacket::StringMembersDtor | likely | Calls FUN_0070a790 twice on embedded member storage, then restores BasePacket vtable and EH state. |

**BasePromotionalPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a057e0` | BasePromotionalPacket::BasePromotionalPacket 🌐 | confirmed | Family constructor: stores BasePacket::vftable, writes the base opcode 0x72 (114) into the packet id slot param_1[1], stores BasePromotionalPacket::vftable and… |
| `00a058a0` | BasePromotionalPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor in an SEH frame: restores BasePromotionalPacket::vftable then BasePacket::vftable and frees this when bit 0 of the flag is set. |
| `00a05850` | BasePromotionalPacket::~BasePromotionalPacket | likely | Destructor body: rewrites this->vtable to BasePromotionalPacket::vftable and then BasePacket::vftable; called from nine unwind funclets. |

**BaseRatingPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c81880` | BaseRatingPacket::BaseRatingPacket | confirmed | Installs BasePacket then BaseRatingPacket vtables, stores family value 0xae at field 1, and zeros a field at index 2. |
| `00c818e0` | BaseRatingPacket::Dtor | confirmed | Restores BaseRatingPacket::vftable and then BasePacket::vftable. |
| `00c85620` | BaseRatingPacketDtor | confirmed | Destroys the contained rating map, releases the reference-counted string when owned, and restores BaseRatingPacket then BasePacket vtables. |

**BaseRecipePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8a2b0` | BaseRecipePacket::BaseRecipePacket | confirmed | Sets BasePacket then BaseRecipePacket vtable, writes discriminator 0x3f, and stores the supplied packet type at field +8. |
| `00c8a8d0` | BaseRecipePacket::ScalarDeletingDtor | confirmed | Restores the BasePacket vtable and frees the object when deleting flag bit 0 is set. |

**BroadcastPacketWorld** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd480` | BroadcastPacketWorld::BroadcastPacketWorld 🌐 | confirmed | Packet constructor: calls the base broadcast constructor 008dd400 with sub-opcode 3, installs BroadcastPacketWorld::vftable, zeroes +0x1ac/+0x1b0, sets the byt… |
| `008dd4f0` | BroadcastPacketWorld::ScalarDeletingDtor | confirmed | SEH-framed scalar deleting destructor: runs the destructor body 008d1b30 and frees the object with _free (00d82463) when the low bit of the flag argument is se… |

**BugSubmissionPacketAddBug** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f9eb0` | BugSubmissionPacketAddBug::Ctor | confirmed | Initializes BaseBugSubmissionPacket with argument 1, installs BugSubmissionPacketAddBug vtable, and initializes two string subobjects with shared empty buffers. |
| `009fa330` | BugSubmissionPacketAddBug::ScalarDeletingDtor | confirmed | Calls the packet destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
| `009f9f40` | BugSubmissionPacketAddBug::Dtor | likely | For the string subobject at +0x24, gates cleanup on length +0x30>0; for the one at +0x10, gates on +0x1c>0. Each path decrements its buffer count and calls vir… |

**ChatPacketDebugChat** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0f150` | ChatPacketDebugChat::Dtor | confirmed | Restores ChatPacketDebugChat and base packet vtables, cleaning up the debug-chat payload member. |
| `00b0f1c0` | ChatPacketDebugChat::ScalarDeletingDtor | confirmed | Calls the concrete debug-chat destructor and frees the object when the low flag bit is set. |
| `00b0f9d0` | ChatPacketDebugChat::Ctor 🌐 | likely | Initializes BaseChatPacket mode 3, installs the concrete vtable, initializes a subobject, and clears a trailing boolean. |

**ChatPacketEnterArea** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0db30` | ChatPacketEnterArea::Ctor | confirmed | Initializes BaseChatPacket mode 2, installs the concrete vtable, and stores the argument in field 3. |
| `00b0e080` | ChatPacketEnterArea::ScalarDeletingDtor | confirmed | Restores BaseChatPacket and BasePacket vtables and frees the object when the low flag bit is set. |

**ChatPacketFromStringId** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0dbf0` | ChatPacketFromStringId::Dtor | confirmed | Restores ChatPacketFromStringId, BaseChatPacket, and BasePacket vtables in sequence. |
| `00b0dc50` | ChatPacketFromStringId::ScalarDeletingDtor | confirmed | Restores the concrete and base packet vtables and frees the object when the low flag bit is set. |
| `00b0e0f0` | ChatPacketFromStringId::Ctor | confirmed | Initializes BaseChatPacket mode 4, installs the concrete vtable, and initializes string and scalar payload fields from defaults. |

**CinematicStartLookAtPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a88200` | CinematicStartLookAtPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of CinematicStartLookAtPacket: calls 00a86ec0 (dtor body that destroys its two embedded Target members), then calls _free(this) thro… |

**ClaimItemsItemDefinitionsRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a01ee0` | ClaimItemsItemDefinitionsRequest::~ClaimItemsItemDefinition… | confirmed | Destructor body: runs the member teardown 00762170, then rewrites the vtable to BaseClaimPacket::vftable and BasePacket::vftable. |
| `00a02550` | ClaimItemsItemDefinitionsRequest::ClaimItemsItemDefinitions… 🌐 | confirmed | Constructor: runs the base constructor 009ff290 with sub-opcode 5, stores ClaimItemsItemDefinitionsRequest::vftable, stores the sentinel id DAT_01be63fc at +0x… |
| `00a025c0` | ClaimItemsItemDefinitionsRequest::ScalarDeletingDestructor | likely | Scalar deleting destructor: runs the destructor body 00a01ee0 and frees this when bit 0 of the flag is set. |

**ClaimItemsItemDefinitionsResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a00570` | ClaimItemsItemDefinitionsResponse::ScalarDeletingDestructor | confirmed | Scalar deleting destructor in an SEH frame: stores BaseClaimPacket::vftable, then BasePacket::vftable, then calls _free(this) when bit 0 of the flag is set. It… |
| `00a00500` | ClaimItemsItemDefinitionsResponse::ClaimItemsItemDefinition… 🌐 | likely | Constructor: runs the base constructor 009ff290 with sub-opcode 6, stores ClaimItemsItemDefinitionsResponse::vftable, stores DAT_01be63fc at +0xc, clears the b… |

**ClaimItemsRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ff630` | ClaimItemsRequestPacket::~ClaimItemsRequestPacket | confirmed | Destructor body: rewrites the vtable to ClaimItemsRequestPacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable; called from the senders 00a0… |
| `009ff690` | ClaimItemsRequestPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor in an SEH frame: restores ClaimItemsRequestPacket::vftable, BaseClaimPacket::vftable, BasePacket::vftable and frees this when bit 0 … |
| `00a00420` | ClaimItemsRequestPacket::ClaimItemsRequestPacket 🌐 | likely | Constructor: runs the base constructor 009ff290 with sub-opcode 3, stores ClaimItemsRequestPacket::vftable, then stores the two caller arguments at param_1[3] … |

**ClaimItemsResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ff700` | ClaimItemsResponsePacket::~ClaimItemsResponsePacket | confirmed | Destructor body: rewrites the vtable to ClaimItemsResponsePacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable; called from the handler 00a… |
| `009ff760` | ClaimItemsResponsePacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor in an SEH frame: restores ClaimItemsResponsePacket::vftable, BaseClaimPacket::vftable, BasePacket::vftable, then frees this when the… |
| `00a00490` | ClaimItemsResponsePacket::ClaimItemsResponsePacket 🌐 | likely | Constructor: runs the base constructor 009ff290 with sub-opcode 4, stores ClaimItemsResponsePacket::vftable, stores the global DAT_01be63fc (the invalid-id con… |

**ClientActivityLaunchBasePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c0bc0` | ClientActivityLaunchBasePacketCtor 🌐 | confirmed | Installs BasePacket and ClientActivityLaunchBasePacket vtables, stores opcode 0xaf, and zeroes three packet fields. |
| `009c1200` | ClientActivityLaunchBasePacketScalarDeletingDtor | confirmed | Rewrites the object vtable to BasePacket::vftable and frees the allocation when delete flag bit 0 is set. |

**ClientGateway::BasePacket** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ada0` | ClientGateway::BasePacketScalarDtor | confirmed | Scalar deleting destructor of ClientGateway::BasePacket: stores the class vtable at +0 and calls _free when bit 0 of param_2 is set. |
| `0089ae20` | ClientGatewayBasePacketDtor_89ae20 | confirmed | Trivial destructor body resetting the ClientGateway::BasePacket vtable at +0; cited by 226 EH unwind funclets. |
| `0089aed0` | ClientGatewayBasePacketDtor_89aed0 | confirmed | Trivial destructor body resetting the ClientGateway::BasePacket vtable at +0; cited by 93 EH unwind funclets. |
| `0089ad90` | ClientGateway::BasePacketDtor | likely | Trivial destructor body of ClientGateway::BasePacket: writes the class vtable pointer at +0 and returns; cited by 351 EH unwind funclets, the highest fan-in of… |
| `00b5bc20` | WrapPetNameRequestPayload 🌐 | likely | Initializes a tunneled-client packet from a stack serialization buffer, copies the serialized data span into the wrapper, stores param_2 as the wrapper byte, i… |
| `00b5bd40` | WrapPetUiModePayload 🌐 | likely | Serializes the stack buffer through FUN_00b5a240, copies its pointer and length into a tunneled-client packet, stores param_2 as a byte, and installs the BaseP… |
| `00b5be50` | WrapPetFavoriteTrickPayload 🌐 | likely | Serializes the supplied stack object, copies the resulting payload span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vt… |
| `00b5bf60` | WrapPetUtilityFeedPayload 🌐 | likely | Serializes the stack payload through FUN_00b5a380, copies its data span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vt… |
| `00b5c070` | WrapPetUtilityGroomPayload 🌐 | likely | Serializes the stack payload through FUN_00b5a420, copies its data span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vt… |
| `00b5c180` | WrapPetPlayWithToyPayload 🌐 | likely | Serializes the stack payload through FUN_00b5a4c0, copies the data span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vt… |
| `00b5c290` | WrapPetCanGoLocationPayload 🌐 | likely | Serializes the payload through FUN_00b5a530, including the location block and trailing byte, copies its span into a tunneled-client packet, and installs the Ba… |
| `00b5c3a0` | WrapPetPerformTrickPayload 🌐 | likely | Serializes the payload through FUN_00b5a590, copies its data span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vtable. |
| `00b5c4b0` | WrapPetGestureResultPayload 🌐 | likely | Serializes the supplied stack payload through FUN_00b5a5f0, copies its byte span into a tunneled-client packet, stores param_2 as a byte, and installs the Base… |
| `00b5c5c0` | WrapPetSummonRecallPayload 🌐 | likely | Serializes a dword plus a byte from the supplied object, copies the resulting span into a tunneled-client packet, stores param_2 as a byte, and installs the Ba… |
| `00b5c6d0` | WrapPetSummonByItemPayload 🌐 | likely | Serializes two dwords from the supplied body, copies the resulting span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vt… |
| `00b5c7e0` | WrapPetEquipPayload 🌐 | likely | Serializes a leading byte and two dwords from the payload, copies its span into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket… |
| `00b5c8f0` | WrapPetInWaterPayload 🌐 | likely | Serializes a single byte from the payload, copies that byte into a tunneled-client packet, stores param_2 as a byte, and installs the BasePacket vtable. |
| `00b5ca00` | WrapPetTargetPayload 🌐 | likely | Serializes four dwords from the supplied target-related body, copies the data span into a tunneled-client packet, stores param_2 as a byte, and installs the Ba… |
| `00b5cd50` | WrapPetEquipByItemRecordPayload 🌐 | likely | Serializes the supplied item-record payload through FUN_00b5b0c0, copies its span into a tunneled-client packet, stores param_2 as a byte, and installs the Bas… |
| `00bb8260` | SerializeRecordAndSend | likely | Serializes the caller's record, writes its length-prefixed body into a tunneled client-world packet, dispatches it through 00926360, then destroys the temporar… |
| `00bb8380` | SerializeListAndSend | likely | Serializes the caller's record and linked values into a tunneled client packet, sends through 00925640, and destroys the temporary packet buffer. |

**ClientGateway::PacketClientIsHosted** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acd390` | ClientGateway::PacketClientIsHosted::ScalarDeletingDtor | confirmed | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of … |
| `00acc3b0` | ClientGateway::PacketClientIsHosted::Destructor | likely | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable; it is reached from an unwind funclet and from the dispatcher's cas… |

**ClientGateway::PacketLogin** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00accb60` | ClientGateway::PacketLogin::PacketLogin 🌐 | confirmed | Constructor of ClientGateway::PacketLogin: installs BasePacket::vftable, stamps the packet id 1 into the word at +4, installs ClientGateway::PacketLogin::vftab… |
| `00accd80` | ClientGateway::PacketLogin::ScalarDeletingDtor | confirmed | 27-byte scalar deleting destructor: calls the destructor body 00accc00 (which releases the three strings) and frees the object with _free when bit 0 of the fla… |

**ClientGateway::PacketLoginReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acd1d0` | ClientGateway::PacketLoginReply::ScalarDeletingDtor | confirmed | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of … |
| `00acc260` | ClientGateway::PacketLoginReply::Destructor | likely | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable and returns; it is reached from compiler unwind funclets and as the… |

**ClientGateway::PacketLogout** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acc2b0` | ClientGateway::PacketLogout::PacketLogout 🌐 | confirmed | Constructor of ClientGateway::PacketLogout: installs BasePacket::vftable, stamps the packet id 3 into the packet word at +4, then installs ClientGateway::Packe… |
| `00acca20` | ClientGateway::PacketLogout::ScalarDeletingDtor | confirmed | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of … |

**ClientGateway::PacketOnlineStatusReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acc5d0` | ClientGateway::PacketOnlineStatusReply::PacketOnlineStatusR… 🌐 | confirmed | Constructor: installs BasePacket::vftable, stamps the packet id 9 into the word at +4, installs ClientGateway::PacketOnlineStatusReply::vftable and clears the … |
| `00acc630` | ClientGateway::PacketOnlineStatusReply::Destructor | confirmed | 73-byte destructor body: rewrites the vtable to ClientGateway::PacketOnlineStatusReply::vftable and then to ClientGateway::BasePacket::vftable; reached from an… |
| `00acc680` | ClientGateway::PacketOnlineStatusReply::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to ClientGateway::PacketOnlineStatusReply::vftable then BasePacket::vftable, and frees the object with _free when… |

**ClientGateway::PacketOnlineStatusRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acc4b0` | ClientGateway::PacketOnlineStatusRequest::PacketOnlineStatu… 🌐 | confirmed | Constructor: installs BasePacket::vftable, stamps the packet id 8 into the word at +4 and installs ClientGateway::PacketOnlineStatusRequest::vftable. |
| `00acc510` | ClientGateway::PacketOnlineStatusRequest::Destructor | confirmed | 73-byte destructor body: rewrites the vtable to ClientGateway::PacketOnlineStatusRequest::vftable and then to ClientGateway::BasePacket::vftable (MSVC's two-st… |
| `00acc560` | ClientGateway::PacketOnlineStatusRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to ClientGateway::PacketOnlineStatusRequest::vftable then BasePacket::vftable, and frees the object with _free wh… |

**ClientGateway::PacketServerForcedLogout** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acd2b0` | ClientGateway::PacketServerForcedLogout::ScalarDeletingDtor | confirmed | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of … |
| `00acc360` | ClientGateway::PacketServerForcedLogout::Destructor | likely | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable; it is reached from an unwind funclet and from the dispatcher's cas… |

**ClientGateway::PacketTunneledClientGatewayPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acca80` | ClientGateway::PacketTunneledClientGatewayPacket::ScalarDel… | confirmed | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of … |
| `00acc400` | ClientGateway::PacketTunneledClientGatewayPacket::PacketTun… | likely | Constructor: installs BasePacket::vftable, stamps the packet id 10 into the word at +4 and installs ClientGateway::PacketTunneledClientGatewayPacket::vftable. |

**ClientGateway::PacketTunneledClientPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089adc0` | ClientGateway::PacketTunneledClientPacketCtor 🌐 | confirmed | Constructor of ClientGateway::PacketTunneledClientPacket: installs ClientGateway::BasePacket::vftable, stores the gateway opcode 5 at +4, then installs ClientG… |
| `008b0ec0` | PacketTunneledClientPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of ClientGateway::PacketTunneledClientPacket::vftable: inside an SEH frame it writes ClientGateway::BasePacket::vftable at… |

**ClientGateway::PacketTunneledClientWorldPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ae70` | ClientGateway::PacketTunneledClientWorldPacketCtor 🌐 | confirmed | Constructor of ClientGateway::PacketTunneledClientWorldPacket: installs ClientGateway::BasePacket::vftable, stores the gateway opcode 6 at +4, then installs Cl… |
| `008b0f20` | PacketTunneledClientWorldPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of ClientGateway::PacketTunneledClientWorldPacket::vftable: writes ClientGateway::BasePacket::vftable at offset 0 inside a… |

**ClientHousingPacketSaveFixture** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0640` | ClientHousingPacketSaveFixtureScalarDeletingDtor | confirmed | Calls the BaseHousingPacket destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |

**ClientServerCore::BasePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d9090` | ClientServerCore::BasePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores ClientServerCore::BasePacket::vftable at this and calls _free on the object when bit 0 of the argument is set; returns this. |

**ClientUpdatePacketCollectionStart** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d8d0` | ClientUpdatePacketCollectionStartCtor 🌐 | confirmed | Constructor of ClientUpdatePacketCollectionStart: runs the base constructor FUN_0089d180 with sub-opcode 8, then installs ClientUpdatePacketCollectionStart::vf… |
| `0089d930` | ClientUpdatePacketCollectionStartDtor | confirmed | Destructor body: installs ClientUpdatePacketCollectionStart::vftable, then BaseClientUpdatePacket::vftable and BasePacket::vftable; no member cleanup. |

**ClientUpdatePacketEquipItem** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc810` | ClientUpdatePacketEquipItem::ClientUpdatePacketEquipItem 🌐 | confirmed | Packet constructor: calls family initializer 0089d180 with sub-opcode 5 (family 38), installs ClientUpdatePacketEquipItem::vftable, calls 0101cd40 with ECX=thi… |
| `008dc890` | ClientUpdatePacketEquipItem::~ClientUpdatePacketEquipItem | confirmed | Destructor body: rewrites ClientUpdatePacketEquipItem::vftable, runs the member destructor 0070a900, then rewrites BaseClientUpdatePacket::vftable and BasePack… |
| `008dc900` | ClientUpdatePacketEquipItem::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008dc890 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**ClientUpdatePacketHitpoints** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d2b0` | ClientUpdatePacketHitpointsCtor 🌐 | confirmed | Constructor of ClientUpdatePacketHitpoints: runs the base constructor FUN_0089d180 with sub-opcode 1, then installs ClientUpdatePacketHitpoints::vftable. |

**ClientUpdatePacketItemAdd** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d410` | ClientUpdatePacketItemAddCtor 🌐 | confirmed | Constructor of ClientUpdatePacketItemAdd: runs the base constructor FUN_0089d180 with sub-opcode 2, then installs ClientUpdatePacketItemAdd::vftable. |
| `0089d470` | ClientUpdatePacketItemAddDtor | confirmed | Destructor body: installs ClientUpdatePacketItemAdd::vftable, then BaseClientUpdatePacket::vftable and BasePacket::vftable; no member cleanup. |
| `0089d4d0` | ClientUpdatePacketItemAddScalarDtor | confirmed | Scalar deleting destructor: installs the three vtables (derived, family, base) and calls _free when bit 0 of param_2 is set. |

**ClientUpdatePacketItemDelete** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d6c0` | ClientUpdatePacketItemDeleteCtor 🌐 | confirmed | Constructor of ClientUpdatePacketItemDelete: runs the base constructor FUN_0089d180 with sub-opcode 4, installs the derived vtable, calls FUN_0101cd40, and whe… |

**ClientUpdatePacketMana** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d360` | ClientUpdatePacketManaCtor 🌐 | confirmed | Constructor of ClientUpdatePacketMana: runs the base constructor FUN_0089d180 with sub-opcode 0xd (13), installs the derived vtable and clears the payload fiel… |

**ClientUpdatePacketNotifyPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cf5e0` | ClientUpdatePacketNotifyPlayer::ClientUpdatePacketNotifyPla… 🌐 | confirmed | Packet constructor: runs the client-update family constructor FUN_0089d180 with subtype 0x17 (23), installs ClientUpdatePacketNotifyPlayer::vftable, then initi… |
| `008dc920` | ClientUpdatePacketNotifyPlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf650 (labelled as ClientUpdatePacketNotifyPlayer destructor) and frees the object with _free (00d82463… |
| `008cf650` | ClientUpdatePacketNotifyPlayer::~ClientUpdatePacketNotifyPl… | likely | Destructor body: releases the embedded SoeUtil::IString at +0xc (buffer +0x10, capacity +0x18) through the locked refcount and virtual slot +8, then resets the… |

**ClientUpdatePacketUnequipSlot** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d7a0` | ClientUpdatePacketUnequipSlotCtor 🌐 | confirmed | Constructor of ClientUpdatePacketUnequipSlot: runs the base constructor FUN_0089d180 with sub-opcode 6, then installs ClientUpdatePacketUnequipSlot::vftable. |
| `0089d800` | ClientUpdatePacketUnequipSlotDtor | confirmed | Destructor body: installs ClientUpdatePacketUnequipSlot::vftable, then BaseClientUpdatePacket::vftable and BasePacket::vftable; no member cleanup. |
| `0089d860` | ClientUpdatePacketUnequipSlotScalarDtor | confirmed | Scalar deleting destructor: installs the three vtables (derived, family, base) and calls _free when bit 0 of param_2 is set. |

**CompletedQuestCountUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7b300` | CompletedQuestCountUpdatePacket::ReadFields | confirmed | Reads the common prefix and one 32-bit completed-count field at output+0xc; short input zeroes the field and marks the reader as erroneous. |
| `00c7b730` | CompletedQuestCountUpdatePacket::Unserialize | likely | Rejects null inputs, constructs a bounded reader, parses the completed-count field and validates reader status and complete consumption policy. |

**CreateLobbyGamePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb0b80` | CreateLobbyGamePacket_Ctor | confirmed | Constructor: calls 008a3ce0 with argument 5 on ECX (lobby packet base constructor), stores the CreateLobbyGamePacket vtable (01824fc8), copies the source recor… |

**EncounterDataCommon** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01050ad0` | EncounterDataCommon::Ctor | confirmed | Installs BasePacket::vftable then EncounterDataCommon::vftable at +0, stores 0x3e at +4 and the constant from _DAT_0175b420 at +0x18, zeroes the byte fields at… |

**EncounterMatchmaking::AddMatchRequestResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa1140` | EncounterMatchmaking::AddMatchRequestResponsePacket::AddMat… | confirmed | Constructor: calls the matchmaking family constructor FUN_00a7f0d0 with 4, stores EncounterMatchmaking::AddMatchRequestResponsePacket::vftable at +0, then cons… |
| `00aa19a0` | EncounterMatchmaking::AddMatchRequestResponsePacket::Scalar… | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00aa11a0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |
| `00aa11a0` | EncounterMatchmaking::AddMatchRequestResponsePacket::~AddMa… | likely | Destructor body: calls FUN_00a82b30 (the embedded request/member teardown) then rewrites the object vtable to EncounterMatchmaking::BaseMatchmakingPacket::vfta… |

**EncounterMatchmaking::BaseMatchmakingPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9dfb0` | EncounterMatchmaking::BaseMatchmakingPacket::~BaseMatchmaki… | confirmed | Destructor body of the matchmaking packet base: restores the SoeUtil::List vtable at +0x28 and calls the list destructor FUN_007798c0, restores the list vtable… |

**EncounterMatchmaking::ClearMatchRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa1200` | EncounterMatchmaking::ClearMatchRequestPacket::ClearMatchRe… | confirmed | Constructor: calls the matchmaking family constructor FUN_00a7f0d0 with 5, stores EncounterMatchmaking::ClearMatchRequestPacket::vftable at +0 and then constru… |
| `00aa19c0` | EncounterMatchmaking::ClearMatchRequestPacket::ScalarDeleti… | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00aa1260 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |
| `00aa1260` | EncounterMatchmaking::ClearMatchRequestPacket::~ClearMatchR… | likely | Destructor body: calls FUN_00a82b30 then rewrites the object vtable to EncounterMatchmaking::BaseMatchmakingPacket::vftable and BasePacket::vftable; referenced… |

**EncounterMatchmaking::ListQueuesResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9f1d0` | EncounterMatchmaking::ListQueuesResponsePacket::ListQueuesR… | confirmed | Constructor: calls the matchmaking family constructor FUN_00a7f0d0 with 2, stores EncounterMatchmaking::ListQueuesResponsePacket::vftable at +0 and a SoeUtil::… |
| `00aa0680` | EncounterMatchmaking::ListQueuesResponsePacket::ScalarDelet… | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00a9f240 (list drain plus base vtable restore) and, when bit 0 of the flags argument is set, cal… |
| `00a9f240` | EncounterMatchmaking::ListQueuesResponsePacket::~ListQueues… | likely | Destructor body: restores the SoeUtil::List vtable at +0x18, then loops calling FUN_00a80850 on the current node at +0x1c until it becomes null (draining the q… |

**EncounterMatchmaking::QueueStatsResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9df30` | EncounterMatchmaking::QueueStatsResponsePacket::QueueStatsR… | confirmed | Constructor: calls the matchmaking-family constructor FUN_00a7f0d0 with 0xe (14), stores EncounterMatchmaking::QueueStatsResponsePacket::vftable at +0, then in… |
| `00a9f2c0` | EncounterMatchmaking::QueueStatsResponsePacket::ScalarDelet… | confirmed | MSVC scalar deleting destructor: calls the shared matchmaking destructor body FUN_00a9dfb0 and, when bit 0 of the flags argument is set, calls _free on the obj… |

**ExecuteScriptPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a8d4e0` | ExecuteScriptPacketDeletingDestructor | confirmed | Calls the packet destructor body, frees the object only when flag bit 0 is set, and otherwise returns the original object pointer. |

**ExecuteScriptWithStringParamsPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a8f900` | ExecuteScriptWithStringParamsPacketConstructor 🌐 | confirmed | Constructs the BaseUiPacket family with subtype value 8, initializes its owned IString member, and initializes an empty list of IString parameters. |
| `00a91790` | ExecuteScriptWithStringParamsDeletingDestructor | confirmed | Runs the packet cleanup body, frees the object only when flag bit 0 is set, and otherwise returns the original pointer. |

**FirstTimeEventClearRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c88320` | FirstTimeEventClearRequestCtor | confirmed | Constructs base first-time-event subtype 3, installs the clear-request vtable, and stores the event identifier. |
| `00c886f0` | FirstTimeEventClearRequestScalarDeletingDtor | confirmed | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**FirstTimeEventEnablePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c883e0` | FirstTimeEventEnablePacketCtor | confirmed | Constructs base first-time-event subtype 4, installs the enable-packet vtable, and stores the one-byte enabled flag. |
| `00c88750` | FirstTimeEventEnablePacketScalarDeletingDtor | confirmed | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**FirstTimeEventScriptPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c88a00` | FirstTimeEventScriptPacketCtor | confirmed | Constructs base first-time-event subtype 5, initializes an owned string, and zero-initializes its integer list and other fields. |
| `00c88a80` | FirstTimeEventScriptPacketDtor | confirmed | Destroys the integer list, releases the owned string buffer, restores the BasePacket vtable, and returns. |
| `00c88da0` | FirstTimeEventScriptPacketScalarDeletingDtor | confirmed | Runs the script-packet destructor, frees the object when flag bit 0 is set, and otherwise returns it. |

**FirstTimeEventStatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c88240` | FirstTimeEventStatePacketCtor | confirmed | Constructs base first-time-event subtype 2, installs the state-packet vtable, and stores the event id and one-byte state. |
| `00c88690` | FirstTimeEventStatePacketScalarDeletingDtor | confirmed | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**FirstTimeEventTriggerRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c88180` | FirstTimeEventTriggerRequestCtor | confirmed | Constructs base first-time-event subtype 1, installs the trigger-request vtable, and stores the request value. |
| `00c88630` | FirstTimeEventTriggerRequestScalarDeletingDtor | confirmed | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**FreeInteractionNpc** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9a90` | FreeInteractionNpc::FreeInteractionNpc 🌐 | confirmed | Constructor for FreeInteractionNpc: calls FUN_008a2820(0x14,0), the BaseCommandPacket constructor that installs BasePacket then BaseCommandPacket vtables and s… |
| `009db7f0` | FreeInteractionNpc::ScalarDeletingDtor | likely | Scalar deleting destructor: rewrites the vtables to BaseCommandPacket::vftable then BasePacket::vftable and frees the object when (param_2 and 1) is set. Slot … |

**FriendAddPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7f8e0` | FriendAddPacket::FriendAddPacket | confirmed | Calls the packet base initializer with 6, installs FriendAddPacket::vftable, then initializes the shared embedded fields through FUN_00c7f750. |
| `00c7f940` | FriendAddPacket::ScalarDeletingDtor | confirmed | Calls the FriendAddPacket destructor and frees the object only when flag bit 0 is set; returns the object pointer otherwise. |

**FriendListPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c81230` | FriendListPacket::ScalarDeletingDtor | confirmed | Calls FriendListPacket cleanup and conditionally frees the object when flag bit 0 is set. |

**FriendMessagePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7f960` | FriendMessagePacket::FriendMessagePacket | confirmed | Initializes packet family value 0, installs FriendMessagePacket::vftable, zeros one field, and initializes two embedded members. |
| `00c7f9f0` | FriendMessagePacket::ScalarDeletingDtor | confirmed | Calls the FriendMessagePacket destructor and conditionally frees the object when flag bit 0 is set. |

**FriendOfflinePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7df20` | FriendOfflinePacket::Construct | confirmed | Calls the common friend packet constructor with sub-opcode 3, installs FriendOfflinePacket::vftable, and stores two arguments at object+0x10 and +0x14. |
| `00c7df90` | FriendOfflinePacket::DeletingDtor | confirmed | Restores BasePacket::vftable, conditionally frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**FriendOnlinePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7e910` | FriendOnlinePacket::Construct | confirmed | Calls friend-family initializer with sub-opcode 2, installs FriendOnlinePacket vtable, initializes embedded IString/default fields and clears the boolean at +0… |
| `00c7ed60` | FriendOnlinePacket::DeletingDtor | confirmed | Calls string-member destructor 00c7e9a0, conditionally frees on flag bit 0, and otherwise returns the object pointer. |

**FriendRemovePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7dff0` | FriendRemovePacket::Construct | confirmed | Calls the common friend packet constructor with sub-opcode 7, installs FriendRemovePacket::vftable, and copies two global default fields into object+0x10/+0x14. |
| `00c7e060` | FriendRemovePacket::DeletingDtor | confirmed | Restores BasePacket::vftable, frees the packet if flag bit 0 is set, and otherwise returns the original object pointer. |

**FriendRenamePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7e810` | FriendRenamePacket::Construct | confirmed | Calls friend-family initializer with sub-opcode 10, installs FriendRenamePacket vtable, initializes an embedded IString and copies two global defaults. |
| `00c7ed40` | FriendRenamePacket::DeletingDtor | confirmed | Calls string-member destructor 00c7e890, frees the object when flag bit 0 is set, otherwise returns the pointer. |

**FriendStatusPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7ea20` | FriendStatusPacket::Construct | confirmed | Calls friend-family initializer with sub-opcode 9, installs FriendStatusPacket vtable, initializes embedded IString/default fields and zeroes trailing status/p… |
| `00c7eea0` | FriendStatusPacket::DeletingDtor | confirmed | Calls string-member destructor 00c7eac0 and frees the object only when flag bit 0 is set. |

**FriendUpdatePositionsPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c80d40` | FriendUpdatePositionsPacket::FriendUpdatePositionsPacket | confirmed | Initializes packet family value 5, installs FriendUpdatePositionsPacket and typed entry-array vtables, zeros array storage/count fields, and optionally grows t… |
| `00c80dd0` | FriendUpdatePositionsPacket::ScalarDeletingDtor | confirmed | Calls the packet cleanup routine and conditionally frees the packet object when flag bit 0 is set. |

**GetAllAvailableClaimItemsRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ff3c0` | GetAllAvailableClaimItemsRequestPacket::GetAllAvailableClai… 🌐 | confirmed | Constructor: runs the BaseClaimPacket base constructor 009ff290 with sub-opcode 1 and then stores GetAllAvailableClaimItemsRequestPacket::vftable. |
| `009ff480` | GetAllAvailableClaimItemsRequestPacket::ScalarDeletingDestr… | confirmed | Scalar deleting destructor in an SEH frame: stores GetAllAvailableClaimItemsRequestPacket::vftable, BaseClaimPacket::vftable, BasePacket::vftable and then _fre… |
| `009ff420` | GetAllAvailableClaimItemsRequestPacket::~GetAllAvailableCla… | likely | Destructor body: rewrites the object's vtable to GetAllAvailableClaimItemsRequestPacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable. |

**GetAllAvailableClaimItemsResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ff5c0` | GetAllAvailableClaimItemsResponsePacket::ScalarDeletingDest… | confirmed | Scalar deleting destructor in an SEH frame: restores GetAllAvailableClaimItemsResponsePacket::vftable, BaseClaimPacket::vftable and BasePacket::vftable, then _… |
| `009ff4f0` | GetAllAvailableClaimItemsResponsePacket::GetAllAvailableCla… 🌐 | likely | Constructor: runs the base constructor 009ff290 with sub-opcode 2, clears the byte at +0xc, stores GetAllAvailableClaimItemsResponsePacket::vftable and stores … |
| `009ff560` | GetAllAvailableClaimItemsResponsePacket::~GetAllAvailableCl… | likely | Destructor body: rewrites the vtable to GetAllAvailableClaimItemsResponsePacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable; called from … |

**HousingPacketFixturePlacementDenied** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0920` | HousingPacketFixturePlacementDeniedScalarDeletingDtor | confirmed | Calls the placement-denied packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |

**HousingPacketFixtureUpdate** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0660` | HousingPacketFixtureUpdateScalarDeletingDtor | confirmed | Calls the fixture-update destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |

**HousingPacketZoneData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab07a0` | HousingPacketZoneDataCtor | confirmed | Constructs the base housing packet with subtype 0x2d, installs the HousingPacketZoneData vtable, initializes its payload defaults, and sets two byte fields and… |
| `00ab0820` | HousingPacketZoneDataScalarDeletingDtor | confirmed | Calls the zone-data packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |

**IgnoreAddPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7eec0` | IgnoreAddPacket::Construct | confirmed | Calls common ignore initializer with sub-opcode 2, installs IgnoreAddPacket vtable and initializes an embedded IString member. |
| `00c7ef20` | IgnoreAddPacket::DeletingDtor | confirmed | Calls cleanup body 00c7ec20 and frees on flag bit 0, otherwise returns the packet pointer. |

**IgnoreListPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c80df0` | IgnoreListPacket::IgnoreListPacket | confirmed | Initializes packet family value 1, installs IgnoreListPacket and typed IgnoreData-list vtables, and zeros the list fields. |
| `00c81130` | IgnoreListPacket::ScalarDeletingDtor | confirmed | Calls the IgnoreListPacket base cleanup and conditionally frees the packet object when flag bit 0 is set. |

**IgnoreRemovePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7e120` | IgnoreRemovePacket::Construct | confirmed | Calls the common ignore packet constructor with sub-opcode 3 and installs IgnoreRemovePacket::vftable. |
| `00c7e180` | IgnoreRemovePacket::DeletingDtor | confirmed | Restores BasePacket::vftable, frees the object if flag bit 0 is set, and otherwise returns its pointer. |

**InventoryPacketEquipByGuid** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bda880` | InventoryPacketEquipByGuid::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquipByGuid::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+0 an… |
| `00bda570` | InventoryPacketEquipByGuid::~InventoryPacketEquipByGuid | likely | Destructor body of the equip-by-guid scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, performing no member cleanup. |

**InventoryPacketEquipByItemRecord** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a10470` | InventoryPacketEquipByItemRecord_Ctor 🌐 | likely | Constructs the packet with FUN_00a10400(8) (inventory family 42, sub-opcode 8), installs InventoryPacketEquipByItemRecord::vftable, then stores the ItemRecord … |

**InventoryPacketEquipItemGroup** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bda960` | InventoryPacketEquipItemGroup::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquipItemGroup::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+0… |
| `00bda670` | InventoryPacketEquipItemGroup_Ctor 🌐 | likely | Constructs the packet with FUN_00a10400(0xe) (inventory family 42, sub-opcode 14) and installs InventoryPacketEquipItemGroup::vftable. |
| `00bda6d0` | InventoryPacketEquipItemGroup::~InventoryPacketEquipItemGro… | likely | Destructor body of the equip-item-group scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, with no member cleanup. |

**InventoryPacketEquipLightsaber** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bda8f0` | InventoryPacketEquipLightsaber::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquipLightsaber::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+… |
| `00bda5c0` | InventoryPacketEquipLightsaber_Ctor 🌐 | likely | Constructs the packet with FUN_00a10400(0xd) (inventory family 42, sub-opcode 13), installs InventoryPacketEquipLightsaber::vftable and calls FUN_0101cd40 to i… |
| `00bda620` | InventoryPacketEquipLightsaber::~InventoryPacketEquipLights… | likely | Destructor body of the equip-lightsaber scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, with no member cleanup. |

**InventoryPacketEquippedRemove** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bda9d0` | InventoryPacketEquippedRemove::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquippedRemove::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+0… |
| `00bda720` | InventoryPacketEquippedRemove_Ctor 🌐 | likely | Constructs the packet with FUN_00a10400(2) (inventory family 42, sub-opcode 2) and installs InventoryPacketEquippedRemove::vftable. |
| `00bda780` | InventoryPacketEquippedRemove::~InventoryPacketEquippedRemo… | likely | Destructor body of the unequip scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, with no member cleanup. |

**InventoryPacketItemActionBarAssignByItemRecord** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a122e0` | InventoryPacketItemActionBarAssignByItemRecord_Ctor 🌐 | confirmed | Constructs the packet with FUN_00a10400(9) (inventory family 42, sub-opcode 9), stores the slot param_2 at +0xc, installs InventoryPacketItemActionBarAssignByI… |

**InventoryPacketUseStyleCardByItemRecord** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a105a0` | InventoryPacketUseStyleCardByItemRecord_Ctor 🌐 | likely | Constructs the packet with FUN_00a10400(0xc) (inventory family 42, sub-opcode 12), installs InventoryPacketUseStyleCardByItemRecord::vftable and zeroes the fie… |

**ItemRecord** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bda810` | ItemRecord::ItemRecord | confirmed | Constructor: installs ItemRecord::vftable at object+0 and copies the dwords pointed to by the three pointer arguments into +4, +8 and +0xc (the vftable plus th… |

**KeyCodeRedemptionNotificationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a05980` | KeyCodeRedemptionNotificationPacket::~KeyCodeRedemptionNoti… | confirmed | Destructor body: rewrites the vtable to KeyCodeRedemptionNotificationPacket::vftable, then BasePromotionalPacket::vftable, then BasePacket::vftable; called fro… |
| `00a059e0` | KeyCodeRedemptionNotificationPacket::ScalarDeletingDestruct… | confirmed | Scalar deleting destructor in an SEH frame: restores KeyCodeRedemptionNotificationPacket::vftable, BasePromotionalPacket::vftable and BasePacket::vftable, then… |
| `00a05910` | KeyCodeRedemptionNotificationPacket::KeyCodeRedemptionNotif… 🌐 | likely | Constructor: runs the base constructor 00a057e0 with sub-opcode 1, stores KeyCodeRedemptionNotificationPacket::vftable and stores the caller's argument at para… |

**LeaveLobbyPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bafe60` | LeaveLobbyPacket_Ctor | confirmed | Constructor: calls 008a3ce0 with argument 9 on ECX (lobby packet base constructor), stores the LeaveLobbyPacket vtable (01824f3c) at +0 and the parameter at +0… |
| `00bb00e0` | LeaveLobbyPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0] of LeaveLobbyPacket): stores the BasePacket vtable (018011ec) and calls _free on the object when the delete flag bi… |

**MemberPanelDataSendPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a40590` | MemberPanelDataSendPacket::MemberPanelDataSendPacket 🌐 | confirmed | Constructor: runs the announcement base constructor FUN_00a3df90(4) (installing BasePacket/AnnouncementBasePacket vtables, base opcode 0xc1 at +4 and sub-opcod… |
| `00a40600` | MemberPanelDataSendPacket::~MemberPanelDataSendPacket | confirmed | Installs MemberPanelDataSendPacket::vftable, removes list-head nodes through FUN_00a3f760 until the +0x10 head is empty, reinstalls the embedded list vtable at… |
| `00a406a0` | MemberPanelDataSendPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of MemberPanelDataSendPacket: calls the destructor body 00a40600 and frees the packet with _free when bit 0 of the d… |
| `00a408f0` | ParseBufferIntoListB | likely | Builds a bounded reader over (param_2,param_3), parses the count-prefixed member-panel list into param_1+0xc through FUN_00a400e0, and returns 0 for null input… |

**MemberPanelDataSendPacket::MemberPanelData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3ebd0` | EmbeddedStrings2_Dtor_8_24 | likely | Destructor of a record with two SoeUtil::IString fields at +8 and +0x24: each field gets the base IString vtable back and, when its length (+0x14, +0x30) is po… |
| `00a3ed40` | EmbeddedStrings2_Ctor_8_24 | likely | Constructor of the two string fields at +8 and +0x24: installs SoeUtil::IString<char>::vftable for each, points the buffers at the shared static empty buffer D… |
| `00a3f100` | ScalarDelete_a3f100 | likely | Scalar deleting destructor under an SEH frame: calls the member-panel row destructor 00a3ebd0 and then calls _free(this) when bit 0 of the flag argument is set. |
| `00a3f560` | MemberPanelData_ReadFields | likely | Decodes one fixed-size record from the raw reader at param_2 into param_1: reads a dword into param_1[0] and param_1[1], then a length-prefixed string block as… |

**MysteryChestBasePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c6bd0` | MysteryChestBasePacketCtor | confirmed | Installs BasePacket then MysteryChestBasePacket vtables, sets the base opcode field to 0xca, and clears the subtype field. |
| `009c6c30` | MysteryChestBasePacketInitSubtype | confirmed | Installs BasePacket and MysteryChestBasePacket vtables, stores opcode 0xca, and writes the supplied subtype field. |
| `009c7400` | MysteryChestBasePacketScalarDeletingDtor | confirmed | Restores BasePacket vtable and frees the object only when the deleting flag is set, otherwise returns the pointer. |

**MysteryChestInfoReplyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c8520` | MysteryChestInfoReplyPacketDtor | confirmed | Destroys the packet's embedded Mystery Chest info map, then restores the BasePacket vtable. |
| `009c8a30` | MysteryChestInfoReplyPacketCtor | confirmed | Initializes MysteryChestBasePacket with subtype 2, installs the reply packet vtable, and initializes its embedded Mystery Chest info map. |
| `009c8a90` | MysteryChestInfoReplyScalarDeletingDtor | confirmed | Destroys the embedded Mystery Chest info map, conditionally frees the packet allocation, and otherwise returns the object pointer. |

**MysteryChestInfoRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c6cf0` | MysteryChestInfoRequestPacketCtor | confirmed | Initializes MysteryChestBasePacket with subtype 1, then installs the MysteryChestInfoRequestPacket vtable. |
| `009c7460` | MysteryChestInfoRequestScalarDeletingDtor | confirmed | Restores BasePacket vtable and frees the allocation when the deleting flag is set; otherwise returns the original pointer. |

**NameChange::BaseNameChangePacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0ff00` | BaseNameChangePacket_CtorDefault 🌐 | confirmed | Default constructor: stores BasePacket::vftable, writes the base opcode 0xc0 (192) into the packet's opcode field at +4, then stores NameChange::BaseNameChange… |
| `00a0ff60` | BaseNameChangePacket_Ctor 🌐 | confirmed | Constructor taking the sub-opcode: stores BasePacket::vftable, writes 0xc0 (192) into the opcode field at +4, installs NameChange::BaseNameChangePacket::vftabl… |
| `00a0ffd0` | BaseNameChangePacket_Dtor | likely | Destructor body: writes NameChange::BaseNameChangePacket::vftable then BasePacket::vftable into this (the MSVC base-destructor pattern); no fields are released. |
| `00a10020` | BaseNameChangePacket_ScalarDeletingDtor | likely | Scalar deleting destructor: writes the derived vtable then BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |
| `00a10100` | BaseNameChangePacket_Dtor2 | likely | Second destructor body for the same base class: stores the derived vtable and then BasePacket::vftable; no field release. |
| `00a12040` | BaseNameChangePacket_ScalarDeletingDtor2 | likely | Scalar deleting destructor: writes NameChange::BaseNameChangePacket::vftable then BasePacket::vftable and frees the object with _free when bit 0 of param_2 is … |

**NameChange::ChangeNameRequestStatusPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a10090` | ChangeNameRequestStatusPacket_Ctor 🌐 | confirmed | Constructs the packet by calling FUN_00a0ff60(5) (NameChange base with sub-opcode 5), installs NameChange::ChangeNameRequestStatusPacket::vftable and stores th… |

**NudgeBasePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a41310` | NudgeBasePacket::NudgeBasePacket 🌐 | confirmed | Base constructor of the nudge packet family: installs BasePacket::vftable, stores the family opcode 0xcb at +4, installs NudgeBasePacket::vftable and stores th… |
| `00a416c0` | NudgeBasePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of NudgeBasePacket: installs BasePacket::vftable and frees the object with _free when bit 0 of the delete flag is se… |
| `00a41380` | NudgeBasePacket::~NudgeBasePacket | likely | Destructor body that only reinstalls BasePacket::vftable at +0, i.e. the base-part teardown of a nudge packet object built on the stack. |
| `00a41440` | NudgeBasePacket::~NudgeBasePacket | likely | Second copy of the base-part teardown: reinstalls BasePacket::vftable at +0 and returns. |

**NudgeUpdatePacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a413d0` | NudgeUpdatePacket::NudgeUpdatePacket 🌐 | confirmed | Constructor of the nudge update message: runs the family base constructor FUN_00a41310(4) (opcode 0xcb, sub-opcode 4 at +8) and installs NudgeUpdatePacket::vft… |
| `00a41720` | NudgeUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of NudgeUpdatePacket: installs BasePacket::vftable and frees the packet with _free when bit 0 of the delete flag is … |
| `00a421f0` | SendNudgeUpdateKind1 🌐 | likely | Builds a NudgeUpdatePacket on the stack with the constructor 00a413d0, fills the payload dwords at +0xc, +0x10 and +0x14 with the three caller arguments and se… |
| `00a42290` | SendNudgeUpdateKind2 🌐 | likely | Same nudge update builder as 00a421f0 with the payload constant at +0x18 set to 2: constructs NudgeUpdatePacket (00a413d0), copies the three caller arguments t… |
| `00a42330` | SendNudgeUpdateKind4 🌐 | likely | Nudge update builder with the payload constant at +0x18 set to 4: constructs the packet (00a413d0), fills +0xc/+0x10/+0x14 from the three arguments, sets +0x18… |
| `00a423d0` | SendNudgeUpdateKind3 🌐 | likely | Nudge update builder with the payload constant at +0x18 set to 3: constructs the packet (00a413d0), copies the three caller arguments into +0xc/+0x10/+0x14, se… |

**OneTimeSessionRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b570` | OneTimeSessionRequestPacketDtor | confirmed | Destructor body: installs OneTimeSessionRequestPacket::vftable then BasePacket::vftable; no member cleanup. |
| `0089b5c0` | OneTimeSessionRequestPacketScalarDtor | confirmed | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
| `0089b510` | OneTimeSessionRequestPacketCtor | likely | Constructor of OneTimeSessionRequestPacket: installs BasePacket::vftable, stores opcode 0x75 (117) at +4, installs the derived vtable and clears the payload wo… |

**OneTimeSessionResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ea370` | OneTimeSessionResponsePacketCtor 🌐 | confirmed | Installs base and derived vtables, sets opcode 0x76, clears a flag, initializes embedded response and string member. |
| `008ea410` | OneTimeSessionResponsePacketDtor | confirmed | Decrements and conditionally destroys shared string buffer, cleans embedded response object and restores BasePacket vtable. |
| `008ea4c0` | OneTimeSessionResponsePacket::ScalarDeletingDtor | confirmed | Calls packet destructor, frees object if flag bit zero is set, otherwise returns original pointer. |

**PacketAddClientPortraitCrc** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b4aa30` | PacketAddClientPortraitCrc::Dtor | confirmed | Destructor body: rewrites the object's vtable to BasePacket::vftable (018011ec); no members are released. |
| `00b4a9c0` | PacketAddClientPortraitCrc::Ctor 🌐 | likely | Constructs a PacketAddClientPortraitCrc object: stores BasePacket::vftable, writes opcode 0x73 (115), stores PacketAddClientPortraitCrc::vftable, and places it… |

**PacketChat** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0f9b0` | PacketChat::ScalarDeletingDtor | confirmed | Calls BaseChatPacket cleanup and frees the object when the low flag bit is set. |
| `00b0f8f0` | PacketChat::Ctor 🌐 | likely | Initializes BaseChatPacket mode 1, constructs chat strings and fixed text storage, installs PacketChat's vtable, and zeros trailing fields. |

**PacketCheckNameReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00953a10` | PacketCheckNameReply::Ctor | confirmed | Fastcall ctor (ECX = this): installs BasePacket::vftable, stores the opcode 0xd3 (211) at +4, installs PacketCheckNameReply::vftable, and runs two member initi… |
| `00953a90` | PacketCheckNameReply::ScalarDeletingDtor | confirmed | Thiscall virtual (PacketCheckNameReply::vftable slot 0, flag arg): calls 00953270 (BasePacket::DtorBodyPartial), frees this when bit 0 of the flag is set, and … |

**PacketCheckNameRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009539f0` | PacketCheckNameRequest::ScalarDeletingDtor | confirmed | Thiscall virtual (PacketCheckNameRequest::vftable slot 0, flag arg): calls 009531b0 (BasePacket::DtorBody), frees this when bit 0 of the flag is set, and retur… |

**PacketClientAreaTimer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1160` | PacketClientAreaTimer::PacketClientAreaTimer 🌐 | confirmed | Initializes a PacketClientAreaTimer object: writes BasePacket::vftable, stores 0xac at this+4, zeroes this+8, installs PacketClientAreaTimer::vftable, stores -… |
| `008b11d0` | PacketClientAreaTimer::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientAreaTimer::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit… |

**PacketClientBeginZoning** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc360` | PacketClientBeginZoning::PacketClientBeginZoning 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0x1f (31) at +4, installs PacketClientBeginZoning::vftable, constructs a StringFixed<32> subobj… |
| `008dc410` | PacketClientBeginZoning::ScalarDeletingDtor | confirmed | Scalar deleting destructor (raw bytes 56 8b f1 e8 ... f6 44 24 08 01 74 09 56 e8 ...): runs the destructor body 008cedd0 and frees the object with _free (00d82… |

**PacketClientCaisNotification** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c2b0` | PacketClientCaisNotificationCtor 🌐 | confirmed | Constructor of PacketClientCaisNotification: installs BasePacket::vftable, stores opcode 0xbe (190) at +4, clears +8/+0xc, installs the derived vtable, copies … |
| `008b1ab0` | PacketClientCaisNotification::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientCaisNotification::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when f… |

**PacketClientExitLaunchUrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc5d0` | PacketClientExitLaunchUrl::PacketClientExitLaunchUrl 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0x61 (97) at +4 and PacketClientExitLaunchUrl::vftable; constructs a StringFixed<256> subobject… |
| `008dc640` | PacketClientExitLaunchUrl::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf270 (packet destructor with one StringFixed256 member) and frees the object with _free (00d82463) whe… |

**PacketClientFinishedLoading** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b000` | PacketClientFinishedLoadingCtor 🌐 | confirmed | Constructor of PacketClientFinishedLoading: installs BasePacket::vftable, stores opcode 10 at +4, then installs PacketClientFinishedLoading::vftable. |
| `0089b060` | PacketClientFinishedLoadingDtor | confirmed | Destructor body: installs PacketClientFinishedLoading::vftable then the BasePacket::vftable (inlined base destructor); no member cleanup. |
| `0089b0b0` | PacketClientFinishedLoadingScalarDtor | confirmed | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |

**PacketClientFlashTimer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c100` | PacketClientFlashTimerCtor 🌐 | confirmed | Constructor of PacketClientFlashTimer: installs BasePacket::vftable, stores opcode 0xb1 (177) at +4, installs the derived vtable, clears the byte at +8 and wri… |
| `008b1910` | PacketClientFlashTimer::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientFlashTimer::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bi… |

**PacketClientGameSettings** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089bc20` | PacketClientGameSettingsCtor 🌐 | confirmed | Constructor of PacketClientGameSettings: installs BasePacket::vftable, stores opcode 0x90 (144) at +4, installs the derived vtable, clears +0x10, sets the byte… |
| `008b1670` | PacketClientGameSettings::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientGameSettings::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag … |

**PacketClientInitializationDetails** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089bfa0` | PacketClientInitializationDetailsCtor 🌐 | confirmed | Constructor of PacketClientInitializationDetails: installs BasePacket::vftable, stores opcode 0xa9 (169) at +4, installs the derived vtable and clears the payl… |
| `008b1850` | PacketClientInitializationDetails::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientInitializationDetails::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this w… |

**PacketClientIsReady** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acc6f0` | PacketClientIsReady::PacketClientIsReady 🌐 | confirmed | Constructor: installs BasePacket::vftable, stamps the packet id 13 into the word at +4 and installs PacketClientIsReady::vftable. |
| `00acc750` | PacketClientIsReady::Destructor | confirmed | 73-byte destructor body: rewrites the vtable to PacketClientIsReady::vftable and then to BasePacket::vftable; reached only from an unwind funclet. |
| `00acc7a0` | PacketClientIsReady::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to PacketClientIsReady::vftable then BasePacket::vftable, and frees the object with _free when bit 0 of the flag … |

**PacketClientKickedFromServer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ceef0` | PacketClientKickedFromServer::PacketClientKickedFromServer 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores the opcode 0x51 (81) in the id slot +4, installs PacketClientKickedFromServer::vftable and initializes… |
| `008dc4d0` | PacketClientKickedFromServer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cef60 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008cef60` | PacketClientKickedFromServer::~PacketClientKickedFromServer | likely | Destructor body: releases the embedded SoeUtil::IString at +8 (capacity +0x14) through the locked refcount and virtual slot +8, then resets the vtable to BaseP… |

**PacketClientLog** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc430` | PacketClientLog::PacketClientLog 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0x6d (109) at +4 and PacketClientLog::vftable; constructs StringFixed<64> at +8 through 00402a1… |
| `008dc4b0` | PacketClientLog::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cee70 (releases the StringFixed64/StringFixed2048 members) and frees the object with _free (00d82463) w… |

**PacketClientLogout** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b120` | PacketClientLogoutCtor 🌐 | confirmed | Constructor of PacketClientLogout: installs BasePacket::vftable, stores opcode 0x10 (16) at +4, then installs PacketClientLogout::vftable. |
| `0089b180` | PacketClientLogoutDtor | confirmed | Destructor body: installs PacketClientLogout::vftable then BasePacket::vftable; no member cleanup. |
| `0089b1d0` | PacketClientLogoutScalarDtor | confirmed | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |

**PacketClientLuaMetrics** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089bb60` | PacketClientLuaMetricsCtor | confirmed | Constructor of PacketClientLuaMetrics: installs BasePacket::vftable, stores opcode 0x8e (142) at +4, installs the derived vtable and stores the payload word pa… |
| `008b1610` | PacketClientLuaMetrics::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientLuaMetrics::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bi… |

**PacketClientMembershipVipInfo** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1970` | PacketClientMembershipVipInfo::PacketClientMembershipVipInfo 🌐 | confirmed | Packet constructor: writes BasePacket::vftable, stores packet id 0xba (186) at this+4, installs PacketClientMembershipVipInfo::vftable, seeds this+8 from DAT_0… |
| `008b19f0` | PacketClientMembershipVipInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientMembershipVipInfo::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when … |

**PacketClientMetrics** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1420` | PacketClientMetrics::PacketClientMetrics 🌐 | confirmed | Constructor: writes BasePacket::vftable, stores id 0x69 at this+4, then installs PacketClientMetrics::vftable. Calls FUN_0089b800 with the payload address this… |
| `008b1490` | PacketClientMetrics::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientMetrics::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0… |

**PacketClientNotifyCoinSpinAvailable** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c050` | PacketClientNotifyCoinSpinAvailableCtor 🌐 | confirmed | Constructor of PacketClientNotifyCoinSpinAvailable: installs BasePacket::vftable, stores opcode 0xab (171) at +4, installs the derived vtable and clears +8/+0x… |
| `008b18b0` | PacketClientNotifyCoinSpinAvailable::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientNotifyCoinSpinAvailable::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this… |

**PacketClientPendingKickFromServer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cefe0` | PacketClientPendingKickFromServer::PacketClientPendingKickF… 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores the opcode 99 in the id slot +4, installs PacketClientPendingKickFromServer::vftable, initializes one … |
| `008dc4f0` | PacketClientPendingKickFromServer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf060 (a packet destructor handling an IString member at +8) and frees the object with _free (00d82463)… |

**PacketClientServerShuttingDown** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b460` | PacketClientServerShuttingDownCtor 🌐 | confirmed | Constructor of PacketClientServerShuttingDown: installs BasePacket::vftable, stores opcode 0x49 (73) at +4, then installs the derived vtable. |
| `008b1230` | PacketClientServerShuttingDown::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientServerShuttingDown::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when… |

**PacketClientSettings** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc780` | PacketClientSettings::PacketClientSettings 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0xc3 (195) at +4 and PacketClientSettings::vftable; calls the three-IString initializer 008dc26… |
| `008dc7f0` | PacketClientSettings::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf410 (which delegates to 008ceaf0, the three-IString payload destructor) and frees the object with _fr… |

**PacketClientSubstringBlacklist** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091bed0` | PacketClientSubstringBlacklist::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls FUN_00914290 (the object destructor body) and then frees this when bit 0 of the flag argument is set. |

**PacketClientTrialMountUpsell** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089be40` | PacketClientTrialMountUpsellCtor 🌐 | confirmed | Constructor of PacketClientTrialMountUpsell: installs BasePacket::vftable, stores opcode 0xc9 (201) at +4, then installs the derived vtable; no payload fields. |
| `008b1790` | PacketClientTrialMountUpsell::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientTrialMountUpsell::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when f… |

**PacketClientTrialPetUpsell** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089bd90` | PacketClientTrialPetUpsellCtor 🌐 | confirmed | Constructor of PacketClientTrialPetUpsell: installs BasePacket::vftable, stores opcode 199 at +4, then installs the derived vtable; no payload fields. |
| `008b1730` | PacketClientTrialPetUpsell::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientTrialPetUpsell::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when fla… |

**PacketClientTrialProfileUpsell** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089bce0` | PacketClientTrialProfileUpsellCtor 🌐 | confirmed | Constructor of PacketClientTrialProfileUpsell: installs BasePacket::vftable, stores opcode 0x91 (145) at +4, then installs the derived vtable; no payload field… |
| `008b16d0` | PacketClientTrialProfileUpsell::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketClientTrialProfileUpsell::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when… |

**PacketEnterSocialZoneRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c142c0` | PacketEnterSocialZoneRequest::PacketEnterSocialZoneRequest 🌐 | confirmed | Constructor: stores BasePacket::vftable, writes the packet opcode 0xcc (204) at +4, then stores PacketEnterSocialZoneRequest::vftable at *this; no payload fiel… |
| `00c1a6b0` | PacketEnterSocialZoneRequest::ScalarDeletingDtor | confirmed | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |

**PacketGameTimeSync** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd1670` | PacketGameTimeSync::ScalarDeletingDtor | confirmed | Virtual slot 0 of PacketGameTimeSync. Stores the BasePacket vftable at this+0 (the base destructor shape), then calls _free(this) only when bit 0 of the flag a… |
| `00bd1600` | PacketGameTimeSync::Ctor | likely | Constructor body: stores the BasePacket vftable at this+0, sets the opcode dword at +4 to 0x34, then stores the PacketGameTimeSync vftable, copies the dwords D… |
| `00bd1910` | PacketGameTimeSyncWrite 🌐 | likely | Serialises a GameTimeSync packet object (param_1) into the writer param_2: writes the low 16 bits of the dword at +4 (the opcode, 0x34 when built by 00bd1600) … |

**PacketGeneratePortraitRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad3190` | PacketGeneratePortraitRequest::Ctor | confirmed | Calls the BaseFotomatPacket constructor with sub-opcode 1, installs PacketGeneratePortraitRequest vtable, and clears one member. |
| `00ad35e0` | PacketGeneratePortraitRequest::ScalarDeletingDtor | confirmed | Rewrites the object vptr through BaseFotomatPacket then BasePacket and frees this when the deleting-destructor flag bit is set. |

**PacketInitializationParameters** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc6f0` | PacketInitializationParameters::PacketInitializationParamet… 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0xa6 (166) at +4 and PacketInitializationParameters::vftable; constructs a StringFixed<64> subo… |
| `008dc760` | PacketInitializationParameters::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf3b0 (packet destructor with one StringFixed64 member) and frees the object with _free (00d82463) when… |

**PacketInviteAndStartMiniGamePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b0fb0` | PacketInviteAndStartMiniGamePacket::PacketInviteAndStartMin… 🌐 | confirmed | Packet constructor: writes BasePacket::vftable, stores the packet id 0xb4 (180) at this+4, overwrites offset 0 with PacketInviteAndStartMiniGamePacket::vftable… |
| `008b1040` | PacketInviteAndStartMiniGamePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketInviteAndStartMiniGamePacket::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this … |

**PacketInviteAndStartMiniGameStartGamePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b10a0` | PacketInviteAndStartMiniGameStartGamePacket::PacketInviteAn… | confirmed | Constructor of the start-game subclass: inside an SEH frame it calls the base constructor 008b0fb0 with argument 3 (which sets packet id 0xb4 and the payload f… |
| `008b1100` | PacketInviteAndStartMiniGameStartGamePacket::ScalarDeleting… | confirmed | Scalar deleting destructor in slot 0 of PacketInviteAndStartMiniGameStartGamePacket::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and fr… |

**PacketLoadWelcomeScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090b680` | PacketLoadWelcomeScreen::~PacketLoadWelcomeScreen | confirmed | Restores the claim-code and content array vtables, frees their allocations, then restores BasePacket::vftable. |
| `00914200` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] (thiscall, one byte argument). Calls thiscall FUN_0090b680 with ECX=this (the destructor body), then if (arg & 1) != 0 calls _free(this);… |
| `0090b5f0` | PacketLoadWelcomeScreen::PacketLoadWelcomeScreen 🌐 | likely | Sets packet opcode 0x5d, initializes the welcome-screen flag, and installs empty ContentInfo and ClaimCodeInfo arrays. |

**PacketMOTD** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ea4e0` | PacketMOTDCtor 🌐 | confirmed | Installs base and PacketMOTD vtables, sets opcode 0x57 and initializes two embedded strings. |
| `008ea560` | PacketMOTD::ScalarDeletingDtor | confirmed | Calls MOTD destructor body, frees object if flag bit zero is set, otherwise returns original pointer. |

**PacketMembershipActivation** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089bab0` | PacketMembershipActivationCtor 🌐 | confirmed | Constructor of PacketMembershipActivation: installs BasePacket::vftable, stores opcode 100 at +4, installs the derived vtable and clears the payload word at +8. |
| `008b15b0` | PacketMembershipActivation::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketMembershipActivation::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when fla… |

**PacketMembershipSubscriptionInfo** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c200` | PacketMembershipSubscriptionInfoCtor 🌐 | confirmed | Constructor of PacketMembershipSubscriptionInfo: installs BasePacket::vftable, stores opcode 0xbd (189) at +4, installs the derived vtable and clears the two f… |
| `008b1a50` | PacketMembershipSubscriptionInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketMembershipSubscriptionInfo::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this wh… |

**PacketPOIChangeMessage** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b9f0` | PacketPOIChangeMessageCtor 🌐 | confirmed | Constructor of PacketPOIChangeMessage: installs BasePacket::vftable, stores opcode 0x68 (104) at +4, installs the derived vtable and clears the three payload w… |
| `008b1550` | PacketPOIChangeMessage::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketPOIChangeMessage::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bi… |

**PacketSendSelfToClient** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b240` | PacketSendSelfToClientCtor 🌐 | confirmed | Constructor of PacketSendSelfToClient: installs BasePacket::vftable, stores opcode 0xc (12) at +4, installs the derived vtable and stores the two payload words… |
| `0089b2b0` | PacketSendSelfToClientDtor | confirmed | Destructor body: installs PacketSendSelfToClient::vftable then BasePacket::vftable; no member cleanup. |
| `0089b300` | PacketSendSelfToClientScalarDtor | confirmed | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |

**PacketSendZoneDetails** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cebc0` | PacketSendZoneDetails::PacketSendZoneDetails 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores the opcode 0x2b (43) into the packet id slot +4, installs PacketSendZoneDetails::vftable, initializes … |
| `008cec60` | PacketSendZoneDetails::~PacketSendZoneDetails | confirmed | Destructor body: rewrites PacketSendZoneDetails::vftable, releases the two embedded SoeUtil::IString members at +0x20 and +8 (capacity check + locked refcount … |
| `008ced20` | PacketSendZoneDetails::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the PacketSendZoneDetails destructor body FUN_008cec60 and frees the object with _free when bit 0 of the flag argument is… |

**PacketSetClientArea** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b6e0` | PacketSetClientAreaCtor 🌐 | confirmed | Constructor of PacketSetClientArea: installs BasePacket::vftable, stores opcode 0x59 (89) at +4, installs the derived vtable, sets the area id at +8 to -1, the… |
| `008b12f0` | PacketSetClientArea::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketSetClientArea::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0… |

**PacketSetLocale** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ea5f0` | PacketSetLocale::ScalarDeletingDtor | confirmed | Calls locale destructor body, frees object if flag bit zero is set, otherwise returns original pointer. |

**PacketShowSystemMessage** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b8e0` | PacketShowSystemMessageCtor 🌐 | confirmed | Constructor of PacketShowSystemMessage: installs BasePacket::vftable, stores opcode 0x67 (103) at +4, installs the derived vtable and clears +8/+0xc while sett… |
| `008b14f0` | PacketShowSystemMessage::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketShowSystemMessage::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag b… |

**PacketTargetClientNotOnline** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ced40` | PacketTargetClientNotOnline::~PacketTargetClientNotOnline | confirmed | Destructor body: rewrites PacketTargetClientNotOnline::vftable, runs the SoeUtil::IStringFixed<char,32> destructor body FUN_00703350 and resets the base vtable… |
| `008dc2f0` | PacketTargetClientNotOnline::PacketTargetClientNotOnline 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0x16 (22) at +4, installs PacketTargetClientNotOnline::vftable, then calls the StringFixed<32> … |
| `008cedb0` | PacketTargetClientNotOnline::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs FUN_008ced40 and frees the object with _free when bit 0 of the flag argument is set. |

**PacketUpdateClientAreaCompositeEffect** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c380` | PacketUpdateClientAreaCompositeEffectCtor 🌐 | confirmed | Constructor of PacketUpdateClientAreaCompositeEffect: installs BasePacket::vftable, stores opcode 0xd0 (208) at +4, installs the derived vtable, clears +8 and … |
| `008b1b10` | PacketUpdateClientAreaCompositeEffect::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketUpdateClientAreaCompositeEffect::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees th… |

**PacketUpdateClientSessionData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc510` | PacketUpdateClientSessionData::PacketUpdateClientSessionData 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0x52 (82) at +4 and PacketUpdateClientSessionData::vftable; constructs four StringFixed<256> me… |
| `008dc5b0` | PacketUpdateClientSessionData::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf0e0 (destroys the four StringFixed256 members) and frees the object with _free (00d82463) when the lo… |

**PacketUpdateUserAge** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b17f0` | PacketUpdateUserAge::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketUpdateUserAge::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0… |
| `0089bef0` | PacketUpdateUserAgeCtor 🌐 | likely | Constructor of PacketUpdateUserAge: installs BasePacket::vftable, stores opcode 0x9d (157) at +4, installs the derived vtable and clears the payload word at +8. |

**PacketWorldDisplayInfo** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1290` | PacketWorldDisplayInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketWorldDisplayInfo::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bi… |
| `0089b630` | PacketWorldDisplayInfoCtor 🌐 | likely | Constructor of PacketWorldDisplayInfo: installs BasePacket::vftable, stores opcode 0x56 (86) at +4, installs the derived vtable and clears the payload word at … |

**PacketWorldShutdownNotice** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc660` | PacketWorldShutdownNotice::PacketWorldShutdownNotice 🌐 | confirmed | Packet constructor: installs BasePacket::vftable, stores opcode 0x5c (92) at +4 and PacketWorldShutdownNotice::vftable; constructs a StringFixed<512> subobject… |
| `008dc6d0` | PacketWorldShutdownNotice::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008cf2d0 (packet destructor for the member handled by 006fff70) and frees the object with _free (00d82463)… |

**PacketWorldTeleportRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1a5e0` | PacketWorldTeleportRequest::Ctor 🌐 | confirmed | Installs the BasePacket and PacketWorldTeleportRequest vtables, stores opcode 0x3a, and copies two 32-bit arguments into packet fields. |
| `00c1a650` | PacketWorldTeleportRequest::ScalarDeletingDtor | confirmed | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |

**PacketZonePing** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1350` | PacketZonePing::PacketZonePing 🌐 | confirmed | Packet constructor: writes BasePacket::vftable, stores packet id 0x60 (96) at this+4 and installs PacketZonePing::vftable at offset 0. No payload fields are in… |
| `008b13b0` | PacketZonePing::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of PacketZonePing::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is s… |

**PacketZoneSafeTeleportRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf7a50` | PacketZoneSafeTeleportRequest::Ctor | confirmed | Constructor body. Stores BasePacket::vftable, sets the opcode dword at this+0x4 to 0x7a (122), then stores PacketZoneSafeTeleportRequest::vftable. No payload f… |

**PetCanGoLocationPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b5a530` | PetCanGoLocationPacket::SerializeBody | confirmed | Writes the common packet prefix, copies the 16-byte block at param_1+0x10, then writes the byte at param_1+0x20. |

**PetListPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b59670` | PetListPacket::~PetListPacket | confirmed | Installs the PetListPacket vtable, destroys its linked-list members, then restores BasePacket vtable. |
| `00b596d0` | PetListPacket::ScalarDeletingDtor | confirmed | Calls the PetListPacket destructor and frees the object only when param_2 bit 0 is set. |

**PetMoodListPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b561d0` | PetMoodListPacket::PetMoodListPacket | confirmed | Calls the PetBasePacket constructor with subtype 0x22, installs PetMoodListPacket and list vtables, and zeros the list fields. |
| `00b56240` | PetMoodListPacket::~PetMoodListPacket | confirmed | Replaces the derived packet vtable with PetMoodListPacket, clears its list, restores the list base vtable, clears that list, then restores BasePacket. |
| `00b562c0` | PetMoodListPacket::ScalarDeletingDtor | confirmed | Calls the packet destructor and frees the object only when the low bit of param_2 is set. |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090d830` | PlayClient::DispatchEncounterPacket 🌐 | confirmed | Parses an encounter-family sub-opcode; for 0x7d constructs the respawn-window packet, updates GroupHandler:SetKO, and forwards two fields to 00904300; 0x7e hid… |

**PlayerUpdateLaunchProjectilePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dccb0` | PlayerUpdateLaunchProjectilePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0b80 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**PlayerUpdatePacketAddNotifications** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcbb0` | PlayerUpdatePacketAddNotifications::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0740 (labelled PlayerUpdatePacketAddNotifications destructor) and frees the object with _free (00d8246… |

**PlayerUpdatePacketCustomizeNpc** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcae0` | PlayerUpdatePacketCustomizeNpc::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d03a0 (a player-update packet destructor) and frees the object with _free (00d82463) when the low bit o… |

**PlayerUpdatePacketEquipItemChange** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcb40` | PlayerUpdatePacketEquipItemChange::PlayerUpdatePacketEquipI… 🌐 | confirmed | Packet constructor: calls family initializer 0089fbb0 with sub-opcode 6 (family 35), installs PlayerUpdatePacketEquipItemChange::vftable, calls 0101cd40 with E… |

**PlayerUpdatePacketLootEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcc90` | PlayerUpdatePacketLootEvent::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0a80 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**PlayerUpdatePacketNpcRelevance** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcbf0` | PlayerUpdatePacketNpcRelevance::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0920 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**PlayerUpdatePacketRemoveNotifications** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcbd0` | PlayerUpdatePacketRemoveNotifications::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0830 (labelled PlayerUpdatePacketRemoveNotifications destructor) and frees the object with _free (00d8… |

**PlayerUpdatePacketRemoveNpcCustomization** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcb00` | PlayerUpdatePacketRemoveNpcCustomization::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0470 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**PlayerUpdatePacketRenamePlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcc10` | PlayerUpdatePacketRenamePlayer::PlayerUpdatePacketRenamePla… 🌐 | confirmed | Packet constructor: calls family initializer 0089fbb0 with sub-opcode 0x13 (19, family 35), installs PlayerUpdatePacketRenamePlayer::vftable and calls the embe… |
| `008dcc70` | PlayerUpdatePacketRenamePlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d09a0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**PlayerUpdatePacketUpdateTintAlias** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcb20` | PlayerUpdatePacketUpdateTintAlias::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d0540 (player-update packet destructor with an IString member at +0x18) and frees the object with _free… |

**ProgressiveQuestRequestClaimPrizePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1a7f0` | ProgressiveQuestRequestClaimPrizePacket::ScalarDeletingDtor | confirmed | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |

**ProgressiveQuestRequestClaimSlotPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1a790` | ProgressiveQuestRequestClaimSlotPacket::ScalarDeletingDtor | confirmed | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |

**ProgressiveQuestRequestStartQuestPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1a730` | ProgressiveQuestRequestStartQuestPacket::ScalarDeletingDtor | confirmed | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |

**QuestAbandonedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7ae40` | QuestAbandonedPacket::ReadFields | likely | Reads the shared prefix, then a 32-bit quest id at output+0xc and one boolean byte at +0x10; missing values are zeroed and set the reader error flag. |
| `00c7b500` | QuestAbandonedPacket::Unserialize | likely | Rejects null inputs, creates a bounded reader, invokes the abandoned-packet field reader, and returns success only without reader error and with full consumpti… |

**QuestAddPacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7bf10` | QuestAddPacket::Construct | confirmed | Calls the base quest-packet constructor with subtype 3, installs QuestAddPacket::vftable, initializes an embedded quest-data record, and clears byte +0x168. |
| `00c7bf80` | QuestAddPacket::DeletingDestruct | confirmed | Calls the embedded-record destructor at 00c7ba90, conditionally frees the packet when flag bit 0 is set, and otherwise returns the object. |
| `00c7ba90` | QuestAddPacket::Destruct | likely | Destroys the embedded quest record by calling FUN_00912450 and restores BasePacket::vftable on the packet object. |
| `00c7cc80` | QuestAddPacket::ReadFields | likely | Reads the common prefix, decodes the embedded quest record through FUN_0093c840, then reads one boolean into packet+0x168; a missing flag is zeroed and marks r… |
| `00c7ccd0` | QuestAddPacket::Unserialize | likely | Rejects null packet or input, creates a bounded reader, parses the common prefix/full quest record/select flag, then verifies no reader error and the caller's … |

**QuestCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7b230` | QuestCompletePacket::ReadFields | confirmed | Reads the common prefix followed by one 32-bit quest id at output+0xc; on a short read it zeroes the field, flags the reader, and clamps the cursor. |
| `00c7b420` | QuestCompletePacket::Unserialize | likely | Rejects null packet or buffer, initializes a bounded reader, calls the quest-complete field parser, then checks reader error and optionally requires full buffe… |

**QuestEndBlockedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7ad30` | QuestEndBlockedPacket::ReadFields | likely | Reads the shared 16-bit/32-bit prefix and then four 32-bit fields at output+0x10..+0x1c, marking short input and clamping the cursor for missing fields. |
| `00c7b3b0` | QuestEndBlockedPacket::Unserialize | likely | Builds a bounded reader over the supplied buffer, calls the end-blocked field reader, and returns true only without reader error and with full consumption unle… |

**QuestEndPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7c0f0` | QuestEndPacket::DeletingDestruct | confirmed | Calls the QuestEndPacket destructor body at 00c79d20, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00c7b990` | QuestEndPacket::ReadFields | likely | Reads the common prefix, an 8-byte value, three 32-bit values, a reward bundle, and a float; short input sets the reader error and NaN progress sets its NaN fl… |
| `00c7bbd0` | QuestEndPacket::Unserialize | likely | Rejects null packet or buffer, parses through a bounded reader, and reports success only when no reader error occurred and the caller's trailing-byte policy is… |

**QuestFailedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7b270` | QuestFailedPacket::ReadFields | likely | Reads the common prefix and one 32-bit quest id at output+0xc; short input zeroes the value, sets the reader error byte, and clamps the cursor. |
| `00c7b490` | QuestFailedPacket::Unserialize | likely | Rejects null packet or buffer, creates a bounded reader, parses the quest id, and succeeds only without reader error and with complete consumption unless trail… |

**QuestInfoPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7bef0` | QuestInfoPacket::DeletingDestruct | confirmed | Calls the QuestInfoPacket destructor body at 00c79470, frees the object only when flag bit 0 is set, then otherwise returns the object pointer. |
| `00c7b7a0` | QuestInfoPacket::ReadFields | likely | Reads the common prefix, six 32-bit fields and a boolean, parses an embedded reward bundle, reads an 8-byte value and float, then two booleans; short fields fl… |
| `00c7bb60` | QuestInfoPacket::Unserialize | likely | Rejects null packet or buffer, initializes a bounded reader and calls the full quest-info parser; returns success only with no reader error and allowed buffer … |

**QuestObjectiveActivatedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7aeb0` | QuestObjectiveActivatedPacket::ReadFields | confirmed | Reads the common prefix, three 32-bit values at output+0xc/+0x10/+0x14 and a boolean at +0x18; short values are zeroed and flag the reader. |
| `00c7b570` | QuestObjectiveActivatedPacket::Unserialize | likely | Rejects null inputs, initializes a buffer-bounded reader, invokes the objective-activation field reader, and checks its error and consumption state. |

**QuestObjectiveAddedPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7bfa0` | QuestObjectiveAddedPacket::Construct | confirmed | Calls the base constructor with subtype 7, installs QuestObjectiveAddedPacket::vftable, stores the quest id at +0xc and initializes embedded objective data. |
| `00c7c010` | QuestObjectiveAddedPacket::DeletingDestruct | confirmed | Calls the objective-added packet destructor body at 00c79810, frees the packet if flag bit 0 is set, and otherwise returns the object pointer. |
| `00c7bd30` | QuestObjectiveAddedPacket::Unserialize | likely | Rejects null inputs, initializes a bounded reader, parses the common prefix and one 32-bit field, then decodes an embedded objective definition; checks reader … |

**QuestObjectiveCompletePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7a7b0` | QuestObjectiveCompletePacket::Construct | confirmed | Calls the base quest-packet constructor with subtype 10, installs QuestObjectiveCompletePacket::vftable, stores three arguments at +0xc/+0x10/+0x14, zeroes +0x… |
| `00c7a830` | QuestObjectiveCompletePacket::ScalarDeletingDtor | confirmed | Replaces the object's vtable with BasePacket::vftable and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
| `00c7b070` | QuestObjectiveCompletePacket::ReadFields | confirmed | Reads the common prefix, two 32-bit values, a float, another 32-bit value, and a boolean; NaN progress sets the reader's NaN flag and short reads set its error… |
| `00c7b650` | QuestObjectiveCompletePacket::Unserialize | likely | Rejects null inputs, initializes a bounded reader, invokes the objective-complete field reader, and validates its error and complete-consumption state. |

**QuestObjectiveFailedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7b160` | QuestObjectiveFailedPacket::ReadFields | likely | Reads the common prefix followed by two 32-bit fields at output+0xc and +0x10; a short field is zeroed, sets the reader error flag and clamps the cursor. |
| `00c7b6c0` | QuestObjectiveFailedPacket::Unserialize | likely | Rejects null inputs, builds a bounded reader, parses the objective-failure fields, and succeeds only when reader error is clear and consumption meets the trail… |

**QuestObjectiveUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7af60` | QuestObjectiveUpdatePacket::ReadFields | confirmed | Reads the common prefix, three 32-bit values, a float at +0x18, a boolean at +0x1c, and a final 32-bit value at +0x20; NaN sets the reader's NaN flag and short… |
| `00c7b5e0` | QuestObjectiveUpdatePacket::Unserialize | likely | Rejects null inputs, sets up a bounded reader, invokes the objective-update body parser, and checks reader errors and the trailing-byte policy. |

**RatingPacketCandidateInfoReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c856c0` | RatingPacketCandidateInfoReplyCtor | confirmed | Constructs BaseRatingPacket with subtype 0x11, installs the reply vtable, and initializes its candidate-info list pointers and count to zero. |
| `00c85730` | RatingPacketCandidateInfoReplyDtor | confirmed | Destroys candidate-info list nodes, then restores BaseRatingPacket and BasePacket vtables. |

**RatingPacketSendFeatured** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c82ad0` | RatingPacketSendFeatured::RatingPacketSendFeatured | confirmed | Initializes BaseRatingPacket with subcommand 0x16, installs RatingPacketSendFeatured vtable, initializes strings and zeroes adjacent fields. |
| `00c831f0` | RatingPacketSendFeatured::ScalarDeletingDtor | confirmed | Calls the packet destructor and conditionally frees the object when flag bit 0 is set. |

**RatingPacketVersion** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c82ca0` | RatingPacketVersion::RatingPacketVersion | confirmed | Initializes BaseRatingPacket with subcommand 0x17, installs RatingPacketVersion vtable, and initializes one embedded IString and scalar fields. |
| `00c83210` | RatingPacketVersion::ScalarDeletingDtor | confirmed | Calls the packet destructor and conditionally frees the object when flag bit 0 is set. |

**RatingPacketVoteReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c81e30` | RatingPacketVoteReply::RatingPacketVoteReply | confirmed | Calls BaseRatingPacket initializer with subcommand 0x12, installs RatingPacketVoteReply::vftable, and returns the object pointer. |
| `00c81e90` | RatingPacketVoteReply::ScalarDeletingDtor | confirmed | Restores the BaseRatingPacket and BasePacket vtables, conditionally frees the object when flag bit 0 is set, and otherwise returns it. |

**RecipeAddPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8b8d0` | RecipeAddPacket::RecipeAddPacket | confirmed | Initializes BaseRecipePacket with type 1, installs RecipeAddPacket vtable, and constructs its embedded RecipeComponentData. |

**RecipeComponentUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8a430` | RecipeComponentUpdatePacket::RecipeComponentUpdatePacket | confirmed | Initializes BaseRecipePacket with type 2, stores values at +0xc and +0x10, and copies an optional dword to +0x14. |
| `00c8a990` | RecipeComponentUpdatePacket::ScalarDeletingDtor | confirmed | Restores the BasePacket vtable and frees the object when deleting flag bit 0 is set. |

**RecipeRemovePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8a370` | RecipeRemovePacket::RecipeRemovePacket | confirmed | Initializes BaseRecipePacket with type 3, installs RecipeRemovePacket vtable, and stores the supplied value at +0xc. |
| `00c8a930` | RecipeRemovePacket::ScalarDeletingDtor | confirmed | Restores the BasePacket vtable and frees the object when deleting flag bit 0 is set. |

**RecipeUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8cc00` | RecipeUpdatePacket::Dtor | confirmed | Destroys embedded recipe map state, restores the BasePacket vtable, and returns. |
| `00c8cf40` | RecipeUpdatePacket::RecipeUpdatePacket | confirmed | Initializes BaseRecipePacket with type 5, installs RecipeUpdatePacket vtable, and constructs its embedded hash list. |
| `00c8cfa0` | RecipeUpdatePacket::ScalarDeletingDtor | confirmed | Calls the packet destructor and frees the object when deleting flag bit 0 is set. |

**ReportPacketReportPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd550` | ReportPacketReportPlayer::ReportPacketReportPlayer 🌐 | confirmed | Packet constructor: calls the report family constructor 008a5650 with sub-opcode 1, installs ReportPacketReportPlayer::vftable, initialises an embedded SoeUtil… |
| `008dd5e0` | ReportPacketReportPlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d1e10 (destroys the IString and the 10-element report vector) and frees the object with _free (00d82463… |

**ShipCombatPacketCameraConfig** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a59520` | ShipCombatPacketCameraConfig::Ctor | confirmed | Base-initializes a ship-combat packet with discriminator 0x0d, installs its vtable, clears a field and initializes a member. |
| `00a59590` | ShipCombatPacketCameraConfig::ScalarDeletingDtor | confirmed | Calls the class cleanup helper, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**ShipCombatPacketCreateProxiedPickup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a59700` | ShipCombatPacketCreateProxiedPickup::Ctor | confirmed | Base-initializes with discriminator 0x1c, installs its packet vtable, copies defaults, initializes a member, and clears trailing fields. |
| `00a597b0` | ShipCombatPacketCreateProxiedPickup::ScalarDeletingDtor | confirmed | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |

**ShipCombatPacketMarkTreasurePickups** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a597d0` | ShipCombatPacketMarkTreasurePickups::ScalarDeletingDtor | confirmed | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |

**ShipCombatPacketPlayerAwards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a59640` | ShipCombatPacketPlayerAwards::ScalarDeletingDtor | confirmed | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |

**ShipCombatPacketSelectShip** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a595b0` | ShipCombatPacketSelectShip::Ctor | confirmed | Base-initializes the packet with discriminator 0x15, installs its vtable, and constructs two string members. |
| `00a59620` | ShipCombatPacketSelectShip::ScalarDeletingDtor | confirmed | Calls the class cleanup helper, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**ShipCombatPacketSpawnProjectile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a59660` | ShipCombatPacketSpawnProjectile::Ctor | confirmed | Base-initializes with discriminator 0x1a, installs the packet vtable, clears fields, and initializes a member. |
| `00a596e0` | ShipCombatPacketSpawnProjectile::ScalarDeletingDtor | confirmed | Calls the projectile-packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |

**SoeUtil::List<AnnouncementDataSendPacket::AnnouncementPacketData,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a40070` | ReadCountedEntryListA | likely | Reads a count-prefixed list of records from the raw reader param_1 into the list at param_2: drains the list through FUN_00a3f6d0, reads one dword as the entry… |

**SoeUtil::List<MemberPanelDataSendPacket::MemberPanelData,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a400e0` | ReadCountedEntryListB | likely | Reads a count-prefixed list from the raw reader param_1 into the list at param_2: drains it through FUN_00a3f760, reads one dword as the entry count (checking … |

**StartInspectPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd230` | StartInspectPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d18f0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |

**StartTimerPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86c60` | StartTimerPacketDtorBody | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and one M… |

**StopInspectPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b7730` | StopInspectPacket::ScalarDeletingDestructor | likely | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**TaskAddPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86850` | TaskAddPacketDtorBody | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and from … |

**TellEchoPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0fab0` | TellEchoPacket::ScalarDeletingDtor | confirmed | Calls the string-bearing base chat destructor and frees the object when the low flag bit is set. |

**(no class)** (460)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089b950` | SystemMessageColorCode | confirmed | Maps the style word at param_1+0x10 to a color literal: 1 -> "0x00FF00", 2 -> "0x0000FF", 3 -> "0xFFFF00", 4 -> "0xFFFFFF", default -> "0xFF0000"; returns the … |
| `008bfc00` | ReadBoundedLengthBlob | confirmed | Reads a signed 32-bit length from the bounded reader, rejects negative or out-of-range lengths by setting its error flag and moving the cursor to end, otherwis… |
| `008cee70` | PacketDtor_FixedStrings2048_64 | confirmed | Destructor body of a packet that embeds fixed-capacity strings: runs the IStringFixed<char,2048> destructor body FUN_00402420 first and then the IStringFixed<c… |
| `008cf0e0` | PacketDtor_FourFixed256 | confirmed | Destructor body of a packet with four IStringFixed<char,256> members: calls the confirmed IStringFixed256 destructor body FUN_00402140 four times (one per memb… |
| `008dadd0` | ReaderReadActionBarIdAndSlot 🌐 | confirmed | Reads two i32 values from the bounded reader param_2 into param_1[0] and param_1[1]; a short read writes 0, sets the reader error byte +0x10 and clamps the cur… |
| `008dae30` | ReaderReadActionBarSlot51 🌐 | confirmed | Reads one fixed 51-byte action-bar slot from the bounded reader param_2 into param_1: bool at +8, i32 fields at +0xc,+0x10,+0x14,+0x18,+0x1c, f32 at +0x20 (a N… |
| `008e8460` | ReadTwoU32AndLengthPrefixedBytes | confirmed | Reads two 32-bit values and a signed-length byte span, zeroing outputs and flagging short input. |
| `008e8510` | ReadU32TwoStringsU32AndString | confirmed | Reads a U32 at destination +0, consumes two signed-length strings, reads a U32 at destination +0x24, then consumes a third signed-length string. Bounded-read f… |
| `00925fe0` | BuildTunneledClientPacket 🌐 | confirmed | Serializes through 00913b80, wraps those bytes in a client packet, and finalizes a BasePacket object. |
| `009260f0` | BuildTunneledClientPacket 🌐 | confirmed | Serializes through 0090b330, wraps those bytes in a client packet, and finalizes a BasePacket object. |
| `00926200` | BuildTunneledClientPacket 🌐 | confirmed | Serializes through 00913bf0, wraps those bytes in a client packet, and finalizes a BasePacket object. |
| `00928f00` | BuildTunneledClientPacketWithU16Prefix 🌐 | confirmed | Writes the source's low 16-bit value to a temporary buffer, wraps it with the supplied byte, and finalizes a BasePacket. |
| `00929040` | BuildTunneledClientPacket 🌐 | confirmed | Serializes the source through 009125a0, wraps the payload with the supplied byte, and finalizes a BasePacket. |
| `00929150` | BuildTunneledClientPacket 🌐 | confirmed | Serializes the source through 00912790, wraps the payload with the supplied byte, and finalizes a BasePacket. |
| `00929260` | BuildTunneledClientPacket 🌐 | confirmed | Serializes the source through 009127f0, wraps the payload with the supplied byte, and finalizes a BasePacket. |
| `00929370` | BuildTunneledClientPacket 🌐 | confirmed | Serializes the source through 00912a80, wraps the payload with the supplied byte, and finalizes a BasePacket. |
| `00929560` | BuildTunneledClientPacket 🌐 | confirmed | Serializes the source through 009130e0, wraps the payload with the supplied byte, and finalizes a BasePacket. |
| `00929760` | BuildTunneledClientPacketWithHandle 🌐 | confirmed | Serializes the record, writes an eight-byte handle and further fields, wraps the bytes, and finalizes a BasePacket. |
| `009298f0` | BuildTunneledClientPacket 🌐 | confirmed | Serializes the source through 0091b720, wraps those bytes with the supplied byte, and finalizes a BasePacket. |
| `00929b40` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes through 009133c0, wraps bytes in a client-world packet envelope, and finalizes a BasePacket. |
| `00929c50` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes through 0091b7a0, wraps bytes in a client-world tunnel envelope, and finalizes a BasePacket. |
| `00929d60` | BuildTunneledClientPacket 🌐 | confirmed | Serializes through 00913470, wraps bytes in a client-packet tunnel envelope, and finalizes a BasePacket. |
| `00929e70` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes through 00913680, wraps bytes in a client-world tunnel envelope, and finalizes a BasePacket. |
| `00929f80` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes a header and source handle, wraps the payload in a client-world envelope, and finalizes a BasePacket. |
| `0092a0a0` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes a header and source handle, wraps the payload in a client-world envelope, and finalizes a BasePacket. |
| `0092a1b0` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes a header and handle field, wraps bytes in the client-world tunnel envelope, and finalizes a BasePacket. |
| `0092a2d0` | BuildTunneledClientPacket 🌐 | confirmed | Serializes through 00913f70, wraps payload with the supplied byte, and finalizes a BasePacket. |
| `0092a3e0` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes through 00913f70, wraps payload in a client-world envelope, and finalizes a BasePacket. |
| `0092a4f0` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes through 00913a40, wraps payload in a client-world envelope, and finalizes a BasePacket. |
| `0092a600` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes a four-word header, eight-byte value, and trailing byte before building a client-world tunnel packet. |
| `0092a710` | BuildTunneledClientPacket 🌐 | confirmed | Serializes header, string, fixed-width fields, and two fixed-size groups through 0091b8d0, then finalizes a BasePacket. |
| `0092a820` | BuildTunneledClientPacket 🌐 | confirmed | Serializes two 16-bit fields, a string, a 32-bit field, and fixed-size list records before finalizing a BasePacket. |
| `0092a930` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes a header, string, and two 32-bit fields, wraps bytes in a client-world envelope, and finalizes a BasePacket. |
| `0092aa50` | BuildTunneledClientPacket 🌐 | confirmed | Serializes a four-word header and trailing byte, wraps it with the supplied byte, and finalizes a BasePacket. |
| `0092ab60` | BuildTunneledClientWorldPacket 🌐 | confirmed | Serializes a header pair and 32-bit field, wraps payload in a client-world envelope, and finalizes a BasePacket. |
| `009afba0` | BasePacketByteArrayDtor | confirmed | Destroys the byte-array member through FUN_009af0a0 and restores BasePacket vtable. |
| `009c9070` | HandleMysteryChestInfoReply 🌐 | confirmed | Initializes the packet base, accepts subtype 2, parses the reply map, copies records into the data source, updates it, and logs entry count and timestamp. |
| `009f6630` | BuildTunneledClientPacket 🌐 | confirmed | Initializes a packet writer, serializes param_1 with 0090ac70, constructs a stack PacketTunneledClientPacket, and fills its length/data fields plus one byte fr… |
| `009f6740` | BuildTunneledClientPacketDword10 🌐 | confirmed | Initializes a packet writer, serializes param_1 with 009f6590 (including the dword at source+0x10), constructs a stack PacketTunneledClientPacket, and fills it… |
| `00a05390` | HandleGetAllAvailableClaimItemsResponse | confirmed | Constructs GetAllAvailableClaimItemsResponsePacket in a stack temporary, passing the manager field at +0x9c as the constructor argument (stored at packet+0x10)… |
| `00a05490` | HandleClaimItemsResponse | confirmed | Receive handler for the 108/4 answer: constructs ClaimItemsResponsePacket over the manager field at +0x9c, parses the body with 00a049b0 (tracker id + success … |
| `00a056a0` | ClaimPacketDispatcher 🌐 | confirmed | Receive dispatcher of the claim family: constructs a BaseClaimPacket temporary with sub-opcode 0 (009ff290(0)), reads the two 16-bit header fields with 00a02a6… |
| `00a3e000` | BasePacket_DtorBody_00a3e000 | confirmed | Destructor body: rewrites BasePacket::vftable at +0 and returns; no member is released. |
| `00a3e0b0` | BasePacket_DtorBody_00a3e0b0 | confirmed | Destructor body: rewrites BasePacket::vftable at +0 and returns, with no member release. |
| `00a40aa0` | AnnouncementsSubsystem_HandlePacket 🌐 | confirmed | Incoming announcement-family handler: reads the sub-opcode from the payload reader, and for sub-opcode 2 constructs an AnnouncementDataSendPacket (00a40460), p… |
| `00a83790` | SendTunneledPacketU16PairHandleB 🌐 | confirmed | Identical clone of 00a83670 (same instruction sequence apart from relative call displacements): packs a caller struct (two 16-bit fields plus an 8-byte handle … |
| `00a838b0` | SendTunneledPacketU16PairHandleC 🌐 | confirmed | Identical clone of 00a83670/00a83790: packs a caller struct (two 16-bit fields plus an 8-byte handle at +0x10) into an 8192-byte buffer, wraps it in a PacketTu… |
| `00a89940` | PacketIStringPairDtorBody | confirmed | Destructor body for a packet object (BasePacket layout) with two IString<char> members at +0xc and +0x1c: sets the +0x1c member vptr, releases its buffer throu… |
| `00a8d900` | ReadWordAndCheckBufferEnd 🌐 | confirmed | Rejects null inputs, initializes a bounded reader over the supplied buffer, reads one word, and returns success only when parsing had no error and either trail… |
| `00a8d970` | ReadWordAndCheckBufferEndVariant 🌐 | confirmed | Rejects null inputs, initializes a bounded reader over the supplied buffer, reads one word, and returns success only when parsing had no error and either trail… |
| `00a8d9e0` | ReadLengthPrefixedScriptRecord 🌐 | confirmed | Validates the input pointers, decodes the header and length-prefixed payload, and returns success only when the bounded parse succeeds and consumes the full bu… |
| `00a8da50` | ReadTwoLengthPrefixedBlocks 🌐 | confirmed | Validates the input pointers, decodes a header and two length-prefixed blocks, and returns success only when the bounded parse succeeds and consumes the full b… |
| `00a8dac0` | ReadWordPayloadAndCheckEnd 🌐 | confirmed | Validates the input pointers, reads a word from the bounded payload, and returns success only when parsing had no error and either trailing bytes are allowed o… |
| `00a8db30` | ReadOptionalRecordAndCheckEnd 🌐 | confirmed | Validates the input pointers, decodes a header and an optional record, then returns success only when parsing had no error and the full buffer was consumed. |
| `00a8dba0` | ReadBytePayloadAndCheckEnd 🌐 | confirmed | Validates the input pointers, reads one byte from the bounded payload, and returns success only when parsing had no error and either trailing bytes are allowed… |
| `00a8dc10` | ReadCompositePayloadAndCheckEnd 🌐 | confirmed | Validates the input pointers, delegates decoding of a composite record, and returns success only when parsing had no error and either trailing bytes are allowe… |
| `00a8f580` | ReadRecordAndCheckBufferEnd 🌐 | confirmed | Rejects null inputs, decodes a mixed-width record through a bounded reader, and returns success only when parsing had no error and either trailing bytes are al… |
| `00a8f5f0` | ReadTwoWordRecordAndCheckBufferEnd 🌐 | confirmed | Rejects null inputs, decodes two 32-bit words through a bounded reader, and returns success only when parsing had no error and either trailing bytes are allowe… |
| `00a8f6e0` | ReadCountedNodeList 🌐 | confirmed | Reads a signed record count, rejects negative or out-of-bounds counts, advances the reader by the validated payload length, then decodes records into the desti… |
| `00a8f7c0` | WriteBaseUiPacketPrefix 🌐 | confirmed | Writes the low 16 bits of source+4, then the byte at source+8, clamping each write to the remaining bounded output capacity. |
| `00a91b00` | ReadCountedScriptStringList 🌐 | confirmed | Validates pointers, creates a bounded reader, delegates count and element decoding to 00a8f6e0, and reports success only when parsing completes without error a… |
| `00a95f20` | SerializeAndSendTunneledClientPacket_A914A0 🌐 | confirmed | Serializes with 00a914a0, constructs PacketTunneledClientPacket and submits it through 00925640; 00894560 only releases the temporary buffer. |
| `00a96030` | SerializeAndSendTunneledWorldPacket_A91500 🌐 | confirmed | Serializes with 00a91500, constructs PacketTunneledClientWorldPacket and submits it through 00926360; 00894560 only releases the temporary buffer. |
| `00a96140` | SerializeAndSendTunneledWorldPacket_A91570 🌐 | confirmed | Serializes with 00a91570, constructs PacketTunneledClientWorldPacket and submits it through 00926360; 00894560 only releases the temporary buffer. |
| `00a96250` | SerializeAndSendTunneledWorldPacket_A915D0 🌐 | confirmed | Serializes with 00a915d0, constructs PacketTunneledClientWorldPacket and submits it through 00926360; 00894560 only releases the temporary buffer. |
| `00a96360` | SerializeAndSendTunneledClientPacket_A91630 🌐 | confirmed | Serializes with 00a91630, constructs PacketTunneledClientPacket and submits it through 00925640; 00894560 only releases the temporary buffer. |
| `00a96470` | SerializeAndSendTunneledClientPacket_A916D0 🌐 | confirmed | Serializes with 00a916d0, constructs PacketTunneledClientPacket and submits it through 00925640; 00894560 only releases the temporary buffer. |
| `00a96580` | SerializeAndSendTunneledClientPacket_A91730 🌐 | confirmed | Serializes with 00a91730, constructs PacketTunneledClientPacket and submits it through 00925640; 00894560 only releases the temporary buffer. |
| `00b1f990` | BasePacketVftableCleanup_00b1f990 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1f9e0` | BasePacketVftableCleanup_00b1f9e0 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fa30` | BasePacketVftableCleanup_00b1fa30 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fa80` | BasePacketVftableCleanup_00b1fa80 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fb20` | BasePacketVftableCleanup_00b1fb20 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fb70` | BasePacketVftableCleanup_00b1fb70 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fbc0` | BasePacketVftableCleanup_00b1fbc0 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fc10` | BasePacketVftableCleanup_00b1fc10 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fc60` | BasePacketVftableCleanup_00b1fc60 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fcb0` | BasePacketVftableCleanup_00b1fcb0 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fd50` | BasePacketVftableCleanup_00b1fd50 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b1fda0` | BasePacketVftableCleanup_00b1fda0 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00b20240` | BasePacketVftableCleanup_00b20240 | confirmed | Writes BasePacket::vftable to the object start and returns. |
| `00c6a820` | BasePacketDtor_00c6a820 | confirmed | Trivial destructor body: writes BasePacket::vftable into this+0 and returns, touching no other field and freeing nothing. |
| `00c6a910` | BasePacketDtor_00c6a910 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6a9f0` | BasePacketDtor_00c6a9f0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6aac0` | BasePacketDtor_00c6aac0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6ab80` | BasePacketDtor_00c6ab80 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6ac40` | BasePacketDtor_00c6ac40 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6ad10` | BasePacketDtor_00c6ad10 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6add0` | BasePacketDtor_00c6add0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6aea0` | BasePacketDtor_00c6aea0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6afa0` | BasePacketDtor_00c6afa0 | confirmed | Destructor body that first destroys the item member at +0x18 through FUN_0103dd90 (an ItemInstance destructor) and then stores BasePacket::vftable at +0. |
| `00c6b0c0` | BasePacketDtor_00c6b0c0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6b1d0` | BasePacketDtor_00c6b1d0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6b2b0` | BasePacketDtor_00c6b2b0 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6b370` | BasePacketDtor_00c6b370 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6b450` | BasePacketDtor_00c6b450 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00c6b540` | BasePacketDtor_00c6b540 | confirmed | Trivial destructor body: stores BasePacket::vftable at +0 and returns. |
| `00cda880` | SerializePayloadFields_00cda880 🌐 | confirmed | Writes the packet header, serializes three four-dword fields from +0x20, +0x30, and +0x40, then writes dwords from +0x50 and +0x54 and one byte from +0x58. |
| `00cda990` | SerializePayloadFields_00cda990 🌐 | confirmed | Writes the packet header, bytes from +0x18 and +0x19, a dword from +0x1c, and three four-dword fields from +0x20, +0x30, and +0x40. |
| `00cdaaa0` | SerializePayloadFields_00cdaaa0 🌐 | confirmed | Writes the packet header, one byte from +0x0c, a four-dword field from +0x10, and the data field at +0x20. |
| `00cdab10` | SerializePayloadFields_00cdab10 🌐 | confirmed | Writes the packet header, a field from +0x10, then dwords from +0x18, +0x1c, and +0x20. |
| `00cdb000` | SerializePayloadFields_00cdb000 🌐 | confirmed | Writes the packet header, five four-dword values from +0x10 through +0x50, bytes from +0x60 and +0x61, dwords from +0x64 and +0x68, and a byte from +0x6c. |
| `008a4920` | PacketDtor_008a4920 | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; callers are the two large functions 0093af90/0093b340 and EH funclets 0159… |
| `008a6a90` | PacketDtor_008a6a90 | likely | Destructor body: writes BasePacket::vftable and returns; callers are the 009471a0 packet router and EH funclets 01584e30, 01593c8a, 0163ddc6 and 0163de46. |
| `008cf060` | PacketDtor_WithIString8b | likely | Destructor body: releases the embedded SoeUtil::IString at +8 (capacity +0x14) through the locked refcount and virtual slot +8, then resets the vtable to BaseP… |
| `008cf190` | PacketDtor_TwoMembers006fff70 | likely | Destructor body: runs the SoeUtil::IStringFixed<char,512> destructor body FUN_006fff70 twice (once per fixed-string member, with separate cleanup states), then… |
| `008cf210` | PacketDtor_Member00792630 | likely | Destructor body: runs the SoeUtil::IStringFixed<char,8> destructor body FUN_00792630 and restores BasePacket::vftable. |
| `008cf270` | PacketDtor_OneFixed256 | likely | Destructor body: runs the IStringFixed<char,256> destructor body FUN_00402140 once and then restores BasePacket::vftable. |
| `008cf2d0` | PacketDtor_Member006fff70 | likely | Destructor body: runs the SoeUtil::IStringFixed<char,512> destructor body FUN_006fff70 once and then restores BasePacket::vftable. |
| `008cf3b0` | PacketDtor_OneFixed64 | likely | Destructor body: runs the IStringFixed<char,64> destructor body FUN_004022b0 once and then restores BasePacket::vftable. |
| `008cf410` | PacketDtor_DelegatesTo008ceaf0 | likely | Destructor body: runs the three-string destructor body FUN_008ceaf0 (IString members at +0/+0x10/+0x20) and then restores BasePacket::vftable. |
| `008cf4c0` | Ctor_PacketIdThreeStrings | likely | Shared base constructor for a packet family: stores the caller's id/opcode word at +0, then initializes three embedded SoeUtil::IString members (+4, +0x14, +0x… |
| `008d1bd0` | DtorWrapper_BaseBroadcastPacket | likely | Thin destructor wrapper: forwards the incoming this pointer to BaseBroadcastPacket::~BaseBroadcastPacket (008d1b30) inside an SEH frame and returns; it perform… |
| `008d4a40` | PacketWriterInit | likely | Initializes a packet writer/stream object: calls FUN_00894870, clears the byte flags at +0x201c and +0x201d, stores the caller's cap argument at +0x2018, zeroe… |
| `008e4ae0` | ReadBeginZoningBody 🌐 | likely | thiscall: ECX = output record, one stack argument = bounded reader. Reads an int16 (sign-extended) to +4; a length-prefixed string via 00894b10 with ECX = reco… |
| `008e82e0` | ReadThreeLengthPrefixedStringsAndScalars | likely | Processes three signed length-prefixed byte strings through 00894b10, then reads U32 values at destination +0x30 and +0x34 and boolean bytes at +0x38 and +0x39… |
| `008e9710` | ReadU32LengthPrefixedStringAndTwoU32 | likely | Reads a U32 to destination +0, reads a signed length and passes the bounded bytes to 00894b10 for the IString at +4, then reads U32 values at +0x14 and +0x18. … |
| `008f6a70` | ReadBodyVia_008d57f0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the 0x14-byte bounded reader {start,… |
| `008f6ae0` | ReadBodyVia_008d5850 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6b90` | ReadBodyVia_008d5ba0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6c00` | ReadBodyVia_008d5c00 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6c70` | ReadBodyVia_008d5c60 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6ce0` | ReadBodyVia_008d5d20 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6d50` | ReadBodyVia_008d5dd0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6dc0` | ReadBodyVia_008d5e80 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6e30` | ReadBodyVia_008d5ee0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6ea0` | ReadBodyVia_008d5f40 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6f10` | ReadBodyVia_008d60b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6f80` | ReadBodyVia_008d61c0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f6ff0` | ReadBodyVia_008d6250 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7060` | ReadBodyVia_008d62b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f70d0` | ReadBodyVia_008d6310 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7140` | ReadBodyVia_008d6370 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f71b0` | ReadBodyVia_008d63f0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7220` | ReadBodyVia_008d6460 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7290` | ReadBodyVia_008d64f0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7300` | ReadBodyVia_008d6600 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f9040` | ReadBodyPairVia_008e4060 | likely | Cdecl: destination, start, length, allowTrailing (plain ret). Returns 0 when the destination or start is null. Builds a bounded reader, calls 008e4060(reader, … |
| `00909cd0` | WriterAppendDwordBounded | likely | Thiscall, ECX = writer object, param_2 = pointer to a dword. Copies the dword into a local, computes the remaining bytes n = writer[0x806] - (writer's header +… |
| `0090a350` | ReadDwordPairThenU32Strings_A | likely | Thiscall, ECX = record, param_2 = reader. Calls 008d6830(reader), then reads two dwords into record+0x10 and +0x14 (on overflow zeros both, sets the fail byte,… |
| `0090a3b0` | ReadDwordPairThenU32Strings_B | likely | Byte-identical logic to 0090a350 (86 bytes): calls 008d6830(reader), reads two dwords into record+0x10 and +0x14 with the overflow rule, then calls 008e8510(re… |
| `00914290` | ListIStringMemberDtorBody | likely | __fastcall (this=ECX). Under an SEH frame with a cookie check: stores List<SoeUtil::IString<char>,-1> vftable 0x017eb8ac at [this+8], calls FUN_007820a0 (recei… |
| `009174c0` | HandleBroadcastPacket | likely | Thiscall with (param_2 = begin, param_3 = length). Constructs a local BaseBroadcastPacket (FUN_008dd380) and decodes the byte range through FUN_008fb0a0. Picks… |
| `0091a640` | AppendFourDwordsToWriter | likely | Plain function with two stack-passed pointers: param_1 is the writer context and param_2 is the source. Repeats four times: loads one source dword, computes th… |
| `0091a6c0` | ReadU16FieldThenCallNext | likely | Takes reader and output pointers from the stack. If cursor+2 exceeds end, stores 0 at output+4, sets reader error byte +0x10 and clamps cursor to end; otherwis… |
| `0091a9b0` | ReadU32FieldThenCallNext | likely | Takes reader and output pointers from the stack. If cursor+4 exceeds end, stores 0 at output+4, sets reader error byte +0x10 and clamps cursor to end; otherwis… |
| `0091b530` | ReadCountPrefixedList_Replacing | likely | First drains any existing list at param_2 by repeatedly calling FUN_00909e60 on the head at param_2+4, then reads an i32 count from the reader at param_1 (curs… |
| `0091b720` | WriteU16_IString2 | likely | Serializes a record into the packet writer at param_2: writes the low 16 bits of this+4 via the clamped byte-buffer helper FUN_007cc400, then serializes two So… |
| `0091b7a0` | WriteU16_IString1 | likely | Serializes a record into the writer at param_2: writes the low 16 bits of this+4 as a clamped 2-byte field via FUN_007cc400 and then one SoeUtil::IString<char>… |
| `0091b870` | WriteHeader_U64_U8 | likely | Writes a packet header via FUN_00913db0 (u16 main, u16 sub, i32 and i32 header words taken from this+4/+8/+0xc/+0x10), then an eight-byte value (two u32) from … |
| `0091b8d0` | WriteU16U8_IString_U32x2_Blob16x2_U32 | likely | Writes a header pair via FUN_0090b0f0 (u16 plus u8), one IString from this+0x10 via FUN_008fb810, two u32 from this+0x18 and this+0x1c, two 16-byte groups from… |
| `0091b9d0` | WriteU16x2_IString_U32_ListOf30 | likely | Writes a two-u16 header from this+4/+8 via FUN_0090b190, one IString from this+0xc via FUN_00911c30, a u32 count/word from this+0x1c, then loops that count ove… |
| `0091bad0` | WriteHeader_U8 | likely | Writes the four-word packet header via FUN_00913db0 from this+4/+8/+0xc/+0x10 and then one byte from this+0x14. |
| `0091bd40` | WriteHandle_U32_Blob16x2_U8_IString2_U32x3 | likely | Serializes a record into the writer at param_2: an eight-byte handle from this+0 via FUN_008fb810, a u32 from this+8, two 16-byte groups from this+0x10 and thi… |
| `0091c7e0` | Ctor_EmbedRewardBundleAt0xc0 | likely | Constructor of an object allocated by the caller: zeroes this+4/+8/+0xc, memsets this+0x10 for 0x8c bytes, sets this+0 to 0, zeroes +0x9c/+0xa8, constructs a s… |
| `00922570` | WriteShortFromPlus4 | likely | Copies the low 16 bits of the dword at this+4 into the output buffer at param_2+0x2020 through 007cc400, copying min(2, remaining) bytes where remaining = para… |
| `00926360` | WriteRecordThroughPacketVirtual | likely | Serializes a temporary record, then dispatches its bytes to packet virtual slot 0x50 or slot 0x54 depending on the flag. |
| `009264b0` | ReadU16BoolStringsU32Pair 🌐 | likely | Reads a 16-bit value, boolean, two delegated strings, and two 32-bit values; bounds failures clear fields and set reader error. |
| `00929a00` | BuildTunneledClientPacketWithU16Prefix 🌐 | likely | Writes the source's low 16-bit field into a temporary buffer, wraps it, and finalizes a BasePacket. |
| `00939200` | ParseWelcomeScreenPayload 🌐 | likely | Constructs PacketLoadWelcomeScreen and its embedded arrays, parses the supplied bounded byte range, calls 00ad0790 only when the reader error flag is clear and… |
| `0094a110` | BuildClientLuaMetricsAndTunnelRecord 🌐 | likely | When the byte at (*(param_1+0x20)+0x104) is set, constructs PacketClientLuaMetrics at (*(param_1+0x20)+0xa0); if param_1+0x280 is nonzero, calls 00945bf0 with … |
| `009c2740` | ReadBoundedHeaderAndReturnError | likely | Builds a bounded reader over the supplied span, calls 009c1e00 to read the shared header fields, and returns the reader error byte. |
| `009c7ed0` | ReadMysteryChestInfoRecord | likely | Reads two boolean bytes and ten 32-bit values into the record at offsets +0x4 through +0x28; each unavailable field is zeroed and sets the cursor failure flag. |
| `009c83b0` | ReadMysteryChestReplyHeader | likely | Reads a 16-bit value and then an 8-bit value from the bounded packet cursor, storing zero and setting the cursor failure flag when unavailable. |
| `009c8940` | ReadMysteryChestInfoReplyRecords | likely | Clears existing map nodes, reads a record count, then for each entry reads a dword key, allocates and links its keyed node through 009c8410, and decodes the fo… |
| `009c8e90` | WriteTwoByteAndOneByteFields 🌐 | likely | Writes at most two bytes from object +4 and one byte from +8 into a bounded output cursor, advancing it after each write. |
| `009c9540` | SendMysteryChestInfoRequest 🌐 | likely | Constructs the Mystery Chest info request, checks gateway-ready field +0x280, and sends with flag 1 when ready. |
| `009e6860` | PacketBuffer_WriteU16Pair 🌐 | likely | Appends two 16-bit values into a packet byte buffer: writes the low 16 bits of this+4 and then of this+8, and for each value clamps the copy length to the spac… |
| `009e6960` | PacketBuffer_WriteU32Pair 🌐 | likely | Appends two 32-bit values into a packet byte buffer: copies *this and this[1] with the copy length clamped to the remaining capacity (min(4, capacity-used)) an… |
| `009fc2e0` | SerializeBugRecordFields 🌐 | likely | Serializes the 16-bit pair at record offsets +4/+8 through FUN_009fc240, writes the bounded dword at +0x0c, serializes the IString at +0x10, writes the bounded… |
| `009fc890` | BuildTunneledClientPacket 🌐 | likely | Initializes a bounded packet writer, serializes a bug record into a local buffer, constructs a ClientGateway tunneled-client-packet object, then cleans up the … |
| `009ff300` | ClaimPacket_TeardownBodyA | likely | Out-of-line claim packet teardown body: writes BaseClaimPacket::vftable into this and then BasePacket::vftable, with no member work at all; 24 compiler unwind … |
| `009ff7d0` | ClaimPacket_TeardownBodyB | likely | Second out-of-line claim packet teardown body, identical in shape to 009ff300: SEH frame, writes BaseClaimPacket::vftable then BasePacket::vftable, no member w… |
| `00a02a60` | ReadTwoU16Fields | likely | Reads two 16-bit fields from a bounded byte reader (cursor at +8, end at +0xc, error flag at +0x10) into param_1+4 and param_1+8; on truncation it stores 0, se… |
| `00a02ac0` | ReadClaimItemHeaderFields | likely | Reads the fixed head of a claim item off a bounded reader: two 16-bit fields through 00a02a60 into +4/+8, a 32-bit field into +0xc, a boolean into +0x10 (with … |
| `00a033e0` | ReadU32IntoField14 | likely | Reads one 32-bit value from a bounded reader after running the nested reader 008cb540 on the same object, storing it into param_2+0x14 (0 on truncation with th… |
| `00a03880` | ReadListOfItemIdEntries | likely | Reads a 32-bit count from a bounded reader, resizes the 0x18-byte-element array behind param_2 (00a01200 to grow, 009ffda0 to shrink) and then reads that many … |
| `00a03ed0` | WriteClaimPacketHeader | likely | Writes the two 16-bit header fields of a claim packet into the bounded byte writer: first the base-opcode field at this+4, then the sub-opcode field at this+8,… |
| `00a03fc0` | WriteInt32ArrayField | likely | Writes a 32-bit element count through 00911420 and then every 32-bit element of the array at param_2+4 through the bounded writer 007cc400, advancing the curso… |
| `00a040e0` | ReadClaimItem | likely | Reads one 0x168-byte claim item: the id list at +4 through 00a02b40, a 32-bit field at +0x144, a 32-bit field at +0x148, a string through 008caaa0, the 0x18-by… |
| `00a04520` | WriteClaimItemsItemDefinitionsRequest | likely | Serialises a ClaimItemsItemDefinitionsRequest: the claim packet header through 00a03ed0, then the 32-bit field at this+0xc through the bounded writer 007cc400,… |
| `00a045d0` | WriteClaimItemsRequest | likely | Serialises a ClaimItemsRequestPacket: the claim packet header through 00a03ed0, the 32-bit field at this+0xc (tracker id) through 007cc400 and, when the id arr… |
| `00a04640` | ReadClaimItemArray | likely | Reads a 32-bit count, resizes the Array<ClaimItem> at param_2+4 (grow via 00a01070, truncate via 009fff10) and then reads that many claim items with 00a040e0 a… |
| `00a04950` | ReadGetAllAvailableClaimItemsResponseFields | likely | Reads the body of a 108/2 response into a temporary packet struct: two 16-bit header fields through 00a02a60, a boolean at +0xc (error flag on truncation) and,… |
| `00a049b0` | ReadClaimItemsResponseFields | likely | Reads the body of a 108/4 response: two 16-bit header fields through 00a02a60, a 32-bit field at +0xc (the tracker id), a boolean at +0x10 and, when the item a… |
| `00a0de80` | Packet_WriteTwoU16 | likely | Writes two 16-bit little-endian values to the byte buffer param_2: first the low 16 bits of *(this+4), then the low 16 bits of *(this+8), each time growing the… |
| `00a0e8f0` | Packet_Write3I32And2Blobs | likely | Writes three 32-bit fields (this+0x7c, this+0x80, this+0x84) into the byte buffer param_2 through FUN_007cc400 (each time growing to 4 bytes if the remaining c… |
| `00a0ece0` | Packet_WriteRecordAndBlobs 🌐 | likely | Serialises a record into the byte buffer param_2: calls FUN_0090ac70(this, buffer) which writes two 16-bit fields from this+4/this+8, writes the 8-byte pair at… |
| `00a0ed50` | Send_GuildPacketBody2U16 🌐 | likely | Builds and sends a packet: initialises a 0x2000-byte buffer with FUN_008d4a40(0x10000000), writes the packet body with FUN_00a0de80 (the two u16 fields at +4/+… |
| `00a0ee70` | Send_GuildInviteAcceptBody 🌐 | likely | Identical to 00a0ed50 (only the SEH label differs): initialises a 0x2000-byte buffer (FUN_008d4a40), writes the packet body with FUN_00a0de80, writes the 8-byt… |
| `00a0ef90` | Send_GuildInviteTimeoutBody 🌐 | likely | Identical to 00a0ed50/00a0ee70 apart from its SEH label: initialises a 0x2000-byte buffer, writes the two-u16 body via FUN_00a0de80, writes the 8-byte field at… |
| `00a0f130` | SendGuildInviteDecline 🌐 | likely | Constructs a GuildInviteDeclinePacket on the stack (FUN_00a09e90 with sub-opcode 9) and, only when the guild manager object reached through FUN_00769710()+4 ha… |
| `00a0f1d0` | SendGuildInviteAccept 🌐 | likely | Constructs a GuildInviteAcceptPacket (FUN_00a09db0, sub-opcode 8), and when the guild manager field at +0x280 is non-null calls FUN_00a0ee70 to serialise and s… |
| `00a0f290` | SendGuildInviteTimeout 🌐 | likely | Constructs a GuildInviteTimeOutPacket (FUN_00a09f70) and, when the guild manager field at +0x280 is non-null, calls FUN_00a0ef90 to serialise and transmit it. |
| `00a0f330` | Send_ConfirmFriendBody 🌐 | likely | Serialises the packet object param_1 into a 0x2000-byte buffer: initialises the buffer with FUN_008d4a40(0x10000000), calls FUN_00a0ece0 (two u16 opcode fields… |
| `00a0f460` | SendConfirmFriendResponse 🌐 | likely | Constructs a CommandPacketConfirmFriendResponse at this+0x10 with FUN_00a0a500 (which sets sub-opcode 0x11 = 17 on a BaseCommandPacket and stores three further… |
| `00a0f500` | SendConfirmFriendResponseCopy2 🌐 | likely | Byte-identical copy of 00a0f460 apart from its SEH label: constructs CommandPacketConfirmFriendResponse at this+0x10 (FUN_00a0a500), sends it through FUN_00a0f… |
| `00a0f5a0` | SendConfirmFriendResponseCopy3 🌐 | likely | Byte-identical copy of 00a0f460 apart from its SEH label: builds CommandPacketConfirmFriendResponse at this+0x10, transmits it with FUN_00a0f330 and destroys i… |
| `00a104e0` | ItemRecord_Sub_Dtor_EquipByItemRecord | likely | Partial destructor: resets the embedded ItemRecord vtable at this+0xc, then the BaseInventoryPacket vtable and finally the BasePacket vtable (the standard MSVC… |
| `00a10540` | ItemRecord_Sub_Dtor_ActionBarAssignByItemRecord | likely | Partial destructor variant that resets the ItemRecord vtable at this+0x10 (one field further along), then the BaseInventoryPacket and BasePacket vtables. |
| `00a122c0` | ItemRecord_Sub_ScalarDeletingDtor | likely | Scalar deleting destructor wrapper: calls FUN_00a104e0 (the record sub-object destructor body) and frees the object with _free when bit 0 of param_2 is set. |
| `00a12360` | ItemRecord_Sub_ScalarDeletingDtor2 | likely | Scalar deleting destructor wrapper: calls FUN_00a10540 (the +0x10 record sub-object destructor body) and frees the object with _free when bit 0 of param_2 is s… |
| `00a167e0` | PacketReader_ReadGuidAndBool | likely | Reads two 32-bit values into dest+0x10 and dest+0x14 plus a boolean byte into dest+0x18, calling the u16/u8 reader first and setting the overflow flag with the… |
| `00a16850` | PacketReader_ReadBaseAndSubOpcode 🌐 | likely | Reads two 16-bit header fields from the cursor structure into dest+4 and dest+8 (base and sub opcode), advancing the cursor and, on overrun, zeroing the field,… |
| `00a18db0` | ReadHeaderAndFields_7C_80_84_A0_0C_10_14 🌐 | likely | Reads the two U16 header fields, then three bounded dwords at destination +0x7c/+0x80/+0x84 and consumes two length-prefixed byte regions through 008e7290; rea… |
| `00a18e50` | ReadHeaderFields_0C_10_14_7C_80_84_A0 🌐 | likely | Reads the two U16 header fields, a bounded dword at destination +0x0c, a two-dword pair at +0x10/+0x14, then three bounded dwords at +0x7c/+0x80/+0x84 and two … |
| `00a18ef0` | ReadPacketHeaderThenFields_0C_10_14_7C_80_84_A0 🌐 | likely | Reads the two U16 header fields, a bounded dword at destination +0x0c, a two-dword pair at +0x10/+0x14, then three bounded dwords at +0x7c/+0x80/+0x84 and two … |
| `00a1ac10` | ParseNameChangeResponseFields 🌐 | likely | For non-null reader and input, builds a bounded reader over param_2 and param_3, calls 00a18ef0, then returns true only if the reader error byte is clear and p… |
| `00a1ada0` | WriteTwoU16PacketHeader | likely | Writes the low 16 bits of packet fields at +4 and +8 into a bounded byte writer, advancing its cursor after each field. |
| `00a1b730` | ReadU16U8AndBoundedRecordFields | likely | For non-null reader and input, builds a bounded reader over param_2 and param_3; 00a16720 reads a U16 and a byte, then 008e73b0 reads two dwords, consumes a le… |
| `00a1ba00` | SerializeU16Pair_DwordAndFixed8 | likely | Writes the two U16 header fields through 0090b290, copies at most four bytes from object+0x18, then writes the fixed eight-byte region at object+0x10 through 0… |
| `00a1ba70` | SerializeU16Pair_DwordAndFixed8_0C | likely | Writes the two U16 header fields through 00a1ae40, copies at most four bytes from object+0x0c, then writes the fixed eight-byte region at object+0x10 through 0… |
| `00a1cd80` | CheckNamePacket::Serialize 🌐 | likely | Writes the two U16 header fields, a bounded dword from +0x0c, the fixed eight-byte region at +0x10, three dwords at +0x7c/+0x80/+0x84, two length-prefixed buff… |
| `00a1ce30` | ChangeNameRequestPacket::Serialize 🌐 | likely | Writes the two U16 header fields, a bounded dword from +0x0c, the fixed eight-byte region at +0x10, three dwords at +0x7c/+0x80/+0x84, and two length-prefixed … |
| `00a1d050` | SendTunneledClientPayload_A 🌐 | likely | Initializes a writer, serializes through 00a1b900, constructs PacketTunneledClientPacket, adds a byte flag, dispatches the gateway packet, and cleans up. |
| `00a1d160` | SendTunneledClientPayload_B 🌐 | likely | Initializes a writer, serializes through 00a1b9a0, constructs PacketTunneledClientPacket, adds a byte flag, dispatches the gateway packet, and cleans up. |
| `00a1d270` | SendTunneledClientPayload_C 🌐 | likely | Initializes a writer, serializes through 00a1ba00, constructs PacketTunneledClientPacket, adds a byte flag, dispatches the gateway packet, and cleans up. |
| `00a1d380` | SendNameChangeStatusPacket 🌐 | likely | Initializes a writer, serializes through 00a1ba70, constructs PacketTunneledClientPacket, adds a byte flag, dispatches the gateway packet, and cleans up. |
| `00a1d490` | SendTunneledClientWorldPayload 🌐 | likely | Initializes a writer, serializes world fields, constructs PacketTunneledClientWorldPacket, dispatches it, and cleans up. |
| `00a1dc20` | SendInventoryPacketEquipByItemRecord 🌐 | likely | Initializes a writer, serializes through 00a1bfe0, constructs PacketTunneledClientPacket, adds a byte flag, dispatches the gateway packet, and cleans up. |
| `00a1dd30` | SendTunneledClientPayload_F 🌐 | likely | Initializes a writer, serializes through 00a1c080, constructs PacketTunneledClientPacket, adds a byte flag, dispatches the gateway packet, and cleans up. |
| `00a2fef0` | ReadInt16PairAndSixDwords | likely | Thiscall-style reader (ECX = output object, reader pointer as stack argument). Calls 00a2fc10 to read two signed 16-bit values into this+4/+8, then reads six 3… |
| `00a30060` | ReadInt16PairCountedBytesThreeFloats | likely | Reads two int16 into this+4/+8 via 00a2fc10. Then reads an i32 count: a non-negative count that fits the remaining bytes is copied through 00894b10 into the st… |
| `00a30170` | ReadInt16PairBoolAndFloat | likely | Reads two int16 into this+4/+8 via 00a2fc10, then one bool byte (non-zero stored as 1) into this+0xc, then a 32-bit value into this+0x10 whose float view sets … |
| `00a31430` | ReadInt16PairAndDword_A | likely | Cdecl reader: calls 00a2fc10 with the output object in ECX to read two signed 16-bit values into out+4 and out+8, then reads one 32-bit value into out+0xc from… |
| `00a31470` | ReadInt16PairAndDword_B | likely | Cdecl reader: calls 00a2fc10 with the output object in ECX to read two signed 16-bit values into out+4 and out+8, then reads one 32-bit value into out+0xc from… |
| `00a31aa0` | CheckedParseVia_00a2fc70 | likely | Checked parse wrapper. Returns 0 when param_1 or param_2 is null. Otherwise builds a packet reader over [param_2, param_2+param_3) (base +0, length +4, cursor … |
| `00a31b10` | CheckedParseVia_00a2fdb0 | likely | Checked parse wrapper with the same reader setup and return rule as the other checked parse wrappers: calls 00a2fdb0 with the reader and param_1 as output obje… |
| `00a31b80` | CheckedParseVia_00a2fe20 | likely | Checked parse wrapper with the same reader setup and return rule: calls 00a2fe20 with the reader and param_1 as output object. |
| `00a31bf0` | CheckedParseVia_00a2fef0 | likely | Checked parse wrapper with the same reader setup and return rule: calls 00a2fef0 with the reader and param_1 as output object. |
| `00a31c60` | CheckedParseVia_00a31430 | likely | Checked parse wrapper with the same reader setup and return rule: calls 00a31430 with the reader and param_1 as output object. |
| `00a31cd0` | CheckedParseVia_00a31470 🌐 | likely | Checked parse wrapper with the same reader setup and return rule: calls 00a31470 with the reader and param_1 as output object. |
| `00a31d40` | CheckedParseVia_00a30060 | likely | Checked parse wrapper with the same reader setup and return rule: calls 00a30060 with the reader and param_1 as output object. |
| `00a31db0` | CheckedParseVia_00a30170 | likely | Checked parse wrapper with the same reader setup and return rule: calls 00a30170 with the reader and param_1 as output object. |
| `00a31f30` | ReadPayloadWithCountedElements | likely | Reads a payload into the object in ECX from the reader: int16 pair through 00a2fc10 (into +4, +8), two dwords at +0x10 and +0x14, a counted element array throu… |
| `00a32e10` | BoundedWriteInt16Pair | likely | Thiscall with the record in ECX: writes the signed int16 at this+4, then the int16 at this+8, into the writer argument. Each write copies min(2, remaining) byt… |
| `00a32eb0` | BoundedWriteInt16PairAndDword | likely | Thiscall with the record in ECX: writes the int16 pair (through 00a32e10) and then the dword at this+0xc, bounded to min(4, remaining) bytes through 007cc400, … |
| `00a32fe0` | CheckedParse_Int16PairDwordTail 🌐 | likely | Checked parse function for the reader: returns 0 when param_1 or param_2 is null. Otherwise builds the reader, calls 00a2fc10 for an int16 pair, reads one 32-b… |
| `00a33090` | CheckedParse_Int16PairFloat4IdSlot | likely | Checked parse function for the reader: returns 0 when param_1 or param_2 is null. Otherwise builds the reader, calls 00a2fc10 for an int16 pair, reads four 32-… |
| `00a33120` | CheckedParse_Int16PairIdSlot | likely | Checked parse function for the reader: returns 0 when param_1 or param_2 is null. Otherwise builds the reader, calls 00a2fc10 for an int16 pair and 008dadd0 fo… |
| `00a33250` | BoundedWriteInt16PairAndTwoDwords | likely | Thiscall with the record in ECX: writes the int16 pair (through 00a32e10), then the dwords at this+0xc and this+0x10, each bounded to min(4, remaining) bytes t… |
| `00a342e0` | BoundedWriteRecordPayload | likely | Thiscall with the record in ECX: writes the int16 pair (through 00a32e10), then the two dwords (through 00a32f10), then the dword at this+0x20 bounded to min(4… |
| `00a345f0` | BuildAndSendTunneledRecord_B 🌐 | likely | Initialises a packet writer (008d4a40, 0x10000000 argument), writes the record through 00a33250 (int16 pair, dwords at +0xc and +0x10), constructs a ClientGate… |
| `00a34720` | BuildAndSendTunneledRecord_C 🌐 | likely | Initialises a packet writer (008d4a40, 0x10000000 argument), writes an int16 pair through 00a32e10 and four dwords from param_1+0x10 through 0091a640, construc… |
| `00a35000` | BuildAndSendTunneledStartAbility 🌐 | likely | Initialises a packet writer (008d4a40, 0x10000000 argument), writes the start-ability record through 00a342e0, constructs a ClientGateway::PacketTunneledClient… |
| `00a382b0` | BuildTunneledClientPacket 🌐 | likely | Builds a serialized payload on a 0x2000-byte stack writer: PacketWriterInit (008d4a40, argument 0x10000000), 00a38230 (copies up to four bytes from the source … |
| `00a3edd0` | DtorWrapper_a3edd0 | likely | SEH-framed cleanup wrapper (frame LAB_015b3888) that calls the announcement-packet row destructor 00a3eaf0 and returns; it touches no fields of its own. |
| `00a3ee20` | DtorWrapper_a3ee20 | likely | SEH-framed cleanup wrapper (frame LAB_015b38b8) that calls the member-panel row destructor 00a3ebd0 and returns; it touches no fields of its own. |
| `00a3ee70` | CtorWrapper_a3ee70 | likely | SEH-framed ECX-this wrapper that calls the announcement-packet row constructor 00a3ecc0 and returns the this pointer in EAX. |
| `00a3eec0` | CtorWrapper_a3eec0 | likely | SEH-framed ECX-this wrapper that calls the member-panel row constructor 00a3ed40 and returns the this pointer in EAX. |
| `00a3f350` | ReadRecordFieldsAndStrings | likely | Reads six dwords into output offsets +0x00..+0x14, then reads two signed length-prefixed byte blocks and passes each to FUN_00894b10 to assign string storage, … |
| `00a40150` | WriteU16AndByteFields | likely | Serialises two fields of the object in this into the byte writer param_2: truncates the 32-bit field at +4 to 16 bits and copies it with FUN_007cc400 (advancin… |
| `00a40ff0` | SendTunneledThreeBytePayload 🌐 | likely | Initializes a stack packet writer with capacity 0x10000000 and 0x2000 bytes of inline storage, serializes a three-byte input packet header through 00a40150, co… |
| `00a41100` | SendTunneledThreeBytePayload 🌐 | likely | Initializes a stack packet writer with capacity 0x10000000 and 0x2000 bytes of inline storage, serializes a three-byte input packet header through 00a40150, co… |
| `00a41d20` | WriteU16AndByteFieldsB | likely | Second copy of the two-field packet header writer: truncates the field at this+4 to 16 bits and the field at this+8 to one byte and copies both into the writer… |
| `00a41dc0` | Serialize4DwordPayload | likely | Serialises the packet body: writes the two-field header by calling 00a41d20, then copies the four dwords at this+0xc, +0x10, +0x14 and +0x18 into the writer wi… |
| `00a420c0` | SendTunneledSerializedPayload 🌐 | likely | Initializes a stack packet writer with capacity 0x10000000 and 0x2000 bytes of inline storage, serializes the caller's packet through 00a41dc0, constructs a st… |
| `00a442c0` | Read7U32Fields | likely | Reads seven consecutive 32-bit fields from the raw reader param_2 into param_1[0..6]; each read bounds-checks the cursor at reader+8 against the end at reader+… |
| `00a443d0` | Read10U32Fields | likely | Reads ten consecutive 32-bit fields from the raw reader param_2 into param_1[0..9] with the same bounded-read pattern (0 on short input, error byte at +0x10 se… |
| `00a44540` | Read3U32Fields | likely | Reads three 32-bit fields from the raw reader param_2 into param_1 in the destination order +4, +0, +8 (the array-form pseudocode writes param_1[1] first, then… |
| `00a445c0` | Read9U32Fields | likely | Reads nine consecutive 32-bit fields from the raw reader param_2 into param_1[0..8] with the bounded-read pattern (0 and error byte at +0x10 on short input). |
| `00a44de0` | ReadU16AndU32Fields | likely | Reads a 16-bit value into param_1+4 and then a 32-bit value into param_1+8 from the raw reader param_2, consuming 2 then 4 bytes and setting the error byte at … |
| `00a45680` | HashListMapInt32_ReadKeyValues 🌐 | likely | Empties the map at param_2 by removing head nodes through 007977c0 until map+8 is null, then reads a u32 count from the bounded reader param_1 (cursor +8, end … |
| `00a45740` | ReadRegionDefinitionMap 🌐 | likely | Removes every node of the region-definition map at param_2 (repeated 00a43630 on map+8), reads a u32 count from the reader param_1, then for each entry reads a… |
| `00a457e0` | ReadHubDefinitionMap 🌐 | likely | Removes every node of the hub-definition map at param_2 (repeated 00a436a0 on map+8), reads a u32 count from reader param_1, then for each entry reads a u32 ke… |
| `00a45880` | ReadHubQuestDefinitionMap 🌐 | likely | Removes every node of the hub-quest-definition map at param_2 (repeated 00a43710 on map+8), reads a u32 count from reader param_1, then for each entry reads a … |
| `00a45920` | ReadStickerDefinitionMap 🌐 | likely | Removes every node of the sticker-definition map at param_2 (repeated 00a43780 on map+8), reads a u32 count from reader param_1, then for each entry reads a u3… |
| `00a45bf0` | AdventurersJournal_ParseQuestUpdateBody 🌐 | likely | Reads a u16 and a u32 header through 00a44de0 into the object at arg1 (+4 and +8), then parses the int table with 00a45680 into the pointer at arg1+0xc from a … |
| `00a45cc0` | AdventurersJournal_ReadDefinitionTables 🌐 | likely | Reads the u16 and u32 header through 00a44de0 from the reader arg1, then reads the four definition tables into the block at arg2+0xc: region through 00a45740 a… |
| `00a45d10` | AdventurersJournal_ParseInfoBody 🌐 | likely | Builds a bounded reader over the buffer arg2 (length arg3), calls 00a45cc0 to read the info body into arg1, and returns 1 when the reader has no error and eith… |
| `00a45d80` | AdventurersJournal_HandlePacket 🌐 | likely | Parses the journal message: reads a u16 and a u32 header, takes the u32 as the sub-kind, and for kind 1 frees the previous 0x250 definition block at this+0x20,… |
| `00a5af10` | ReadFloatRecordFieldsAndString | likely | Reads four checked float dwords into +0..+0x0c, ten checked float dwords into +0x10..+0x34, a boolean at +0x38, three unchecked dwords at +0x3c..+0x44, six mor… |
| `00a5b510` | ReadSevenDwordsAndByteSpan | likely | Reads seven dwords into destination offsets 0 through +0x18, reads a signed byte count, rejects a negative or out-of-bounds span by marking the reader failed, … |
| `00a5b660` | ReadRecordVectorsScalarsAndThreeSpans | likely | Reads four 16-byte vector blocks at +0x20, +0x40, +0x60 and +0x80, checked and unchecked scalar fields, then three signed length-prefixed byte spans with bound… |
| `00a5bc10` | ReadFloat4DwordAndLengthPrefixedString | likely | Calls 008e2410 to read four checked float dwords into +0..+0x0c, reads a dword at +0x10, reads a signed length, rejects negative or out-of-bounds spans, assign… |
| `00a5c090` | ReadDwordLengthPrefixedStringAndDword | likely | Reads one dword into destination+0, reads and validates a signed length, assigns that byte span to the string member at destination+4 through 00894b10, advance… |
| `00a5cdc0` | ReadCountedTwoVectorAndDwordList | likely | Clears the destination list, reads a 32-bit count, then while the count is positive allocates a node, reads two 16-byte float blocks into it, reads one dword a… |
| `00a5d700` | ReadCountedRecordsIntoArray | likely | Reads a signed count and validates it against remaining input; resizes the destination array count to that value (growing storage when capacity is insufficient… |
| `00a5d780` | ReadCountedVectorAndDwordList | likely | Clears the destination list, reads a 32-bit count, then while the count is positive allocates and links a node, reads a 16-byte float block into it through 008… |
| `00a68dc0` | ReadElementArray0x4c | likely | Reads a count-prefixed array of 0x4c-byte records from bounded reader param_1, validates count against remaining bytes, resizes the destination array at param_… |
| `00a69630` | Wrapper_00a68100_U32 🌐 | likely | Builds and sends a ShipCombat admin command carrying one argument: constructs an AdminCommandPacketShipCombatCommand with FUN_00a57a30 (command key ShipCombat,… |
| `00a696b0` | Wrapper_00a68100_FourValues 🌐 | likely | ShipCombat admin command wrapper taking four arguments: constructs the AdminCommandPacketShipCombatCommand with FUN_00a57a30, stores param_2 at +0x10, param_3 … |
| `00a69750` | Wrapper_00a68100_U32AndString 🌐 | likely | ShipCombat admin command wrapper taking a number and a string: constructs AdminCommandPacketShipCombatCommand with FUN_00a57a30, stores param_2 at +0x10, copie… |
| `00a697e0` | Wrapper_00a68100_ThreeValues 🌐 | likely | ShipCombat admin command wrapper taking a number and an 8-byte handle: constructs AdminCommandPacketShipCombatCommand with FUN_00a57a30, stores param_2 at +0x1… |
| `00a82a90` | PackUint16Pair 🌐 | likely | Writes the low 16 bits of this+4 and then this+8 to the bounded byte writer through 007cc400, requesting at most two bytes for each value and advancing param_2… |
| `00a84300` | BasePacketDtorBody_00a84300 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; it is reached from an MSVC unwind funclet, i.e. it des… |
| `00a84a30` | PackUint16Uint8 🌐 | likely | Writes the low 16 bits of this+4 and then the low 8 bits of this+8 to the bounded byte writer through 007cc400, requesting at most two bytes and then one byte … |
| `00a861a0` | BasePacketDtorBody_00a861a0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; it is reached from an MSVC unwind funclet. |
| `00a86470` | BasePacketDtorBody_00a86470 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from two MSVC unwind funclets. |
| `00a86530` | BasePacketDtorBody_00a86530 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from one MSVC unwind funclet. |
| `00a865f0` | BasePacketDtorBody_00a865f0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from one MSVC unwind funclet. |
| `00a866b0` | BasePacketDtorBody_00a866b0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from 41 MSVC unwind funclets. |
| `00a86780` | BasePacketDtorBody_00a86780 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from 12 MSVC unwind funclets. |
| `00a86970` | BasePacketDtorBody_00a86970 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and one M… |
| `00a86a30` | BasePacketDtorBody_00a86a30 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and one M… |
| `00a86ae0` | BasePacketDtorBody_00a86ae0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and one M… |
| `00a86ba0` | BasePacketDtorBody_00a86ba0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from one MSVC unwind funclet. |
| `00a86cc0` | BasePacketDtorBody_00a86cc0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 (case 14,… |
| `00a86de0` | BasePacketDtorBody_00a86de0 | likely | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and one M… |
| `00a8b790` | PacketIStringListDtorBody | likely | Destructor body for a packet with an IString<char> member at +0xc and a List<int,-1> at +0x1c: sets the list vptr at +0x1c, calls 007798c0 (unlabelled), sets t… |
| `00a8f860` | WriteTwoByteAndWordFields | likely | Writes the low 16 bits of source+4, then the 32-bit field at source+8, clamping both writes to remaining output capacity. |
| `00a915d0` | WriteNestedPairAndWord | likely | Serializes a nested pair, then writes the 32-bit field at source+0x28 to the bounded output buffer. |
| `00a91630` | WritePairWordAndByte | likely | Serializes a two-byte pair, writes the 32-bit field at source+0x0c, then writes the byte at source+0x10 to the bounded output buffer. |
| `00a916d0` | WritePairAndWord | likely | Serializes a two-byte pair, then writes the 32-bit field at source+0x0c to the bounded output buffer. |
| `00a91730` | WritePairAndWordVariant | likely | Serializes a two-byte pair, then writes the 32-bit field at source+0x0c to the bounded output buffer. |
| `00a95c50` | WriteSharedPacketFields 🌐 | likely | Calls 008fb810 on this and this+0x90, calls 00a0e8f0 twice, then copies the dword at +0x120 to the bounded output, clamped to four remaining bytes. |
| `00a9d4f0` | BodyRead_i16x2 | likely | Reads two sign-extended 16-bit values from the stream object (cursor at +8, end at +0xc) into destination +4 and +8, advancing the cursor by two bytes each; on… |
| `00a9d590` | BodyRead_i16x2_i32x2_str2 | likely | Reads a header pair via FUN_00a9d4f0, then a 32-bit pair into destination +0x10/+0x14, then delegates two string fields to FUN_008dab40 at +0x18 and +0x28, zer… |
| `00a9d5f0` | BodyRead_i16x2_i32x2_i32 | likely | Reads the two header words through FUN_00a9d4f0, then a 32-bit pair into destination +0x10/+0x14 and one 32-bit value into +0x18, with the usual zero-and-error… |
| `00a9d6a0` | BodyRead_i16x2_i32x2_i32x2_i32_i32 | likely | Reads the header pair via FUN_00a9d4f0, two 32-bit pairs into +0x10/+0x14 and +0x18/+0x1c, then two single 32-bit values into +0x20 and +0x24, with bounds chec… |
| `00a9d960` | ReadLengthPrefixedStringInto8 🌐 | likely | Reads an int32 count from the stream; when 0 <= count and count bytes fit before the end, it assigns exactly count bytes from the cursor into the string member… |
| `00a9d9c0` | BodyRead_i32x19_bool2 | likely | Copies nineteen 32-bit values from the stream into destination +4 through +0x4c (strictly sequential dword reads with per-field bounds checks) and then two sin… |
| `00a9dca0` | BodyRead_i32x6_skip_i32 🌐 | likely | Reads six consecutive 32-bit values into destination +0, +4, +8, +0xc, +0x10 and +0x14, skips one 32-bit slot, then reads a last 32-bit value into +0x1c; every… |
| `00a9ee60` | BodyRead_HeaderPair_i32_at0c | likely | Reads the two 16-bit header words through FUN_00a9d4f0 and then one 32-bit value into destination +0xc (past the header pair), with the usual error handling on… |
| `00a9eec0` | BodyRead_HeaderPair_bool_at0c | likely | Reads the header pair via FUN_00a9d4f0 and then one boolean byte (nonzero compare) into destination +0xc. |
| `00a9ffb0` | ParseMatchmakingBody_Case12 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ee60 (header pair plus one 32-bit value … |
| `00aa0020` | ParseMatchmakingBody_SubOp14 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9d590 (header pair, 32-bit pair and two s… |
| `00aa0090` | ParseMatchmakingBody_Case18 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9d5f0 (header pair, 32-bit pair and one 3… |
| `00aa0100` | ParseMatchmakingBody_Case15 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9eec0 (header pair plus one boolean at +0… |
| `00aa0170` | ParseMatchmakingBody_Case9 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9d6a0 (header pair, two 32-bit pairs and … |
| `00aa0460` | ReadIntrusiveListOfQueueEntries | likely | Drains the list reached through param_2+4 with FUN_00a80850, reads a 32-bit count, then per element constructs a node with FUN_00a9aa60 and fills it through FU… |
| `00aa0550` | ReadIntrusiveListOfHashNodes | likely | Drains the hash list at param_2+8 with FUN_00a807a0, reads a 32-bit count, then per element reads one 32-bit key, appends a keyed node through FUN_00a9ddb0 (wh… |
| `00aa1050` | BodyRead_i16x2_i32x2_list | likely | Reads the header pair through FUN_00a9d4f0, a 32-bit pair into destination +0x10/+0x14, then builds an entry list at +0x18 through FUN_00aa0460 (count plus nin… |
| `00aa16f0` | BodyRead_MatchmakingRequestFields | likely | Reads a 32-bit pair into +8/+0xc, single 32-bit values into +0x10 and +0x14, another pair into +0x18/+0x1c, single values into +0x20, +0x50 and +0x54, an entry… |
| `00aa1c20` | ParseMatchmakingBody_SubOp2 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00aa1050 (header pair, 32-bit pair and a cou… |
| `00aa2460` | ParseMatchmakingBody_SubOp4 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), reads the header pair with FUN_00a9d4f0 and then the r… |
| `00aa24e0` | ParseMatchmakingBody_SubOp5 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), reads the header pair with FUN_00a9d4f0 and then the r… |
| `00ab0520` | ReleaseTwoEmbeddedMembers | likely | Runs under an exception frame and invokes the owned-member cleanup routine twice before restoring the previous exception list. |
| `00ab0680` | DestroyTwoFixedStringsAndBasePacket | likely | Calls the fixed-string destructor twice, then installs BaseHousingPacket and BasePacket vtables during cleanup. |
| `00ab0710` | DestroyTwoFixedStringsAndBasePacketVariant | likely | Calls the fixed-string destructor twice, then installs BaseHousingPacket and BasePacket vtables during cleanup. |
| `00ab0840` | DestroyFixtureAssetPacketMembers | likely | Releases the embedded character-guid array and fixed strings, then installs BaseHousingPacket and BasePacket vtables during cleanup. |
| `00ab0a60` | FixtureItemListScalarDeletingDtor | likely | Calls the fixture-item-list GUID-array cleanup helper, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00aba870` | WriteU16ThenU8 | likely | Reads a u16 from this+4 and a u8 from this+8, then writes those fields in order through FUN_007cc400. Raw x86 shows the writer remaining-space and cursor field… |
| `00acc310` | PacketVtableResetDtor_A | likely | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable and returns; its only caller is an exception-unwind funclet, so the… |
| `00acc460` | PacketVtableResetDtor_B | likely | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable and returns; its only caller is an exception-unwind funclet, so the… |
| `00acd100` | ParseMessageBodyTunneled 🌐 | likely | Builds a bounded reader over [param_1, param_1+param_2), runs the u16+bool+blob reader 00acced0 on it and returns the reader's error byte (0 = clean). |
| `00ad61f0` | BuildTunneledPortraitPacket 🌐 | likely | Builds a ClientGateway tunneled-client packet around serialized portrait data, copies two header values and one flag into packet fields, then destroys the temp… |
| `00adae70` | PacketStream_ReadSignedShortAndByte | likely | Bounds-checks and reads a signed 16-bit value into dest+4, then a signed byte into dest+8; advances the stream cursor after each read. A short read zeroes that… |
| `00adaed0` | PacketStream_ReadStateFields | likely | Reads a 2-byte id and a 1-byte flag with 00adae70, then reads five little-endian dwords with individual bounds checks into dest+0xc, +0x10, +0x14, +0x18 and +0… |
| `00adb070` | PacketStream_ReadConsecutiveFields | likely | Reads the 2-byte id and 1-byte flag with 00adae70 and then six bounds-checked dwords into dest+0xc through dest+0x20, setting the stream error flag and clampin… |
| `00adb160` | PacketStream_ReadAdminFields | likely | Reads the 2-byte id and 1-byte flag with 00adae70, then two bounds-checked dwords into +0xc and +0x10, then an 8-byte little-endian value into +0x18/+0x1c (all… |
| `00adb1f0` | PacketStream_ReadAdminFieldsAndBool | likely | Runs 00adb160 and then reads one more bounds-checked byte into dest+0x20, storing it as a non-zero bool (0 with the error flag when the stream is exhausted). |
| `00adb5d0` | PacketStream_ReadAdminFieldsAndBool2 | likely | Variant of 00adb1f0 with these/argument roles swapped: runs the admin header reader 00adb160 on the stream object and then reads one bounds-checked byte into d… |
| `00b19f00` | ReadTwoShorts | likely | Reader helper: takes two 16-bit values from the bounded reader (cursor at +8, end at +0xc) into output+4 and output+8, writing 0 and setting the truncation fla… |
| `00b1a1b0` | TwoNameRecords_Release | likely | Cleanup body that releases two name-record sub-objects by calling 0070a790 twice under an exception frame; callers are the guild invite notification destructor… |
| `00b1a810` | ReadTwoShorts | likely | Reader helper with nineteen callers: takes two 16-bit values from the bounded reader into output+4 and output+8, writing 0 and setting the truncation flag at r… |
| `00b1b000` | TwoNameRecords_Construct | likely | Constructor helper that builds two name-record sub-objects by calling 0070bac0 twice inside an exception frame; the inverse cleanup is 00b1a1b0, which releases… |
| `00b40a60` | TryConsumeWholePayload | likely | Builds a reader context {ptr=param_2, len=param_3, cursor=param_2, end=param_2+param_3, flag=0, err=0} and runs FUN_00b3d5f0 over it; returns 1 only when no er… |
| `00b484f0` | ReadTwoU32AndOptionalPayload | likely | Initializes a bounded reader over the supplied byte range, reads two 32-bit fields, and parses an optional destination at this+0x20 with 00b45340 when that poi… |
| `00b49e00` | ReadU16ThenU32 | likely | Reads a 16-bit value from the stream cursor at [stream+8] into (dest+4) and then a 32-bit value into (dest+8), advancing the cursor; if the range is short it z… |
| `00b49f90` | ReadU16U32U32 | likely | Reads the shared 16-bit + 32-bit field pair with FUN_00b49e00 and then a further 32-bit value into (param_2+0xc), with the same short-read behaviour (zero the … |
| `00b4a280` | ReadU16U32ThenFourU32 | likely | Initializes a bounded reader, consumes the shared 16-bit and 32-bit fields with 00b49e00, then calls 008e8640, which reads four 32-bit words; returns 1 on a cl… |
| `00b4a300` | WriteU16ThenU32 | likely | Appends two fields to a bounded writer buffer: the 16-bit value at [this+4] and the 32-bit value at [this+8], each copied with memcpy (FUN_007cc400) after clam… |
| `00b4a570` | WriteU16U32ThenU32 | likely | Writes the packet header with FUN_00b4a300 (16-bit value from +4 then 32-bit value from +8) and then appends the 32-bit value at [this+0xc] to the same bounded… |
| `00b577d0` | ReadShortByteThenDword | likely | Reads a short and byte through 00b55280, then reads one bounded dword into param_2+0xc; on insufficient input it zeros the field and marks the reader failed. |
| `00b578d0` | ReadShortBytePairAndFlag | likely | Reads a short/byte prefix and an eight-byte pair, then reads a bounded boolean into param_2+0x20; failed input stores false and marks the reader failed. |
| `00b57c70` | CheckedReaderAdapter_00b57c70 | likely | For non-null reader and output, calls the 3-dword packet-field reader and returns success only when no failure flag is set and the input is consumed or trailin… |
| `00b57ce0` | CheckedReaderAdapter_00b57ce0 | likely | For non-null inputs, calls the short/byte plus pair/seven-dword/flag reader, then validates the failure flag and full-consumption policy. |
| `00b57d50` | CheckedReaderAdapter_00b57d50 | likely | For non-null inputs, calls the short/byte/dword reader and returns success only if its failure flag is clear and the buffer meets the caller's trailing-data po… |
| `00b57dc0` | CheckedReaderAdapter_00b57dc0 | likely | For non-null inputs, calls the short/byte plus three-dword reader and applies the common reader-failure and input-consumption checks. |
| `00b57e30` | CheckedReaderAdapter_00b57e30 | likely | For non-null inputs, reads a short/byte prefix and fixed float record through 00b555a0, then checks reader failure and trailing-data policy. |
| `00b57ea0` | CheckedReaderAdapter_00b57ea0 | likely | For non-null inputs, invokes the short/byte/dword/pair reader and checks the reader failure flag plus the configured full-consumption policy. |
| `00b57f10` | CheckedReaderAdapter_00b57f10 | likely | For non-null inputs, invokes the short/byte/pair/length reader and checks the common parse-failure flag and trailing-data policy. |
| `00b57f80` | CheckedReaderAdapter_00b57f80 | likely | For non-null inputs, invokes the short/byte/three-dword/flag reader then applies common parse-failure and buffer-consumption checks. |
| `00b57ff0` | CheckedReaderAdapter_00b57ff0 | likely | For non-null inputs, invokes the nine-dword/flag reader and returns success only when parsing succeeds and the trailing-data policy is satisfied. |
| `00b58060` | CheckedReaderAdapter_00b58060 | likely | For non-null inputs, invokes the short/byte plus three-dword reader and applies the shared failure and input-consumption checks. |
| `00b580d0` | CheckedReaderAdapter_00b580d0 | likely | For non-null inputs, invokes the short/byte plus five-dword reader and checks its parse status and the requested trailing-data policy. |
| `00b58140` | CheckedReaderAdapter_00b58140 | likely | For non-null inputs, invokes the short/byte/flag/three-dword reader and applies the shared parse-failure and input-consumption checks. |
| `00b581b0` | CheckedReaderAdapter_00b581b0 | likely | For non-null inputs, invokes the short/byte/flag/two-dword reader and applies the shared parse-failure and trailing-data checks. |
| `00b58220` | CheckedReaderAdapter_00b58220 | likely | For non-null inputs, invokes the short/byte/six-dword reader and applies the common failure and input-consumption policy. |
| `00b58290` | CheckedReaderAdapter_00b58290 | likely | For non-null inputs, invokes the short/byte/pair/two-dword reader and applies the common parse-failure and trailing-data policy. |
| `00b58300` | CheckedReaderAdapter_00b58300 | likely | For non-null inputs, invokes the short/byte/pair/flag reader and applies the common parse-failure and input-consumption checks. |
| `00b58370` | CheckedReaderAdapter_00b58370 | likely | For non-null inputs, reads a short/byte prefix and dword, delegates the remaining object body reader, then checks failure and trailing-data policy. |
| `00b583e0` | CheckedReaderAdapter_00b583e0 | likely | For non-null inputs, invokes the short/byte reader and applies the shared parse-failure and input-consumption checks. |
| `00b58450` | CheckedReaderAdapter_00b58450 | likely | For non-null inputs, invokes the short/byte/four-dword reader and applies the common failure and input-consumption checks. |
| `00b58780` | ReadTwoFlagsAndDwordAndOptionalObject | likely | Reads two bounded boolean bytes and one bounded dword into fields at param_1+0xc, +0xd and +0x10; if the first flag is set and param_1+0x14 is nonzero, reads t… |
| `00b595d0` | SerializePacketPrefixFields | likely | Writes two bytes from the dword at param_1+4 followed by one byte from param_1+8 into the bounded output buffer and advances its cursor by the bytes accepted. |
| `00b5a130` | ReadCountedFiveDwordRecordsChecked | likely | Reads the leading short and byte, parses a counted list of five-dword records, and returns success only when parsing and trailing-data checks pass. |
| `00b5a1b0` | ReadFlagsDwordAndOptionalNestedObjectChecked | likely | Invokes the field reader for two flags, one dword and an optional nested object, then returns success only when parsing and trailing-data checks pass. |
| `00b5a240` | SerializeOneDwordField | likely | Writes the shared packet prefix, then writes the dword stored at param_1+0xc into the bounded output buffer and advances its cursor. |
| `00b5a2a0` | SerializeThreeDwordFields | likely | Writes the shared packet prefix, then writes dwords at param_1+0xc, +0x10 and +0x14 into the bounded output buffer. |
| `00b5a380` | SerializeTwoDwordFieldsA | likely | Writes the shared packet prefix followed by dwords at param_1+0xc and +0x10 into the bounded output buffer. |
| `00b5a420` | SerializeTwoDwordFieldsB | likely | Writes the shared packet prefix followed by dwords at param_1+0xc and +0x10 into the bounded output buffer. |
| `00b5a4c0` | SerializeDwordAndVirtualBody | likely | Writes the common packet prefix and dword at param_1+0xc, then calls the virtual serializer on the object at param_1+0x10. |
| `00b5a590` | SerializeOneDwordFieldA | likely | Writes the shared packet prefix, then writes the dword at param_1+0xc into the bounded output buffer. |
| `00b5a5f0` | SerializeOneDwordFieldB | likely | Writes the shared packet prefix, then writes the dword at param_1+0xc into the bounded output buffer. |
| `00b5a650` | SerializeDwordAndByteFields | likely | Writes the common prefix, the dword at param_1+0xc and the byte at param_1+0x10 into the bounded output buffer. |
| `00b5a6f0` | SerializeTwoDwordFieldsC | likely | Writes the shared packet prefix followed by dwords at param_1+0xc and +0x10 into the bounded output buffer. |
| `00b5a790` | SerializeByteAndTwoDwordFields | likely | Writes the shared prefix, then a byte at param_1+0xc and dwords at +0x14 and +0x10 into the bounded output buffer. |
| `00b5a870` | SerializeOneByteField | likely | Writes the common packet prefix, then writes the byte at param_1+0xc into the bounded output buffer. |
| `00b5a8d0` | SerializeFourDwordFields | likely | Writes the shared packet prefix followed by four dwords at param_1+0xc, +0x10, +0x14 and +0x18. |
| `00b5b0c0` | SerializeDwordWithNestedData | likely | Writes the common packet prefix, delegates nested serialization to 00a1aee0, then writes the dword at param_1+0x1c. |
| `00b5ba90` | ReadShortByteAndTwoWords 🌐 | likely | Reads the short and byte fields through FUN_00b55280, reads another value through FUN_00b5aea0, then consumes one dword and a pair of dwords with bounds checks… |
| `00b5cb20` | ValidateParsedFieldRange 🌐 | likely | Parses a temporary record with FUN_00b5ba90, then returns true only if the parse flag stays clear and either param_4 is nonzero or param_3 is zero/negative by … |
| `00b70e40` | ReadTwoU16OrSetReaderError | likely | Reads two 16-bit words from the cursor at reader+8 into destination fields +4 and +8, advancing by two bytes after each read; a short read zeroes the destinati… |
| `00b950a0` | Packet_WriteHeaderWords | likely | Writes the two 16-bit header words of a packet-shaped object into a writer: copies the low 16 bits of this+4 and of this+8 through FUN_007cc400 at the writer c… |
| `00b96d40` | Packet_SerializeBaseFields | likely | Serializes a packet object into a writer: writes the two header words from this+4/+8 through 00b950a0, appends the dword at this+0xc, writes the field at this+… |
| `00b97a80` | PacketBase_DtorEmissionA | likely | Destructor body of a packet class whose only base is BasePacket: stores BasePacket::vftable at this+0 and returns (no fields, no deallocation). |
| `00b97b30` | PacketBase_DtorEmissionB | likely | Destructor body of a packet class whose only base is BasePacket: stores BasePacket::vftable at this+0 and returns. |
| `00b97bf0` | PacketBase_DtorEmissionC | likely | Destructor body of a packet class whose only base is BasePacket: stores BasePacket::vftable at this+0 and returns. |
| `00b99000` | Packet_SendZoneTeleportRequest 🌐 | likely | Builds and sends the op-90 teleport request: allocates an 8KB buffer (FUN_008d4a40), serializes the packet object with 00b98ad0 (header word plus payload dword… |
| `00bb0e90` | PacketWriter_AppendU16U8 | likely | Thiscall (ECX = source struct, stack argument = packet writer): appends the 2-byte value at source+4 and the 1-byte value at source+8 to the writer's buffer at… |
| `00bb0fd0` | PacketWriter_AppendStruct_U16U8U32U32 | likely | Thiscall (ECX = source struct, stack argument = packet writer): writes the u16 at +4 and the byte at +8 through 00bb0e90, then the dword at +0xc and the dword … |
| `00bb1070` | PacketWriter_AppendStruct_U16U8U32U32_B | likely | Same serializer as 00bb0fd0 (identical instructions apart from the relative call to 00bb0e90): writes the u16 at +4 and byte at +8, then the dwords at +0xc and… |
| `00bb1110` | PacketWriter_AppendStruct_U16U8U32U32_C | likely | Same serializer as 00bb0fd0 (identical instructions apart from the relative call to 00bb0e90): writes the u16 at +4 and byte at +8, then the dwords at +0xc and… |
| `00bda240` | GatewaySend_PacketWithDword 🌐 | likely | Gateway send helper: initialises an 8KB packet buffer with limit 0x10000000 (FUN_008d4a40), serialises the packet record passed in ECX through 00bd98c0 (u16 pa… |
| `00bdb450` | RecordWriter_HeaderAndThreeDwords 🌐 | likely | Serialises a packet record with three dword fields into a packet byte buffer: calls 00a1ada0 for the two u16 header fields (this+4, this+8), then copies the dw… |
| `00bdb530` | RecordWriter_HeaderAndTwoDwords 🌐 | likely | Serialises a packet record with two dword fields into a packet byte buffer: calls 00a1ada0 for the u16 header pair (this+4, this+8), then copies this+0xc and t… |
| `00bdb5d0` | RecordWriter_HeaderAndThreeDwords_B 🌐 | likely | Serialises a packet record with three dword fields (this+0xc, this+0x10, this+0x14) after the u16 header pair written by 00a1ada0, clamping each copy to min(re… |
| `00bdb6b0` | RecordWriter_HeaderAndOneDword_B 🌐 | likely | Serialises a packet record with a single dword payload: calls 00a1ada0 to append the u16 header pair (this+4, this+8) and then copies the dword at this+0xc wit… |
| `00bdb790` | GatewaySend_PacketWithThreeDwords 🌐 | likely | Gateway send helper: initialises the 8KB buffer (FUN_008d4a40 with limit 0x10000000), serialises the record through 00bdb450 (u16 header pair plus three dwords… |
| `00bdb8a0` | GatewaySend_PacketWithTwoDwords 🌐 | likely | Gateway send helper: same template as 00bdb790 but with the two-dword record writer 00bdb530, so the payload after the u16 header pair is exactly two dwords; s… |
| `00bdb9b0` | GatewaySend_PacketWithThreeDwords_B 🌐 | likely | Gateway send helper: same template as 00bdb790 but paired with the record writer 00bdb5d0, so three dwords follow the u16 header pair; sends through FUN_009256… |
| `00bdbac0` | GatewaySend_PacketWithOneDword 🌐 | likely | Gateway send helper: same template as 00bdb790 but paired with the one-dword record writer 00bdb6b0; after FUN_00925640 sends the tunneled packet the local vta… |
| `00bddf50` | Reader_ReadU16U32AndByte | likely | Calls 00bdddd0 to read the u16 and u32 into the record at param_2+4 and +8, then reads one byte into param_2+0xc (a boolean), setting the reader error byte and… |
| `00beefd0` | ReadTwoS16 | likely | Reads two signed 16-bit values from a bounded reader into offsets 4 and 8, advances the cursor by two bytes per value, and marks failure with zero defaults on … |
| `00bef0b0` | ReadTwoS16 | likely | Reads two signed 16-bit values into destination offsets 4 and 8 from the bounded reader at offsets 8 and 12, marking failure and clearing values if truncated. |
| `00bef110` | ReadLengthPrefixedBytesAfterS16Pair | likely | Reads two signed shorts, then a signed length; validates the remaining range, copies that many bytes through FUN_00894b10, and updates cursor and failure state. |
| `00bef170` | ReadLengthPrefixedBytesAfterS16Pair | likely | Reads two signed shorts, then a signed length; validates the remaining range, copies that many bytes through FUN_00894b10, and updates cursor and failure state. |
| `00bef400` | ValidateReaderRange | likely | Runs the short-pair reader against local cursor state and returns true only when no read error occurred and the requested range is valid unless partial reads a… |
| `00c14270` | BasePacketDtorNoMembers_00c14270 | likely | Destructor body that stores only BasePacket::vftable at *this and returns (a packet class whose own vtable slot was already reset by the caller or that has no … |
| `00c14320` | BasePacketDtorNoMembers_00c14320 | likely | Destructor body that stores only BasePacket::vftable at *this and returns. |
| `00c20930` | SerializeU16AndEightByteField | likely | Writes two bytes from source+4 to the bounded writer, then passes source+8 to 008fb810, which copies eight bytes; advances the writer cursor after each write. |
| `00c20990` | SerializeU16AndU32 | likely | Writes two bytes from source+4, then copies four bytes from the value at source+8 to the bounded writer and advances its cursor after each write. |
| `00c20ad0` | SerializeU16PairAndU32 | likely | Writes a pair of 16-bit fields through 009e6860, then copies the field at source+0xc as four bytes to the bounded writer. |
| `00c20b30` | SerializeU16PairEightByteFieldAndTwoU32 | likely | Writes the two-U16 header via 00a1ada0, copies eight bytes from source+0x10 via 008fb810, then writes four-byte fields from source+0x18 and source+0x1c. |
| `00c214d0` | SerializeU16PairAndTwoU32 | likely | Serializes a common prefix through 00c20be0, then writes four-byte fields from source offsets +0xc and +0x10. |
| `00c23d00` | RequestEnterSocialZone | likely | Reads one value, constructs PacketEnterSocialZoneRequest, and sends its tunneled body when the command service is enabled. |
| `00c49680` | BasePacketVtableReset_00c49680 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49740` | BasePacketVtableReset_00c49740 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49800` | BasePacketVtableReset_00c49800 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49850` | BasePacketVtableReset_00c49850 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49910` | BasePacketVtableReset_00c49910 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c499d0` | BasePacketVtableReset_00c499d0 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49a90` | BasePacketVtableReset_00c49a90 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49b50` | BasePacketVtableReset_00c49b50 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49c10` | BasePacketVtableReset_00c49c10 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49cd0` | BasePacketVtableReset_00c49cd0 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c49d90` | BasePacketVtableReset_00c49d90 | likely | Rewrites *this to BasePacket::vftable and returns; it destroys no other member and does not free the object. Its sole recorded caller is an EH unwind funclet. |
| `00c74e40` | ReadBoolFieldFromBuffer | likely | Reads one byte from the bounded buffer at param_1 (cursor +8, end +0xc, overrun flag +0x10 set and cursor clamped on overrun) into byte field +0x10 of param_2 … |
| `00c7acd0` | ReadQuestPacketPrefix | likely | Reads a 16-bit field into output+4 and a 32-bit field into output+8; on either short read it zeroes that field, sets the reader error byte, and clamps the curs… |
| `00c7bc50` | SelectQuestPacket::Write | likely | Serializes the base UI packet fields, then appends the 32-bit quest id at object+0xc to the bounded output writer. |
| `00c7bcb0` | SelectedQuestLockedPacket::Write | likely | Serializes the base UI packet fields, then appends the single boolean byte at object+0xc to the bounded output writer. |
| `00c7e890` | FriendPacketStringDestruct | likely | Restores the embedded IString vtable, decrements its shared-buffer count atomically and frees the last reference through virtual slot +8, then restores BasePac… |
| `00c7e9a0` | FriendPacketStringDestruct | likely | Restores embedded IString vtable, decrements its shared-buffer count atomically and frees on last reference through virtual slot +8, then restores BasePacket v… |
| `00c7eac0` | FriendPacketStringDestruct | likely | Restores the embedded IString vtable, atomically decrements its buffer count, frees at last reference via virtual slot +8, then restores BasePacket vtable. |
| `00c7ef80` | ReadSocialPacketPrefix | likely | Reads a 16-bit value and one signed byte into output+4/+8; a missing field is zeroed, sets reader error and clamps cursor to end. |
| `00c7efe0` | ReadSocialRecordWithLength | likely | Reads the shared short/byte prefix, a signed length and bounded byte span, then two 32-bit fields; invalid length or short fields mark reader error and clamp t… |
| `00c7f070` | ReadSocialRecordWithFlag | likely | Reads the shared prefix, a signed bounded length-delimited byte span, two 32-bit values, then one boolean; short or invalid input sets reader error. |
| `00c7f1d0` | ReadSocialPacketPrefix | likely | Reads a 16-bit field and a signed byte into output+4/+8, zeroing missing values and setting reader error while clamping the cursor. |
| `00c7f590` | ReadTwoDwordsAndSkip | likely | Reads two 32-bit values into output+0/+4, then reads a signed length and advances over that many bytes only when the bounded span is valid; invalid input flags… |
| `00c89a20` | SendFirstTimeEventBytePacket 🌐 | likely | Serializes the byte-field first-time-event message, wraps the bytes in a tunneled client packet, and submits it to the gateway. |
| `00c89b30` | SendFirstTimeEventDwordPacket 🌐 | likely | Serializes the dword-field first-time-event message, wraps the bytes in a tunneled client packet, and submits it to the gateway. |
| `00c89c40` | SendFirstTimeEventDwordPacket 🌐 | likely | Serializes the dword-field first-time-event message, wraps the bytes in a tunneled client packet, and submits it to the gateway. |
| `00c8b000` | ReadShortAndByte | likely | Reads a signed 16-bit value then one byte from a bounded serialization cursor, advancing it and setting its failure flag on truncation. |
| `00c8be30` | ReadFieldsWithinBounds | likely | Reads an initial short and byte fields, calls the recipe row reader, then checks cursor bounds and failure state. |
| `00c8d0f0` | ReadRecipePacketPayloadWithinBounds | likely | Reads common short and byte fields, parses recipe components, then returns success if cursor bounds and parser status permit. |
| `00cff950` | SerializePacketFields_00cff950 🌐 | likely | Writes the 16-bit field from object offset 4 and the low byte from offset 8 into the supplied packet writer, advancing its write cursor after each field. |
| `00cff9f0` | SerializeObjectiveCompletionBody 🌐 | likely | Calls the preceding serializer for the 16-bit and byte fields, then appends the dword at object offset 0x0c to the packet writer. |
| `010d9800` | TryDispatchKeyedFrame | likely | Thiscall with a buffer pointer and a length. Only runs when the byte at this+0x14 is nonzero. It reads a 16-bit key in one of three encodings chosen by the mod… |

### `net.packet.ability`

**AbilityPacketAbilityDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d320` | AbilityPacketAbilityDefinition::AbilityPacketAbilityDefinit… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xd, stores AbilityPacketAbilityDefinition::vftable and constructs the embedded definition obj… |
| `00a2e2c0` | AbilityPacketAbilityDefinition::ScalarDeletingDtor | likely | Runs the destructor 00a2d380 (which destroys the embedded definition member and restores the base vtables) and then releases the object with _free when bit 0 o… |

**AbilityPacketCastInterrupt** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2e3c0` | AbilityPacketCastInterrupt::AbilityPacketCastInterrupt 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0x12 and then stores AbilityPacketCastInterrupt::vftable at +0. |
| `00a2e420` | AbilityPacketCastInterrupt::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketClientMoveAndCast** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2cda0` | AbilityPacketClientMoveAndCast::AbilityPacketClientMoveAndC… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 6, stores AbilityPacketClientMoveAndCast::vftable, and zeroes +0x10..+0x20 with +0x24 set to 0… |
| `00a2deb0` | AbilityPacketClientMoveAndCast::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketClientRequestStartAbility** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2e0e0` | AbilityPacketClientRequestStartAbility::AbilityPacketClient… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 10, stores AbilityPacketClientRequestStartAbility::vftable, sets +0xc to 0 and +0x10 to 0xffff… |
| `00a2e170` | AbilityPacketClientRequestStartAbility::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketDetonateProjectile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2ee30` | AbilityPacketDetonateProjectile::AbilityPacketDetonateProje… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xe, stores AbilityPacketDetonateProjectile::vftable, copies the default id pair DAT_01be6870/… |
| `00a2eeb0` | AbilityPacketDetonateProjectile::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketExecuteClientLua** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2f5e0` | AbilityPacketExecuteClientLua::AbilityPacketExecuteClientLua 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0x11, stores AbilityPacketExecuteClientLua::vftable, then initialises an owned SoeUtil::IStrin… |

**AbilityPacketFailed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d0c0` | AbilityPacketFailed::AbilityPacketFailed 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 1, stores AbilityPacketFailed::vftable and zeroes the dword at +0xc. |
| `00a2e070` | AbilityPacketFailed::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketLaunchAndLand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2f450` | AbilityPacketLaunchAndLand::AbilityPacketLaunchAndLand 🌐 | likely | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 4, stores AbilityPacketLaunchAndLand::vftable, writes the default id pair DAT_01be6870/DAT_01b… |

**AbilityPacketMeleeRefresh** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d1c0` | AbilityPacketMeleeRefresh::AbilityPacketMeleeRefresh 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xb, stores AbilityPacketMeleeRefresh::vftable and zeroes the dword at +0xc. |
| `00a2e1e0` | AbilityPacketMeleeRefresh::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketPulseLocationTargeting** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d430` | AbilityPacketPulseLocationTargeting::AbilityPacketPulseLoca… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xf, stores AbilityPacketPulseLocationTargeting::vftable, clears the byte at +0xc and copies t… |
| `00a2e2e0` | AbilityPacketPulseLocationTargeting::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketPurchaseAbility** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2ce70` | AbilityPacketPurchaseAbility::AbilityPacketPurchaseAbility 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 7, stores AbilityPacketPurchaseAbility::vftable, and sets +0xc to 0xffffffff and +0x10 to 0. |
| `00a2df20` | AbilityPacketPurchaseAbility::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketReceivePulseLocation** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2e350` | AbilityPacketReceivePulseLocation::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketRequestAbilityDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d270` | AbilityPacketRequestAbilityDefinition::AbilityPacketRequest… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xc, stores AbilityPacketRequestAbilityDefinition::vftable and sets the dword at +0xc to 0xfff… |
| `00a2e250` | AbilityPacketRequestAbilityDefinition::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketSetDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2ccf0` | AbilityPacketSetDefinition::AbilityPacketSetDefinition 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 5, stores AbilityPacketSetDefinition::vftable, zeroes +0xc and allocates an 8-byte child objec… |
| `00a2de10` | AbilityPacketSetDefinition::~AbilityPacketSetDefinition | confirmed | Stores AbilityPacketSetDefinition::vftable, destroys and frees the 8-byte child object at +0x10 (FUN_01046050 then _free), then resets the vtable to BaseAbilit… |
| `00a2de90` | AbilityPacketSetDefinition::ScalarDeletingDtor | likely | Runs the destructor 00a2de10 and then releases the object with _free when bit 0 of the flag argument is set. |

**AbilityPacketStartCasting** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2ed30` | AbilityPacketStartCasting::AbilityPacketStartCasting 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 3, stores AbilityPacketStartCasting::vftable, initialises +0x10..+0x1c with two copies of the … |
| `00a2edc0` | AbilityPacketStartCasting::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketStopAura** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2cfe0` | AbilityPacketStopAura::AbilityPacketStopAura 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 9, stores AbilityPacketStopAura::vftable, and copies the two dwords pointed to by the second a… |
| `00a2e000` | AbilityPacketStopAura::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**AbilityPacketUpdateAbilityExperience** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2cf20` | AbilityPacketUpdateAbilityExperience::AbilityPacketUpdateAb… 🌐 | confirmed | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 8, stores AbilityPacketUpdateAbilityExperience::vftable, and zeroes the six dwords from +0xc t… |
| `00a2df90` | AbilityPacketUpdateAbilityExperience::ScalarDeletingDtor | likely | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; ra… |

**BaseAbilityPacket** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2cb10` | BaseAbilityPacket::BaseAbilityPacket 🌐 | confirmed | Writes BasePacket::vftable then BaseAbilityPacket::vftable at +0, family id 0x24 at +4 and a zero sub-opcode at +8. |
| `00a2cbe0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2cca0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2ce20` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2ced0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2cf90` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d070` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d120` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d170` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d220` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d2d0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d3e0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d4a0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d550` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d5a0` | BaseAbilityPacket::~BaseAbilityPacket | confirmed | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2cd80` | ScalarDeletingDtorFor8ByteBlock | likely | Calls the member destructor FUN_01046050 and then releases the block with _free when bit 0 of the flag argument is set. |
| `00a2d380` | DestroyAbilityDefinitionMember | likely | Runs the embedded member destructor FUN_010647c0 and then resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable. |
| `00a2f550` | AbilityPacketLaunchAndLandPayloadDtor | likely | Runs the projectile-parameter destructor FUN_008d0240, restores the SoeUtil::Array<Target,0,1>::vftable at +0x18, destroys that array (FUN_008bedb0, then frees… |
| `00a2f650` | AbilityPacketExecuteClientLuaDtor | likely | Restores the SoeUtil::IString<char>::vftable of the owned string at +0xc; when length at +0x18 is positive, atomically decrements its buffer-header refcount an… |
| `00a2fab0` | ReadCountedVtableElementArray 🌐 | likely | Reads an element count from the first argument's packet-reader view, resizes the array owned by the second argument (00a2ec70 when growing and 008ac420 when sh… |
| `00a2fb30` | AbilityPacketLaunchAndLandDeletingDtor | likely | Runs the launch-and-land payload destructor 00a2f550 and then releases the object with _free when bit 0 of the flag argument is set. |
| `00a2fb50` | AbilityPacketExecuteClientLuaDeletingDtor | likely | Runs the execute-client-Lua destructor 00a2f650 and then releases the object with _free when bit 0 of the flag argument is set. |
| `00a2fc10` | ReadInt16Pair 🌐 | likely | Reads two little-endian 16-bit values from the bounded reader (cursor +8, end +0xc, overflow +0x10) into the result object's +4 and +8, advancing two bytes per… |
| `00a2fc70` | ReadPayloadTwoIdPairsFourDwordsAndByte 🌐 | likely | Calls ReadInt16Pair (00a2fc10), reads two 8-byte pairs into +0x10 and +0x18, four dwords into +0x20/+0x24/+0x28/+0x2c, and one byte into +0x30; NaN at +0x2c se… |
| `00a2fdb0` | ReadPayloadIdPairAndDword 🌐 | likely | Calls ReadInt16Pair (00a2fc10), reads one 8-byte pair into +0x10 and one dword into +0x18, zeroing a field and setting overflow on short read. |
| `00a2fe20` | ReadPayloadIdPairAndThreeDwords 🌐 | likely | Calls ReadInt16Pair (00a2fc10), reads one 8-byte pair into +0x10 and three dwords into +0x18/+0x1c/+0x20; NaN at +0x20 sets reader flag +0x11, and short reads … |

### `net.packet.activity`

**ActivityJoinErrorPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be0090` | ActivityJoinErrorPacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of ActivityJoinErrorPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d824… |
| `00bdec00` | ActivityJoinErrorPacket_Ctor 🌐 | likely | Constructs the opcode-147 sub-opcode-2 packet with FUN_00bdeb20(2) and installs ActivityJoinErrorPacket::vftable. |
| `00bdec60` | ActivityJoinErrorPacket::~ActivityJoinErrorPacket | likely | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; called at the end of the join-error branch of 00be7360 and referenced b… |

**ActivityManagerBasePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdeac0` | ActivityManagerBasePacket_Ctor 🌐 | confirmed | Constructor of the opcode 147 family: stores BasePacket::vftable, writes the family opcode 0x93 (147) into the packet field at +4, installs ActivityManagerBase… |
| `00bdeb20` | ActivityManagerBasePacket_CtorWithSubOpcode | confirmed | Constructor variant of the opcode 147 family that takes the sub-opcode as an argument: stores BasePacket::vftable, writes 0x93 (147) into +4, installs Activity… |
| `00be0030` | ActivityManagerBasePacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of ActivityManagerBasePacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d8… |
| `00bdeb90` | ActivityManagerBasePacket::~ActivityManagerBasePacket | likely | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; called at the end of every branch of the opcode-147 handler 00be7360 an… |

**ActivityPacketActivityListRefreshRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2330` | ActivityPacketActivityListRefreshRequest::ActivityPacketAct… 🌐 | confirmed | Constructor: calls FUN_008a21f0(4) so the packet is opcode 167 with activity sub-opcode 4, then installs ActivityPacketActivityListRefreshRequest::vftable. |
| `008a2390` | ActivityPacketActivityListRefreshRequest::~ActivityPacketAc… | likely | Destructor body: restores BaseActivityPacket, BaseActivityServicePacket and BasePacket vtables; called by the request senders 009392c0 and 009471a0 plus four E… |
| `008b5f70` | ActivityPacketActivityListRefreshRequest::ScalarDeletingDes… | likely | Stores BaseActivityPacket, BaseActivityServicePacket, then BasePacket vtable pointers at *this; frees this only when deleting-flag bit 0 is set. |

**ActivityPacketJoinActivityRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be0120` | ActivityPacketJoinActivityRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vftable to BaseActivityPacket, then BaseActivityServicePacket, then BasePacket; frees this through _free (thunk) only wh… |
| `00bdefa0` | ActivityPacketJoinActivityRequest_Ctor 🌐 | likely | Constructs the request with FUN_008a21f0(2) (the BaseActivityPacket constructor with activity sub-opcode 2) and installs ActivityPacketJoinActivityRequest::vft… |

**ActivityPacketListOfActivities** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be5590` | ActivityPacketListOfActivities::Ctor | confirmed | Constructor body: calls FUN_008a21f0 with the constant 1 (the activity base constructor with sub-opcode), stores the ActivityPacketListOfActivities vtable at *… |
| `00be6190` | ActivityPacketListOfActivities_ScalarDtor | likely | Calls 00be5600 (list destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set (ret 4); returns this. |

**ActivityPacketUpdateActivityFeaturedStatus** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be0190` | ActivityPacketUpdateActivityFeaturedStatus::ScalarDeletingD… | confirmed | Scalar deleting destructor: resets the vftable to BaseActivityPacket, then BaseActivityServicePacket, then BasePacket; frees this through _free (thunk) only wh… |
| `00bdf0c0` | ActivityPacketUpdateActivityFeaturedStatus::~ActivityPacket… | likely | Destructor body: stores BaseActivityPacket::vftable, BaseActivityServicePacket::vftable and finally BasePacket::vftable at object+0, with no member cleanup. |

**ActivityProfileListPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be5490` | ActivityProfileListPacket::Ctor | confirmed | Constructor body: calls FUN_00bdeb20 with sub-opcode 1 (the ActivityManagerBase sub-opcode constructor), stores the ActivityProfileListPacket vtable at *this, … |
| `00be54f0` | ActivityProfileListPacket::DtorBody | confirmed | Destructor body with an unwind frame: stores the ActivityProfileListPacket vtable, empties the embedded ActivityForProfileType hash list through 00be2220 (afte… |
| `00be5570` | ActivityProfileListPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): calls 00be54f0 (the packet destructor body) on this, calls _free when bit 0 of the flag argument is set, and returns this. |

**BaseActivityPacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a21f0` | BaseActivityPacket::BaseActivityPacket | confirmed | Activity base constructor taking the activity sub-opcode: calls FUN_008a20c0(1) so the packet header carries opcode 167 with service sub-opcode 1, installs Bas… |
| `008a2260` | BaseActivityPacket::~BaseActivityPacket | confirmed | Destructor body: restores BaseActivityPacket::vftable, BaseActivityServicePacket::vftable then BasePacket::vftable; 13 EH funclets call it. |
| `008a22c0` | BaseActivityPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the activity, activity-service and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `00bdeda0` | BaseActivityPacket_Ctor 🌐 | likely | Constructs the activity packet with FUN_008a20c0(1) (service 167, service-type 1) and installs BaseActivityPacket::vftable, storing 0 in the field at +0xc. |
| `00bdf000` | BaseActivityPacket::~BaseActivityPacket | likely | Destructor body: stores BaseActivityPacket::vftable, BaseActivityServicePacket::vftable and finally BasePacket::vftable at object+0, with no member cleanup. |

**BaseActivityServicePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a20c0` | BaseActivityServicePacket::BaseActivityServicePacket | confirmed | Activity-service family constructor: installs BasePacket::vftable, stores 0xa7 (167) in the packet id slot param_1[1], installs BaseActivityServicePacket::vfta… |
| `008a2130` | BaseActivityServicePacket::~BaseActivityServicePacket | confirmed | Destructor body: restores BaseActivityServicePacket::vftable then BasePacket::vftable; called from 18 EH unwind funclets, which is why it is the shared destruc… |
| `00bded40` | BaseActivityServicePacket_Ctor 🌐 | confirmed | Family constructor of the activity-service packets: stores BasePacket::vftable, writes the family opcode 0xa7 (167) into the packet field at +4, installs BaseA… |
| `008a2180` | BaseActivityServicePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the activity-service and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |

**BaseScheduledActivityPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdef30` | BaseScheduledActivityPacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of BaseScheduledActivityPacket::vftable): stores the three vtable levels (BaseScheduledActivityPacket, BaseActivityS… |
| `00bdee00` | BaseScheduledActivityPacket_Ctor 🌐 | likely | Constructs the scheduled-activity packet with FUN_008a20c0(2) (service 167, service-type 2) and installs BaseScheduledActivityPacket::vftable, storing 0 in the… |
| `00bdee60` | BaseScheduledActivityPacket_CtorWithPayload 🌐 | likely | Variant of the scheduled-activity constructor that also stores its argument in the packet field at +0xc after installing BaseScheduledActivityPacket::vftable. |
| `00bdeed0` | BaseScheduledActivityPacket::~BaseScheduledActivityPacket | likely | Destructor body of the scheduled-activity family: stores BaseScheduledActivityPacket::vftable, then BaseActivityServicePacket::vftable and finally BasePacket::… |

**ClientActivityLaunchBasePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c0c30` | ClientActivityLaunchBasePacketInit | confirmed | Stores opcode 0xaf, writes supplied fields in packet slots +3 and +4, and stores the activity-launch base vtable. |
| `009c0cb0` | ClientActivityLaunchBasePacketDtor | likely | Rewrites the object vtable pointer to BasePacket::vftable and returns without freeing the object. |

**ClientActivityLaunchMember** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b050` | ClientActivityLaunchMember_Construct | likely | Constructs one launch-member record: stores the member id at +0, the two guid/id dwords at +8 and +0xc, runs the member initialiser 00703540, clears the bytes … |

**ClientActivityLaunchPacketActivityCanceled** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c0d00` | ClientActivityLaunchPacketActivityCanceledCtor | confirmed | Initializes the family packet with sub-opcode 6 and supplied fields, installs the canceled-packet vtable, and zeroes its trailing field. |
| `009c1260` | ClientActivityLaunchPacketActivityCanceledScalarDeletingDtor | confirmed | Rewrites the object vtable to BasePacket::vftable and frees the allocation when delete flag bit 0 is set. |

**ClientActivityLaunchPacketActivityLaunched** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b300` | ClientActivityLaunchPacketActivityLaunched::ClientActivityL… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 5, installs its vtable, and initialises an embedded SoeUtil guid list at +0x14 (list vtable plus zeroed head … |
| `00c4bb50` | ClientActivityLaunchPacketActivityLaunched::ScalarDeletingD… | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4b380 (which destroys the embedded guid list) and frees the object with CRT _free when… |
| `00c4b380` | ClientActivityLaunchPacketActivityLaunched::~ClientActivity… | likely | Destructor: re-installs the embedded guid-list vtable at +0x14, destroys that list through 008d3ee0 and rewrites *this to BasePacket::vftable. |

**ClientActivityLaunchPacketAddMember** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b3e0` | ClientActivityLaunchPacketAddMember::ClientActivityLaunchPa… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 7, installs its vtable and initialises the embedded member through 009c21b0. |
| `00c4b450` | ClientActivityLaunchPacketAddMember::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4aa00 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4aa00` | ClientActivityLaunchPacketAddMember::~ClientActivityLaunchP… | likely | Destructor: destroys the embedded member object through 00703350 and rewrites *this to BasePacket::vftable. |

**ClientActivityLaunchPacketInviteDetails<SoeUtil::List<ClientActivityLaunchMember,-1>>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4d590` | ClientActivityLaunchPacketInviteDetails::ClientActivityLaun… 🌐 | confirmed | Constructor of the invite-details packet (template instance with an external member list): family base 009c0c30 with sub-opcode 1, installs its vtable, install… |
| `00c4d650` | ClientActivityLaunchPacketInviteDetails::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4d410 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4d410` | ClientActivityLaunchPacketInviteDetails::~ClientActivityLau… | likely | Destructor of the invite-details packet: destroys the member subobject through 00703350, installs the member-list vtable at +0x14 and drains that list with 00c… |

**ClientActivityLaunchPacketInviteDetails<SoeUtil::List<ClientActivityLaunchPacketMember,-1>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c3160` | ActivityLaunchInviteDetailsCtor | confirmed | Initializes the invite-details packet and its member list, sets default fields and two shared constants, then clears a later field. |
| `009c3200` | ActivityLaunchInviteDetailsScalarDeletingDtor | confirmed | Calls packet teardown, conditionally frees the allocation from the deleting flag, and otherwise returns the original pointer. |
| `009c30d0` | ClientActivityLaunchPacketInviteDetailsDtor | likely | Calls the IStringFixed cleanup on the subobject at +0x38, restores the member-list vtable at +0x14 and removes its nodes, then restores BasePacket::vftable at … |

**ClientActivityLaunchPacketInviteMemberReply** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b1a0` | ClientActivityLaunchPacketInviteMemberReply::ClientActivity… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 2, installs its vtable, initialises the member guid to the default pair, runs the member initialiser, and cle… |
| `00c4b230` | ClientActivityLaunchPacketInviteMemberReply::ScalarDeleting… | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4a940 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4a940` | ClientActivityLaunchPacketInviteMemberReply::~ClientActivit… | likely | Destructor: destroys the embedded member object through 00703350 and rewrites *this to BasePacket::vftable. |
| `00c4acd0` | InviteMemberReply_ReadFields | likely | Field reader for an invite-member-reply packet: calls the shared header reader 009c1e00, then reads a 64-bit guid into +0x18/+0x1c, a length-prefixed string th… |

**ClientActivityLaunchPacketInviteMemberRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b100` | ClientActivityLaunchPacketInviteMemberRequest::ClientActivi… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 0x14 (20), installs its vtable, initialises the member guid to the default pair DAT_01bee130/DAT_01bee134 and… |
| `00c4b180` | ClientActivityLaunchPacketInviteMemberRequest::ScalarDeleti… | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4a8e0 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4a8e0` | ClientActivityLaunchPacketInviteMemberRequest::~ClientActiv… | likely | Destructor: destroys the embedded member object through 00703350 and then rewrites *this to BasePacket::vftable; it does not free the object. |

**ClientActivityLaunchPacketInviteMemberResult** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b250` | ClientActivityLaunchPacketInviteMemberResult::ClientActivit… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 3, installs its vtable, initialises the member guid to the default pair, runs the member initialiser, and cle… |
| `00c4b2e0` | ClientActivityLaunchPacketInviteMemberResult::ScalarDeletin… | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4a9a0 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4a9a0` | ClientActivityLaunchPacketInviteMemberResult::~ClientActivi… | likely | Destructor: destroys the embedded member object through 00703350 and rewrites *this to BasePacket::vftable. |
| `00c4adb0` | InviteMemberResult_ReadFields | likely | Field reader with the same layout as 00c4acd0 (guid at +0x18/+0x1c, string via 00894b10, byte at +0x54, dword at +0x58) but called from the invite-member-resul… |

**ClientActivityLaunchPacketInviteResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49610` | ClientActivityLaunchPacketInviteResponse::ClientActivityLau… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x11 and the two header arguments, installs ClientActivityLaunchPacketInviteResponse::v… |
| `00c49fd0` | ClientActivityLaunchPacketInviteResponse::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this (the trivial destructor body) and, when bit 0 of the flag is set, frees the obj… |

**ClientActivityLaunchPacketKickMemberEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b510` | ClientActivityLaunchPacketKickMemberEvent::ClientActivityLa… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 0xd (13), installs its vtable, initialises the first member (default guid pair plus initialiser 00703540) and… |
| `00c4b5b0` | ClientActivityLaunchPacketKickMemberEvent::ScalarDeletingDt… | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4aba0 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4aba0` | ClientActivityLaunchPacketKickMemberEvent::~ClientActivityL… | likely | Destructor of the two-member kick-event packet: destroys both embedded member objects through 00703350 (the second inside an EH scope) and rewrites *this to Ba… |
| `00c4af20` | KickMemberEvent_ReadFields | likely | Field reader for a kick-event packet: shared header read 009c1e00, then a 64-bit guid at +0x18/+0x1c, a length-prefixed string via 00894b10, a second 64-bit gu… |

**ClientActivityLaunchPacketKickMemberRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4b470` | ClientActivityLaunchPacketKickMemberRequest::ClientActivity… 🌐 | confirmed | Constructor: family base 009c0c30 with sub-opcode 0x15 (21), installs its vtable, initialises the target guid to the default pair DAT_01bee130/DAT_01bee134 and… |
| `00c4b4f0` | ClientActivityLaunchPacketKickMemberRequest::ScalarDeleting… | confirmed | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4ab40 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4ab40` | ClientActivityLaunchPacketKickMemberRequest::~ClientActivit… | likely | Destructor: destroys the embedded member object through 00703350 and rewrites *this to BasePacket::vftable. |

**ClientActivityLaunchPacketMemberQuit** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49960` | ClientActivityLaunchPacketMemberQuit::ClientActivityLaunchP… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x13 (19) and the two header arguments, then installs ClientActivityLaunchPacketMemberQ… |
| `00c4a150` | ClientActivityLaunchPacketMemberQuit::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |

**ClientActivityLaunchPacketMemberReady** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c498a0` | ClientActivityLaunchPacketMemberReady::ClientActivityLaunch… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x12 (18) and the two header arguments, then installs ClientActivityLaunchPacketMemberR… |
| `00c4a0f0` | ClientActivityLaunchPacketMemberReady::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set; the bytes co… |

**ClientActivityLaunchPacketOnMatchmakingCancel** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49d20` | ClientActivityLaunchPacketOnMatchmakingCancel::ClientActivi… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x10 (16) and the two header arguments, then installs ClientActivityLaunchPacketOnMatch… |
| `00c4a330` | ClientActivityLaunchPacketOnMatchmakingCancel::ScalarDeleti… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set; the body con… |

**ClientActivityLaunchPacketOnMatchmakingLaunch** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49c60` | ClientActivityLaunchPacketOnMatchmakingLaunch::ClientActivi… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0xf (15) and the two header arguments, then installs ClientActivityLaunchPacketOnMatchm… |
| `00c4a2d0` | ClientActivityLaunchPacketOnMatchmakingLaunch::ScalarDeleti… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |

**ClientActivityLaunchPacketOnMatchmakingStart** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49ba0` | ClientActivityLaunchPacketOnMatchmakingStart::ClientActivit… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0xe (14) and the two header arguments, then installs ClientActivityLaunchPacketOnMatchm… |
| `00c4a270` | ClientActivityLaunchPacketOnMatchmakingStart::ScalarDeletin… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |

**ClientActivityLaunchPacketOwnerLaunchRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49a20` | ClientActivityLaunchPacketOwnerLaunchRequest::ClientActivit… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x16 (22) and the two header arguments, then installs ClientActivityLaunchPacketOwnerLa… |
| `00c4a1b0` | ClientActivityLaunchPacketOwnerLaunchRequest::ScalarDeletin… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |

**ClientActivityLaunchPacketOwnerMatchmakingRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49ae0` | ClientActivityLaunchPacketOwnerMatchmakingRequest::Ctor 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x17 (23) and the two header arguments, then installs ClientActivityLaunchPacketOwnerMa… |
| `00c4a210` | ClientActivityLaunchPacketOwnerMatchmakingRequest::ScalarDe… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |

**ClientActivityLaunchPacketPlayerHasBeenAccepted** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4a030` | ClientActivityLaunchPacketPlayerHasBeenAccepted::ScalarDele… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |

**ClientActivityLaunchPacketRemoveMember** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c49790` | ClientActivityLaunchPacketRemoveMember::ClientActivityLaunc… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 8 and the two header arguments, installs ClientActivityLaunchPacketRemoveMember::vftabl… |
| `00c4a090` | ClientActivityLaunchPacketRemoveMember::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4aeb0` | RemoveMember_ReadFields | likely | Field reader for a remove-member packet: shared header read 009c1e00, then two 32-bit fields written at +0x14 and +0x18 with stream bound checks. |

**ClientActivityLaunchPacketUpdateMemberState** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4aa70` | ClientActivityLaunchPacketUpdateMemberState::ClientActivity… 🌐 | confirmed | Constructor: calls the family base constructor 009c0c30 with sub-opcode 9 and the two header arguments, installs ClientActivityLaunchPacketUpdateMemberState::v… |
| `00c4aae0` | ClientActivityLaunchPacketUpdateMemberState::ScalarDeleting… | confirmed | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4b9e0` | UpdateMemberState_ReadFields | likely | Field reader for the member-state update: shared header read 009c1e00, then a 32-bit member id into +0x14 and one byte into +0x18 with stream bound checks. |

**ClientActivityLaunchRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4d160` | WriteInviteResponseFields | likely | Serialises an invite response: first writes the shared launch-request fields through 00c4cd20, then appends a single byte taken from this+0x14 and a 32-bit val… |

**ScheduledActivityPacketListOfActivities** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be5690` | ScheduledActivityPacketListOfActivities::Ctor | confirmed | Constructor body: calls FUN_00bdee60 with the constant 1 (the scheduled-activity base constructor with payload), stores the ScheduledActivityPacketListOfActivi… |
| `00be61b0` | ScheduledActivityPacketListOfActivities_ScalarDtor | likely | Calls 00be5750 (list destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set (ret 4); returns this. |

**(no class)** (32)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c29d0` | HandleActivityLaunchCancellation | confirmed | Parses the cancellation body to obtain a request id, searches the request list and processor map, invokes a matching processor callback, logs a missing request… |
| `009c3590` | DispatchActivityLaunchMessage 🌐 | confirmed | Parses a leading message discriminator, routes value 1 to the activity launch handler and value 6 to another handler, and dispatches remaining recognized messa… |
| `009c0d70` | BasePacketDtor | likely | Rewrites the object vtable pointer to BasePacket::vftable and returns; an EH unwind caller establishes destructor use. |
| `009c1e00` | ReadActivityLaunchHeaderFields | likely | Reads one signed 16-bit value followed by three 32-bit values from the bounded reader, zero-filling missing fields and marking failure on overrun. |
| `009c1f40` | ReadActivityLaunchRequestRecord | likely | Reads paired 32-bit values, scalar and boolean fields, trailing integers, and a variable-length field into record offsets +0x18 through +0x4c, zeroing fields w… |
| `009c2780` | ReadActivityLaunchId | likely | Parses the bounded input prefix and copies the first 32-bit value to output offset +0x14; reports failure when unavailable. |
| `009c2ec0` | ReadLaunchRequestFields | likely | Reads bounded fields from the supplied packet cursor into offsets 0x2c, 0x30, 0x34, and 0x6c, marking the cursor failed on truncation. |
| `00be7200` | ScheduledActivityPacket_ParseApply | likely | Constructs a local ScheduledActivityPacketListOfActivities through 00be5690, reads one 0x80-layout record from the bounded stream param_2 through 00be62d0 and,… |
| `00be7660` | ScheduledActivityPacket_ParseIfFlagged | likely | Constructs a local BaseScheduledActivityPacket through 00bdee00, builds a bounded stream from param_1 (start) and param_2 (length) and reads a record through 0… |
| `00be79d0` | ActivityPacketListOfActivities_ParseApply | likely | Constructs a local ActivityPacketListOfActivities through 00be5590, reads a record from bounded stream param_2 through 00be75c0 and, when the stream has no EOF… |
| `00be7b30` | ActivityPacket_DispatchSubOp | likely | Constructs a local BaseActivityPacket through 00bdeda0, reads one record from bounded stream param_2 through 00be2e80 into a local whose byte at +0xc is the su… |
| `00be7c50` | ActivityFamilyPacket_Dispatch 🌐 | likely | If the input pointer is nonnull and holds at least two bytes, reads the 16-bit id at the start: 0x93 (147) calls 00be7360 with the buffer; 0xa7 (167) construct… |
| `00be7d80` | ActivityJoinRequest_SendByType | likely | Looks up activity id param_2 in the 1024-bucket table at this+0x18 (bucket id & 0x3ff, key at node+0xc8, next at node+0xcc) and returns 0 when absent; otherwis… |
| `00c4a810` | MemberList_ReadFromStream | likely | Clears the target list (008d3ee0) and then reads a 32-bit count followed by that many pairs of dwords from the stream param_1, appending one list node per entr… |
| `00c4ac70` | ReleaseIStringFixed32At10 | likely | EH-protected cleanup helper that calls 00703350 for an IStringFixed<char,32> subobject at this+0x10, then restores the exception-list pointer. No caller is rec… |
| `00c4b620` | ClientActivityLaunchMember_ConstructWrapper | likely | Wrapper that forwards its second argument to the member constructor 00c4b050 and returns the first argument; used to construct a launch member from a descripto… |
| `00c4b840` | ClientActivityLaunchMember_ScalarDeletingDtor | likely | Ninety-seven byte scalar-deleting-destructor-shaped routine: destroys the member subobject through 00703350 and frees the object with CRT _free when bit 0 of t… |
| `00c4b8b0` | ActivityLaunchMemberNode_ConstructFromDescriptor | likely | Tiny wrapper that calls 00c4b620 with the descriptor pointer stored at *param_2, i.e. it constructs the member of a freshly allocated node from a caller-suppli… |
| `00c4ba60` | NodeMember_ConstructWrapperB | likely | Seventy-two byte wrapper identical in shape to 00c4b5d0: calls the member initialiser 009c21b0 and returns its argument; used by the append helper 00c4bf90. |
| `00c4bca0` | ActivityLaunchPacketParse_00c4bca0 | likely | Fifty-nine byte buffer-parse wrapper: builds the standard stream object over (param_1, param_2), calls only the shared launch header reader 009c1e00 and return… |
| `00c4bce0` | ActivityLaunchPacketParse_00c4bce0 | likely | Fifty-nine byte buffer-parse wrapper: builds the stream over (param_1, param_2) and calls the invite-member-reply field reader 00c4acd0, returning the stream e… |
| `00c4bd20` | ActivityLaunchPacketParse_00c4bd20 | likely | Fifty-nine byte buffer-parse wrapper calling the invite-member-result field reader 00c4adb0 and returning the stream error byte. |
| `00c4bd60` | ReadHeaderAndCountedPairList | likely | Builds a bounded stream over the supplied buffer, reads the shared header through 009c1e00, then invokes 00c4a810 with the destination at param_3+0x14. The hel… |
| `00c4bdc0` | ActivityLaunchPacketParse_00c4bdc0 | likely | Fifty-nine byte buffer-parse wrapper calling the remove-member field reader 00c4aeb0 and returning the stream error byte. |
| `00c4be00` | ActivityLaunchPacketParse_00c4be00 | likely | Fifty-nine byte buffer-parse wrapper calling the kick-event field reader 00c4af20 and returning the stream error byte. |
| `00c4be40` | ActivityLaunchPacketParse_00c4be40 | likely | Fifty-nine byte buffer-parse wrapper that builds the stream and calls only the shared launch header reader 009c1e00, returning the stream error byte. |
| `00c4be80` | ActivityLaunchPacketParse_00c4be80 | likely | Fifty-nine byte buffer-parse wrapper that builds the stream and calls only the shared launch header reader 009c1e00, returning the stream error byte. |
| `00c4bec0` | ActivityLaunchPacketParse_00c4bec0 | likely | Fifty-nine byte buffer-parse wrapper that builds the stream and calls only the shared launch header reader 009c1e00, returning the stream error byte. |
| `00c4ccb0` | InviteDetails_ReadMemberRoster | likely | Clears the supplied list, reads a count from the bounded stream, then appends that many member nodes with 00c4bf90 and reads each node through 009c24e0; stream… |
| `00c4cfd0` | ActivityLaunchPacketParse_00c4cfd0 | likely | Seventy-seven byte buffer-parse wrapper: builds the stream over (param_1, param_2), calls the shared header reader 009c1e00 and then the member-field reader 00… |
| `00c4d060` | InviteDetails_ReadFields | likely | Reads the shared header, a byte at +0x2c, a pair of dwords at +0x30/+0x34, a length-prefixed string into the packet object, a dword at +0x6c, and optional list… |
| `00c4d750` | ActivityLaunchPacketParse_00c4d750 | likely | Fifty-nine byte buffer-parse wrapper calling the invite-details field reader 00c4d060 and returning the stream error byte. |

### `net.packet.battlemages`

**BattleMagesProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b71670` | BattleMagesProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the processor destructor FUN_00b71320 (argument-less call in the pseudocode) and, when the flag byte's low bit is set, calls … |
| `00b728a0` | BattleMagesProcessor::DispatchPacket 🌐 | confirmed | Virtual slot 4 of BattleMagesProcessor: builds a stack BaseBattleMagesPacket temp (ctor FUN_00b6f250), reads the two-u16 inner header from (data, length) with … |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b71490` | BattleMagesReadHeaderU32 🌐 | likely | Calls FUN_00b70e40 to read the two-u16 BattleMages inner header into out+4/out+8, then reads one 32-bit payload word into out+0xc; when fewer than 4 bytes rema… |
| `00b71530` | BattleMagesReadHeaderU32b 🌐 | likely | Byte-for-byte the same body as FUN_00b71490: reads the two-u16 inner header through FUN_00b70e40 and then one 32-bit payload word into out+0xc, zeroing it and … |
| `00b71570` | BattleMagesReadHeaderU32x2Vec4 🌐 | likely | Reads the two-u16 inner header plus two 32-bit words into out+0x10/out+0x14 and then a 16-byte vector into out+0x20 via FUN_008e2410; short reads zero the affe… |
| `00b718e0` | BattleMagesReadCreateProjectileFields 🌐 | likely | Reads the two-u16 inner header plus one dword into out+0xc, two more dwords into out+0x10/out+0x14, four 16-byte vectors into out+0x20/+0x30/+0x40/+0x50 via FU… |
| `00b71a70` | BattleMagesReadUpdateProjectileFields 🌐 | likely | Reads the two-u16 inner header and one dword (out+0xc) and then six consecutive 16-byte vectors into out+0x10 through out+0x60 via FUN_008e2410; short reads ze… |
| `00b71ef0` | BattleMagesHandleDestroyProxiedProjectile 🌐 | likely | Builds a BattleMagesPacketDestroyProxiedProjectile temp (ctor FUN_00b6f670), reads the inner header plus one dword (an entity id) with FUN_00b71530, walks the … |
| `00b72020` | BattleMagesReadUpdateDataPayload 🌐 | likely | Builds a reader state over (buffer, length) and calls FUN_00b71570 to fill the caller's object with the inner header, two dwords and one vector; returns 1 when… |
| `00b72090` | BattleMagesReadCameraConfigPayload 🌐 | likely | Reads the two-u16 inner header via FUN_00b70e40, then one dword into out+0xc (zeroed and flagged when short), then delegates to FUN_00b71000 for the remaining … |
| `00b72450` | BattleMagesWriteHeaderEightByteValueAndTail 🌐 | likely | Serialises a message body into the bit-stream: delegates the two-u16 header to FUN_00b72160, writes an eight-byte value from source+0x10 through FUN_008fb810, … |
| `00b72dd0` | BattleMagesSendRegisterPlayer 🌐 | likely | When given a live entity and nonzero flag, constructs a BattleMagesPacketRegisterPlayer temp (ctor FUN_00b6fc50, sub 3), calls the client interface slot +0x10 … |
| `00b72e80` | BattleMagesSendLocalPlayerTail 🌐 | likely | Fetches the entity guid through the client interface slot +0x10, reads the local player object at client+0xe8 and builds a body with the guid, the dword at pla… |

### `net.packet.broker`

**BaseBrokerPacket** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0c6e0` | BaseBrokerPacket_DtorBody | confirmed | Base-class destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; called from the Broker.Search handler 00c0e110 and from … |
| `00c0c7a0` | BaseBrokerPacket_DtorBody_00c0c7a0 | confirmed | Copy of the BaseBrokerPacket destructor body in this translation unit: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached from … |
| `00c0c7f0` | BaseBrokerPacket_DtorBody_00c0c7f0 | confirmed | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached from the Broker.Relist path 00… |
| `00c0c840` | BaseBrokerPacket_DtorBody_00c0c840 | confirmed | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached from the Broker.Buy path 00c0d… |
| `00cee280` | BaseBrokerPacketDtor | confirmed | Resets the object's vtable first to BaseBrokerPacket and then to BasePacket. |
| `00cee340` | BaseBrokerPacketDtor | confirmed | Resets the object's vtable first to BaseBrokerPacket and then to BasePacket. |
| `00c0c890` | BaseBrokerPacket_DtorBody_00c0c890 | likely | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached only from the unwind funclet 0… |
| `00c0c8e0` | BaseBrokerPacket_DtorBody_00c0c8e0 | likely | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached only from the unwind funclet 0… |

**BrokerBuyItemPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0cbe0` | BrokerBuyItemPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of BrokerBuyItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free w… |

**BrokerCancelItemPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0ccc0` | BrokerCancelItemPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of BrokerCancelItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _fre… |

**BrokerMyItemsPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cee2d0` | BrokerMyItemsPacket::Ctor | confirmed | Calls the base packet initializer with subtype 8, installs BrokerMyItemsPacket vtable, and stores the constructor argument at field 3. |
| `00ceeb00` | BrokerMyItemsPacket::ScalarDeletingDtor | confirmed | Restores BaseBrokerPacket then BasePacket vtables, and frees the object when the deleting bit is set. |

**BrokerPlaceItemPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0ca10` | BrokerPlaceItemPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of BrokerPlaceItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free… |

**BrokerReListItemPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0cb00` | BrokerReListItemPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of BrokerReListItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _fre… |
| `00c0ca80` | BrokerReListItemPacket_Ctor 🌐 | likely | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(4), installs BrokerReListItemPacket::vftable, copies two static default words (DAT_01… |

**BrokerRequestItemSaleCoinPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0cda0` | BrokerRequestItemSaleCoinPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of BrokerRequestItemSaleCoinPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and c… |

**BrokerSearchItemsPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceea10` | BrokerSearchItemsPacket::Ctor | confirmed | Calls the base initializer with subtype 10, installs BrokerSearchItemsPacket vtable, stores two default field values and the constructor argument. |
| `00ceea90` | BrokerSearchItemsPacket::ScalarDeletingDtor | confirmed | Restores BaseBrokerPacket then BasePacket vtables, and frees the object when the deleting bit is set. |

**BrokerSearchPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0c9a0` | BrokerSearchPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of BrokerSearchPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free wh… |

**(no class)** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0cef0` | BrokerPacket_WriteCommonHeader | likely | Writes the two leading 16-bit fields of a broker packet body: the low short of the object's +4 dword and the low short of its +8 dword, each through the bounde… |
| `00c0cfa0` | BrokerPlaceItemPacket_SerializeBody | likely | Broker place-item body writer: calls the shared header writer FUN_00c0cef0, then appends the 32-bit fields at +0xc, +0x14, +0x18, +0x10 and +0x1c through the b… |
| `00c0d140` | BrokerBuyItemPacket_SerializeBody | likely | Broker buy body writer: calls the shared header writer FUN_00c0cef0, writes the structured field at +0x10 through FUN_008fd1a0 and appends the 32-bit field at … |
| `00c0d1a0` | BrokerReListItemPacket_SerializeBody | likely | Broker relist body writer: calls the shared header writer FUN_00c0cef0, writes the structured field at +0x10 through FUN_008fd1a0 and appends the 32-bit fields… |
| `00c0d590` | SendBrokerPlaceItemPacket_Gateway | likely | ClientGateway send template for the broker place-item packet: serialises the source through FUN_00c0cfa0 into a local buffer, finalises it, constructs ClientGa… |
| `00c0d6a0` | SendBrokerCancelItemPacket_Gateway | likely | ClientGateway send template for the broker cancel packet: writes the shared two-short header (FUN_00c0cef0) and the structured +0x10 field (FUN_008fd1a0) into … |
| `00c0d7c0` | SendBrokerRequestItemSaleCoinPacket_Gateway | likely | ClientGateway send template for the broker sale-coin request: identical body to the cancel template (shared header FUN_00c0cef0 plus the structured +0x10 field… |
| `00c0d8e0` | SendBrokerBuyItemPacket_Gateway | likely | ClientGateway send template for the broker buy packet: serialises the source through FUN_00c0d140 into a local buffer, finalises it, constructs ClientGateway::… |
| `00c0d9f0` | SendBrokerReListItemPacket_Gateway | likely | ClientGateway send template for the broker relist packet: serialises the source through FUN_00c0d1a0 into a local buffer, finalises it, constructs ClientGatewa… |
| `00c0db00` | SendBrokerSearchPacket_Gateway | likely | ClientGateway send template for the broker search packet: serialises the source through FUN_00c0d2c0 into a local buffer, finalises it, constructs ClientGatewa… |
| `00cefdf0` | ReadBrokerItemPairAndNextRow | likely | Reads the broker row's leading fields, checks for two remaining dwords, copies them to the output, advances the reader by eight bytes, and parses the following… |
| `00cf0180` | ReadBrokerItemPairAndNextRow | likely | Reads a two-short header, checks for two more dwords, stores or zeroes that pair, advances the bounded reader when available, then parses subsequent row data. |
| `00cf02a0` | ReadBrokerItemPacketRange | likely | Validates nonnull input and length, initializes a bounded reader, parses a broker item row, copies two output words when in range, and returns a status byte. |
| `00cf0340` | ParseBrokerSearchResults | likely | Initializes result-list state, parses bounded search results, updates the broker row list, removes entries failing the active filter, and marks the list change… |
| `00cf0590` | ParseBrokerMyItems | likely | Constructs a broker item-list packet object, parses bounded item rows, updates item-definition requests and broker lists, then notifies the owning object when … |

### `net.packet.client_path`

**ClientPathBasePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4800` | ClientPathBasePacket::ClientPathBasePacket 🌐 | confirmed | Client-path family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 98 (0x62) at +4, installs ClientPathBasePacket::vftable and … |
| `008a4860` | ClientPathBasePacket::ClientPathBasePacketSub 🌐 | confirmed | Client-path family constructor taking the sub-opcode: installs BasePacket::vftable, stores opcode 98 (0x62) at +4, installs ClientPathBasePacket::vftable and s… |
| `008b7d90` | ClientPathBasePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a48d0` | ClientPathBasePacket::~ClientPathBasePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the 009471a0 router destroys the packet with it at the end of case 0x62, a… |

**ClientPathReplyPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fe1f0` | ClientPathReplyPacket::ScalarDeletingDtor | confirmed | thiscall with one byte argument (delete flags, ret 4). Calls 008eba90 on this (accepted label BasePacketDtorWithVectorList: the base packet destructor body), t… |

**ClientPathRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b7df0` | ClientPathRequestPacket::ClientPathRequestPacket | confirmed | Calls ClientPathBasePacket constructor with sub-opcode 1, installs ClientPathRequestPacket::vftable, copies two globals, and zeroes its remaining payload field… |
| `008b7ea0` | ClientPathRequestPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |

### `net.packet.client_update`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091e1a0` | BaseClient::HandleClientUpdateStat 🌐 | confirmed | Handles the ClientUpdate stat packet: constructs ClientUpdatePacketUpdateStat (FUN_008ea870, sub-opcode 7), parses the body with FUN_0090a540 (two i16 header w… |

**ClientUpdatePacketActivateProfile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091bfe0` | ClientUpdatePacketActivateProfile::Ctor 🌐 | confirmed | Constructor: runs the BaseClientUpdatePacket family constructor FUN_0089d180 with sub-opcode 0x15 (21), installs ClientUpdatePacketActivateProfile::vftable, an… |
| `0091c060` | ClientUpdatePacketActivateProfile::~ClientUpdatePacketActiv… 🌐 | confirmed | Destructor body: installs ClientUpdatePacketActivateProfile::vftable, drains the SoeUtil::List<CharacterAttachmentData> at +0x14 through FUN_0070e670, then rew… |
| `0091c0f0` | ClientUpdatePacketActivateProfile::ScalarDeletingDtor 🌐 | confirmed | Scalar deleting destructor: calls the packet destructor 0091c060 and then frees this when bit 0 of the flag argument is set. |

**ClientUpdatePacketAddAbility** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b28b0` | ClientUpdatePacketAddAbility::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketCoinCount** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2840` | ClientUpdatePacketCoinCount::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketDoneSendingPreloadCharacters** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2940` | ClientUpdatePacketDoneSendingPreloadCharacters::ScalarDelet… | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketHitpoints** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2500` | ClientUpdatePacketHitpoints::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set. It has no … |

**ClientUpdatePacketImmediateActivationCheck** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2a20` | ClientUpdatePacketImmediateActivationCheck::ScalarDeletingD… | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketImmediateActivationFailed** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2a90` | ClientUpdatePacketImmediateActivationFailed::ScalarDeleting… | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketItemDelete** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2760` | ClientUpdatePacketItemDelete::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketItemUpdate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b25e0` | ClientUpdatePacketItemUpdate::ClientUpdatePacketItemUpdate 🌐 | confirmed | Calls the opcode-38 family constructor 0089d180(3) (which stores BasePacket id 0x26 and sub-opcode 3), installs ClientUpdatePacketItemUpdate::vftable, construc… |
| `008b2680` | ClientUpdatePacketItemUpdate::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketMana** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2570` | ClientUpdatePacketMana::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**ClientUpdatePacketSetGrandfatheredStatus** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b29b0` | ClientUpdatePacketSetGrandfatheredStatus::ScalarDeletingDes… | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketStorage** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b26f0` | ClientUpdatePacketStorage::ScalarDeletingDestructor | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketUpdateActionBarSlot** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2920` | ClientUpdatePacketUpdateActionBarSlot::ScalarDeletingDestru… | likely | Calls the packet destructor at 0089e860 (which destroys the packet member through 0101cd70 and then resets the BaseClientUpdatePacket and BasePacket vtables) a… |

**ClientUpdatePacketUpdateProfileRank** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b27d0` | ClientUpdatePacketUpdateProfileRank::ScalarDeletingDestruct… | likely | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached on… |

**ClientUpdatePacketUpdateStat** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ea870` | ClientUpdatePacketUpdateStatCtor 🌐 | confirmed | Constructs base client-update packet with sub-opcode 7, installs derived vtable and initializes CharacterStat list. |
| `008ea8e0` | ClientUpdatePacketUpdateStatDtor | confirmed | Destroys embedded CharacterStat list then restores base packet vtables. |
| `008ea950` | ClientUpdatePacketUpdateStat::ScalarDeletingDtor | confirmed | Calls packet destructor, frees object if flag bit zero is set, otherwise returns pointer. |

**(no class)** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008f8890` | ReadBodyVia_008d8210 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8900` | ReadBodyVia_008d8280 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8970` | ReadHeaderBlobVia_008d81b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f89f0` | ReadBodyVia_008d8350 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8a60` | ReadBodyPairVia_008e3e70 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8ad0` | ReadBodyVia_008d8490 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8b40` | ReadHeaderBlobVia_008d81b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8bc0` | ReadBodyPairVia_008e3ef0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8c30` | ReadBodyVia_008d8570 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8ca0` | ReadBodyVia_008d8670 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8d10` | ReadBodyVia_008d86e0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8d80` | ReadHeaderBlobVia_008d81b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8e00` | ReadHeaderBlobVia_008d81b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8e80` | ReadBodyPairVia_008e3fc0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8ef0` | ReadBodyVia_008d8870 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8f60` | ReadBodyPairVia_008e4010 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8fd0` | ReadBodyVia_008d8910 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f90b0` | ReadBodyVia_008d89c0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f9120` | ReadBodyPairVia_008e40b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f9190` | ReadBodyPairVia_008e40f0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f9200` | ReadBodyVia_008d8af0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f9270` | ReadBodyVia_008d8bc0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f92e0` | ReadBodyVia_008d8cb0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `00908eb0` | SpanReadBoolToThis0x31 🌐 | likely | Thiscall, ECX = output object, param_2 = span iterator. Calls 008fa4b0 on the iterator; if cursor+1 > end sets this+0x31 = 0, fail byte = 1 and cursor = end; o… |
| `009099d0` | ReadDwordFieldsAt9cAndB8 🌐 | likely | Thiscall, ECX = record (param_1), param_2 = bounded reader (cursor +8, end +0xc, fail byte +0x10). Reads four dwords into record+0x9c, +0xa0, +0xa4 and +0xa8, … |
| `0090a540` | ReadDwordPairThenFieldAt18 🌐 | likely | Thiscall, ECX = record, param_2 = reader. Calls 008d81b0(reader), reads two dwords into record+0x10 and +0x14 (overflow zeros both, sets the fail byte, moves t… |
| `0091b080` | ReadI16x2_Blob_List_I32x3 🌐 | likely | Reads a record from the reader at param_2 into param_1: two i16 into +4/+8 via FUN_008d81b0, a length-prefixed blob (pointer + length) into +0xc/+0x10 via FUN_… |

### `net.packet.combat`

**BaseCombatPacket** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29780` | BaseCombatPacket::BaseCombatPacket 🌐 | confirmed | Writes BasePacket::vftable then BaseCombatPacket::vftable at +0, family id 0x20 at +4 and a zero sub-opcode at +8. |
| `00a297e0` | BaseCombatPacket::BaseCombatPacketSubOpcode | confirmed | Calls the family base initializer and stores the caller-supplied sub-opcode at +8. |
| `00a29850` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a298a0` | BaseCombatPacket::ScalarDeletingDtor | confirmed | Resets both vtable entries and, when bit 0 of the flag argument is set, releases the object with _free. |
| `00a29910` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29960` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a299b0` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29a00` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29a50` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29aa0` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29af0` | BaseCombatPacket::~BaseCombatPacket | confirmed | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |

**CombatPacketAttackAttackerMissed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29e90` | CombatPacketAttackAttackerMissed::CombatPacketAttackAttacke… 🌐 | confirmed | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 5 and then stores CombatPacketAttackAttackerMissed::vftable at +0. |
| `00a29ef0` | CombatPacketAttackAttackerMissed::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**CombatPacketAttackProcessed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2a470` | CombatPacketAttackProcessed::CombatPacketAttackProcessed 🌐 | confirmed | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 7, stores CombatPacketAttackProcessed::vftable and zeroes the dwords at +0x28 and +0x30. |
| `00a2a4d0` | CombatPacketAttackProcessed::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**CombatPacketAttackTargetDamage** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29dc0` | CombatPacketAttackTargetDamage::CombatPacketAttackTargetDam… 🌐 | confirmed | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 4, stores CombatPacketAttackTargetDamage::vftable and zeroes the byte at +0x24. |
| `00a29e20` | CombatPacketAttackTargetDamage::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**CombatPacketAttackTargetDodged** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29fc0` | CombatPacketAttackTargetDodged::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**CombatPacketAutoAttackTarget** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29c20` | CombatPacketAutoAttackTarget::CombatPacketAutoAttackTarget 🌐 | confirmed | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 1 and then stores CombatPacketAutoAttackTarget::vftable at +0. |
| `00a29c80` | CombatPacketAutoAttackTarget::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**CombatPacketEnableBossDisplay** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2a030` | CombatPacketEnableBossDisplay::CombatPacketEnableBossDisplay 🌐 | confirmed | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 9 and then stores CombatPacketEnableBossDisplay::vftable at +0. |
| `00a2a090` | CombatPacketEnableBossDisplay::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**CombatPacketSingleAttackTarget** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29cf0` | CombatPacketSingleAttackTarget::CombatPacketSingleAttackTar… 🌐 | confirmed | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 3 and then stores CombatPacketSingleAttackTarget::vftable at +0. |
| `00a29d50` | CombatPacketSingleAttackTarget::ScalarDeletingDtor | likely | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw… |

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2c370` | DispatchBaseCombatPacket 🌐 | confirmed | Constructs a BaseCombatPacket on the stack, reads the family/sub-opcode header with ReadInt16Pair (00a2a740), and switches on the sub-opcode value: 4 to 00a2bf… |
| `00a2a740` | ReadInt16Pair | likely | Reads two little-endian 16-bit values out of the packet reader object (buffer base at +0, length at +4, cursor at +8, end at +0xc, overflow flag at +0x10) into… |
| `00a2a7a0` | ReadCombatPayloadTwoIds_Missed | likely | Calls ReadInt16Pair (00a2a740) for the family/sub-opcode shorts, then reads two 8-byte id pairs into +0x10 and +0x18, zeroing each field and setting the overfl… |
| `00a2a820` | ReadCombatPayloadTwoIds_Dodged | likely | Calls ReadInt16Pair (00a2a740) for the family/sub-opcode shorts, then reads two 8-byte id pairs into +0x10 and +0x18, zeroing each field and setting the overfl… |
| `00a2a8a0` | ReadCombatPayloadIdAndBool_BossDisplay | likely | Calls ReadInt16Pair (00a2a740), reads one 8-byte id pair into +0x10, then reads one byte into +0x18 as a boolean; each truncated read zeroes the field, sets th… |
| `00a2a910` | ReadCombatPayloadProcessed | likely | Calls ReadInt16Pair (00a2a740), then consumes three 8-byte pairs: the first and second both write +0x10/+0x14 (the second overwrites the first), and the third … |
| `00a2aab0` | ReadCombatPayloadDamage | likely | Calls ReadInt16Pair (00a2a740), reads two 8-byte id pairs into +0x10 and +0x18, one dword into +0x20 and one byte into +0x24, zeroing each field and setting th… |
| `00a2b360` | HandleCombatPacketAttackAttackerMissed | likely | Constructs BaseCombatPacket sub-opcode 5 (CombatPacketAttackAttackerMissed), reads the payload with 00a2a7a0, looks up both actors by id, and attaches the lite… |
| `00a2b910` | HandleCombatPacketEnableBossDisplay | likely | Constructs BaseCombatPacket sub-opcode 9 (CombatPacketEnableBossDisplay), reads the payload with 00a2a8a0, looks up the actor by id and sets or clears bit 7 of… |
| `00a2c2d0` | WritePacketFamilyAndSubOpcode 🌐 | likely | Writes the dword at this+4 and the dword at this+8 as 16-bit values into the outgoing message object through FUN_007cc400, each copy bounded by min(2, remainin… |
| `00a2c7d0` | ClientAutoAttackTarget 🌐 | likely | Turns the player toward the target (00a2a160), constructs CombatPacketAutoAttackTarget (00a29c20), reads the target id pair through the actor vtable slot 0x54-… |
| `00a2c880` | ClientSingleAttackTarget 🌐 | likely | Turns the player toward the target (00a2a160), constructs CombatPacketSingleAttackTarget (00a29cf0), reads the target id pair through the actor vtable slot 0x5… |

### `net.packet.command`

**AccountCommand** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01407cf0` | AccountCommand::Read | confirmed | Virtual slot 9 (+0x24) of AccountCommand::vftable. Calls vtable slot 4 (+0x10) with the stream and its own argument (header read). If that returns true, reads … |
| `01407d30` | AccountCommand::Write | confirmed | Virtual slot 10 (+0x28) of AccountCommand::vftable. Calls vtable slot 2 (+0x8) with the stream, writes the text markers 'Starting AccountCommand', 'AccountID: … |

**AccountCommand_RecordSinglePlayerGame** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01407ff0` | AccountCommand_RecordSinglePlayerGame::Read | confirmed | Virtual slot 9 (+0x24) of AccountCommand_RecordSinglePlayerGame::vftable. Calls vtable slot 4 (+0x10) with the stream (header read); calls 01407cf0 (AccountCom… |
| `014080c0` | AccountCommand_RecordSinglePlayerGame::Write | confirmed | Virtual slot 10 (+0x28) of AccountCommand_RecordSinglePlayerGame::vftable. Calls vtable slot 2 (+0x8) with the stream, calls 01407d30 (AccountCommand::Write), … |

**AccountCommand_SetChallengeStatus** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014071f0` | AccountCommand_SetChallengeStatus::Write | confirmed | Virtual slot 10 (+0x28). Calls vtable slot 2 (+0x8) on this with the serial buffer, calls 01407d30(buffer), writes the integer at this+0x20 (013c23f0), then fo… |
| `014073b0` | AccountCommand_SetChallengeStatus::Dump | confirmed | Virtual slot 19 (+0x4c). Appends 'Set Challenge Status Command' plus a newline (0x1d characters) through 01241650, calls 01407e70 on its output argument, then … |
| `01407930` | AccountCommand_SetChallengeStatus::Read | confirmed | Virtual slot 9 (+0x24) of AccountCommand_SetChallengeStatus::vftable. Calls vtable slot 4 (+0x10) with the stream and a local string (header read; returns fals… |

**AccountCommand_SetPreferences** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01408990` | AccountCommand_SetPreferences::Write | confirmed | Virtual slot 10 (+0x28). Calls vtable slot 2 (+0x8) with the stream, writes 'Starting AccountCommand_SetPreferences' through 013c26e0 (a no-op returning 1), ca… |
| `01408aa0` | AccountCommand_SetPreferences::Read | confirmed | Virtual slot 9 (+0x24). Calls vtable slot 4 (+0x10) with the stream (header read; returns false on failure), calls 01407cf0 (AccountCommand::Read), then reads … |

**AdminCommandPacketShipCombatCommand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a57a30` | AdminCommandPacketShipCombatCommand::Ctor 🌐 | confirmed | Calls BaseCommandPacket construction with command identifier 0x491 and the ShipCombat string, installs the concrete vtable, initializes numeric fields, and con… |

**BaseCommandPacket** (22)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2820` | BaseCommandPacket::BaseCommandPacket 🌐 | confirmed | Command family constructor taking sub-opcode and an extra word: installs BasePacket::vftable, stores 0x1a (26) in the packet id slot param_1[1], installs BaseC… |
| `008a2890` | BaseCommandPacket::~BaseCommandPacket | confirmed | Destructor body: restores BaseCommandPacket::vftable then BasePacket::vftable; 133 EH unwind funclets call it, making it the shared cleanup body for command pa… |
| `009f58f0` | BaseCommandPacket::DtorBody | confirmed | Destructor body without free: stores BaseCommandPacket::vftable then BasePacket::vftable into *this and returns. |
| `009f5990` | BaseCommandPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor used as PacketDialogResponse::vftable slot [0]: stores BaseCommandPacket::vftable then BasePacket::vftable into *this, sets the EH s… |
| `00a0a4a0` | BaseCommandPacket::~BaseCommandPacket | confirmed | Calls FUN_0070a790 with receiver this+0x1c, then rewrites the vtable to BaseCommandPacket::vftable and BasePacket::vftable. |
| `00a36c00` | BaseCommandPacket::~BaseCommandPacket | confirmed | Bytes: prologue installs an MSVC SEH frame (push -1, fs:[0] save, /GS cookie xor), then stores BaseCommandPacket::vftable into *this and afterwards BasePacket:… |
| `00a57ad0` | BaseCommandPacket_DtorWithString | confirmed | Releases the embedded shared string buffer and resets the object through BaseCommandPacket then BasePacket vtables. |
| `00a98620` | BaseCommandPacket::BaseCommandPacketInit 🌐 | confirmed | Stores BasePacket::vftable, sets the word at +4 to 0x1a, stores BaseCommandPacket::vftable, and zeroes the words at +8 and +12. No callees. |
| `00c09fd0` | BaseCommandPacket_DtorBody | confirmed | Base-class destructor body: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable; reached from the unwind funclet 015f4600 of the quest-a… |
| `00c1be10` | BaseCommandPacket::EmbeddedMemberCleanupA | confirmed | Runs the embedded-member cleanup helper under an exception frame, restores BaseCommandPacket and BasePacket vptrs, restores the exception list and returns. |
| `00c1be70` | BaseCommandPacket::EmbeddedMemberCleanupB | confirmed | Runs the embedded-member cleanup helper under an exception frame, restores BaseCommandPacket and BasePacket vptrs, restores the exception list and returns. |
| `008a28e0` | BaseCommandPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `00a103b0` | BaseCommandPacket_Dtor | likely | Destructor body: stores BaseCommandPacket::vftable then BasePacket::vftable; no fields released. |
| `00a12250` | BaseCommandPacket_ScalarDeletingDtor | likely | Scalar deleting destructor: stores BaseCommandPacket::vftable then BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |
| `00a4f510` | BaseCommandPacket::BaseCommandPacket 🌐 | likely | Constructor of the generic command packet: stores BasePacket::vftable, writes opcode 0x1a (26) into field +4, stores BaseCommandPacket::vftable and stores the … |
| `00a988e0` | BaseCommandPacket::BaseDtorChain | likely | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. |
| `00c0a800` | BaseCommandPacket_DtorBody_00c0a800 | likely | Second copy of the BaseCommandPacket destructor body in this translation unit: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable; reac… |
| `00c14980` | BaseCommandPacketDtorNoMembers_00c14980 | likely | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns (no member releases). |
| `00c14a30` | BaseCommandPacketDtorNoMembers_00c14a30 | likely | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c14ae0` | BaseCommandPacketDtorNoMembers_00c14ae0 | likely | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c14ba0` | BaseCommandPacketDtorNoMembers_00c14ba0 | likely | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c14cc0` | BaseCommandPacketDtorNoMembers_00c14cc0 | likely | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |

**ClearInteractionMerchantSetId** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0a9c0` | ClearInteractionMerchantSetId_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of ClearInteractionMerchantSetId::vftable: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable and ca… |

**ClientCommandProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa2560` | ClientCommandProcessor::DispatchCommandPacket 🌐 | likely | Calls FUN_00a9cb50 to read a command header, then dispatches cases 2, 3, 4, 7, 9, 0xc, 0x10, 0x12, 0x16, 0x19, 0x1c, 0x1d, 0x1f, 0x23, 0x26 and 0x27. In case 0… |

**CommandPacketActivateProfileFailed** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2ca0` | CommandPacketActivateProfileFailed::CommandPacketActivatePr… 🌐 | confirmed | Calls FUN_008a2820(0x1a,0), installs CommandPacketActivateProfileFailed::vftable, and zeroes payload dwords param_1[4] and param_1[5]; caller 0092f260 fills bo… |
| `008a2d10` | CommandPacketActivateProfileFailed::~CommandPacketActivateP… | likely | Destructor body: restores the derived vtable, then BaseCommandPacket::vftable and BasePacket::vftable. |
| `008a2d70` | CommandPacketActivateProfileFailed::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the ActivateProfileFailed, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is se… |

**CommandPacketAddFriendRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008eb5c0` | CommandPacketAddFriendRequestCtor 🌐 | confirmed | Constructs command packet with sub-opcode 14, installs derived vtable, initializes string member and copies argument. |
| `008eb650` | CommandPacketAddFriendRequest::ScalarDeletingDtor | confirmed | Calls packet destructor, frees object if flag bit zero is set, otherwise returns original pointer. |
| `008d14c0` | CommandPacketAddFriendRequest::~CommandPacketAddFriendReque… | likely | Destructor body of a command packet: rewrites the embedded SoeUtil::IString<char> member at +0x10 (param_1[4]) to its vtable, LOCK-decrements the shared refcou… |

**CommandPacketChatChannelOff** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1dd60` | CommandPacketChatChannelOff::Ctor 🌐 | confirmed | Calls the command base constructor with sub-opcode 0x21, installs its packet vptr, initializes an IString member and assigns the supplied channel string. |
| `00c1ddf0` | CommandPacketChatChannelOff::ScalarDeletingDtor | confirmed | Calls embedded string and packet-base cleanup, frees the object when the deleting flag is set, and otherwise returns it. |

**CommandPacketChatChannelOn** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1dcb0` | CommandPacketChatChannelOn::Ctor 🌐 | confirmed | Calls the command base constructor with sub-opcode 0x20, installs its packet vptr, initializes an IString member and assigns the supplied channel string. |
| `00c1dd40` | CommandPacketChatChannelOn::ScalarDeletingDtor | confirmed | Calls embedded string and packet-base cleanup, frees the object when the deleting flag is set, and otherwise returns it. |

**CommandPacketClosedMinigameEndScreen** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2eb0` | CommandPacketClosedMinigameEndScreen::CommandPacketClosedMi… 🌐 | confirmed | Constructor: calls FUN_008a2820(0x2a,0) so the packet is command opcode 26 with sub-opcode 42, then installs CommandPacketClosedMinigameEndScreen::vftable insi… |
| `008b6d30` | CommandPacketClosedMinigameEndScreen::ScalarDeletingDestruc… | likely | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct calle… |

**CommandPacketConfirmFriendRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9c270` | CommandPacketConfirmFriendRequest::~CommandPacketConfirmFri… | confirmed | Stores CommandPacketConfirmFriendRequest::vftable, then cleans up the member object at this+0x18 through 0070a790 (the bytes load lea ecx,[esi+0x18] before the… |
| `00a9de90` | CommandPacketConfirmFriendRequest::CommandPacketConfirmFrie… 🌐 | confirmed | Constructor of the client-side confirm-friend-request command packet: calls the command-family constructor FUN_008a2820 with sub-opcode 0x10 (16) and 0, instal… |
| `00a9c2e0` | CommandPacketConfirmFriendRequest::ScalarDeletingDtor | likely | Calls 00a9c270 (the ConfirmFriendRequest destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |

**CommandPacketConfirmFriendResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0a580` | CommandPacketConfirmFriendResponse::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the base command teardown 00a0a4a0 and frees this when the delete flag is set. |
| `00a0a500` | CommandPacketConfirmFriendResponse::CommandPacketConfirmFri… 🌐 | likely | Constructor: runs the shared command base constructor with sub-opcode 0x11 (17), installs the CommandPacketConfirmFriendResponse vtable, stores three arguments… |

**CommandPacketEndDialog** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f5760` | CommandPacketEndDialog::CommandPacketEndDialog | confirmed | Constructor: stores 0xffffffff into the EH state slot, installs a SEH frame, calls FUN_008a2820(4,0) (the BaseCommandPacket family constructor: sub-opcode 4, e… |
| `009f57c0` | CommandPacketEndDialog::DtorBody | confirmed | Destructor body without free: stores CommandPacketEndDialog::vftable, then BaseCommandPacket::vftable, then BasePacket::vftable into *this in that order and re… |
| `009f5820` | CommandPacketEndDialog::ScalarDeletingDtor | confirmed | Scalar deleting destructor, vtable slot [0] of CommandPacketEndDialog::vftable: stores CommandPacketEndDialog, BaseCommandPacket and BasePacket vtables in orde… |

**CommandPacketFriendsPositionRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c149d0` | CommandPacketFriendsPositionRequest::CommandPacketFriendsPo… 🌐 | confirmed | Constructor: calls BaseCommandPacket::BaseCommandPacket (FUN_008a2820) with sub-opcode 0x15 (21) and extra word 0, then stores CommandPacketFriendsPositionRequ… |
| `00c1a9b0` | CommandPacketFriendsPositionRequest::ScalarDeletingDtor | confirmed | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |

**CommandPacketIgnoreRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1dc10` | CommandPacketIgnoreRequest::Ctor 🌐 | confirmed | Calls the command base constructor with sub-opcode 0x1e, installs CommandPacketIgnoreRequest::vftable, copies three strings and three dwords from the supplied … |
| `00c1dc90` | CommandPacketIgnoreRequest::ScalarDeletingDtor | confirmed | Calls the packet base cleanup, frees the object when the deleting flag is set, and returns the original pointer otherwise. |

**CommandPacketInteractRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2c50` | CommandPacketInteractRequest::~CommandPacketInteractRequest | confirmed | Destructor body: restores BaseCommandPacket::vftable then BasePacket::vftable; callers 0092efe0 and 00934f90 construct CommandPacketInteractRequest (via FUN_00… |
| `008b6c40` | CommandPacketInteractRequest::ScalarDeletingDestructor | likely | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct calle… |

**CommandPacketInteractionList** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9c1c0` | CommandPacketInteractionList::CommandPacketInteractionList 🌐 | confirmed | Calls 008a2820 with sub-opcode 9 and zero extra word, calls 00a9c090 on a member, stores CommandPacketInteractionList::vftable and zeroes the byte at +0x248. |
| `00a9b7e0` | CommandPacketInteractionList::~CommandPacketInteractionList | likely | Calls 00a9b760 on this (that helper destroys the embedded InteractionData array through 00a9af50 and releases the IString<char> member at +0xc), then stores Ba… |
| `00a9c230` | CommandPacketInteractionList::ScalarDeletingDtor | likely | Calls 00a9b7e0 (the member-cleanup destructor chain) on this, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object… |

**CommandPacketInteractionSelect** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a36f80` | CommandPacketInteractionSelect::CommandPacketInteractionSel… | confirmed | Ctor with SEH frame: calls 008a2820(10, 0), the BaseCommandPacket family constructor (accepted label), stores CommandPacketInteractionSelect::vftable into *thi… |
| `00a37000` | CommandPacketInteractionSelect::~CommandPacketInteractionSe… | confirmed | Virtual slot [0] of CommandPacketInteractionSelect::vftable (binary vtable read). Body shape of a scalar deleting destructor: sets the vtable to BaseCommandPac… |

**CommandPacketMoveAndInteract** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a99cd0` | CommandPacketMoveAndInteract::CommandPacketMoveAndInteract 🌐 | confirmed | Calls 008a2820 with sub-opcode 0x16 (22) and zero extra word, then stores CommandPacketMoveAndInteract::vftable and returns this. |
| `00a99d30` | CommandPacketMoveAndInteract::ScalarDeletingDtor | confirmed | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; … |
| `00a98750` | CommandPacketMoveAndInteract::~CommandPacketMoveAndInteract | likely | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the family-26 inbound dispatcher 00a… |

**CommandPacketPlayDialogEffect** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9ad20` | CommandPacketPlayDialogEffect::CommandPacketPlayDialogEffect 🌐 | confirmed | Calls 008a2820 with sub-opcode 0x1c (28) and zero extra word, stores CommandPacketPlayDialogEffect::vftable and zeroes the words at +0x10, +0x14 and +0x18. |
| `00a9ad90` | CommandPacketPlayDialogEffect::ScalarDeletingDtor | confirmed | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; … |

**CommandPacketPlaySoundAtLocation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9b650` | CommandPacketPlaySoundAtLocation::CommandPacketPlaySoundAtL… 🌐 | confirmed | Calls 008a2820 with sub-opcode 7 and zero extra word, stores CommandPacketPlaySoundAtLocation::vftable, stores an empty SoeUtil::IString<char> at +0x10 (vtable… |
| `00a9b6d0` | CommandPacketPlaySoundAtLocation::~CommandPacketPlaySoundAt… | likely | Stores the IString<char> vtable at +0x10, releases the string buffer at +0x14 when the count at +0x1c is positive (interlocked decrement and release through vt… |
| `00a9c1a0` | CommandPacketPlaySoundAtLocation::ScalarDeletingDtor | likely | Calls 00a9b6d0 (the PlaySoundAtLocation destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |

**CommandPacketPlaySoundIdOnTarget** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98680` | CommandPacketPlaySoundIdOnTarget::CommandPacketPlaySoundIdO… 🌐 | confirmed | Calls 008a2820 with sub-opcode 0x27 and a zero extra word, then stores CommandPacketPlaySoundIdOnTarget::vftable, zeroes +0x10, stores Target::vftable at +0x14… |
| `00a99cb0` | CommandPacketPlaySoundIdOnTarget::ScalarDeletingDtor | confirmed | Calls 00a986f0 (the BaseCommandPacket destructor body) on this, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the obje… |
| `00a986f0` | CommandPacketPlaySoundIdOnTarget::~CommandPacketPlaySoundId… | likely | Calls the Target sub-object destructor 0101b370 (whose body writes Target::vftable and releases the pointer at Target+4) on the embedded Target member, then st… |

**CommandPacketQuestAbandon** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0a2e0` | CommandPacketQuestAbandon_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of CommandPacketQuestAbandon::vftable: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable and calls … |

**CommandPacketRecipeStart** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c14b30` | CommandPacketRecipeStart::CommandPacketRecipeStart 🌐 | confirmed | Constructor taking one word: calls BaseCommandPacket::BaseCommandPacket (FUN_008a2820) with sub-opcode 0x18 (24) and extra word 0, stores CommandPacketRecipeSt… |
| `00c1aa90` | CommandPacketRecipeStart::ScalarDeletingDtor | confirmed | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |

**CommandPacketRemoveFriendRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1db80` | CommandPacketRemoveFriendRequest::Ctor 🌐 | confirmed | Calls the command base constructor with sub-opcode 0x0f, installs CommandPacketRemoveFriendRequest::vftable, and copies three strings and three dwords from the… |
| `00c1dbf0` | CommandPacketRemoveFriendRequest::ScalarDeletingDtor | confirmed | Calls the packet base cleanup, frees the object when the deleting flag is set, and returns the original pointer otherwise. |

**CommandPacketRequestPlayIntroEncounter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1a940` | CommandPacketRequestPlayIntroEncounter::ScalarDeletingDtor | confirmed | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |

**CommandPacketRequestPlayerPositions** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c14bf0` | CommandPacketRequestPlayerPositions::~CommandPacketRequestP… | confirmed | Destructor body: stores CommandPacketRequestPlayerPositions::vftable, then BaseCommandPacket::vftable and finally BasePacket::vftable at *this. |
| `00c14c50` | CommandPacketRequestPlayerPositions::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of CommandPacketRequestPlayerPositions): resets the three vtables (own, BaseCommandPacket, BasePacket) and frees the … |

**CommandPacketRequestPlayerPositionsReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa1880` | CommandPacketRequestPlayerPositionsReply::CommandPacketRequ… 🌐 | confirmed | Constructor: calls the command-family constructor FUN_008a2820 with sub-opcode 0x23 (35) and 0, stores CommandPacketRequestPlayerPositionsReply::vftable at +0 … |
| `00aa18f0` | CommandPacketRequestPlayerPositionsReply::~CommandPacketReq… | confirmed | Destructor body: rewrites the class vtable, restores the SoeUtil::List vtable at +0x1c and drains the player-position list by repeatedly calling FUN_00a85d60 o… |
| `00aa1980` | CommandPacketRequestPlayerPositionsReply::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00aa18f0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |

**CommandPacketRequestRewardPreviewUpdate** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2de0` | CommandPacketRequestRewardPreviewUpdate::~CommandPacketRequ… | likely | Destructor body: restores CommandPacketRequestRewardPreviewUpdate::vftable, BaseCommandPacket::vftable and BasePacket::vftable; its only non-self caller is EH … |
| `008a2e40` | CommandPacketRequestRewardPreviewUpdate::ScalarDeletingDest… | likely | Scalar deleting destructor: restores the reward-preview, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `008b6cb0` | CommandPacketRequestRewardPreviewUpdate::CommandPacketReque… 🌐 | likely | Calls the command family constructor 008a2820(0x25, 0), installs CommandPacketRequestRewardPreviewUpdate::vftable, stores param_2 into +0x10 and the global pai… |

**CommandPacketRequestRewardPreviewUpdateReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98930` | CommandPacketRequestRewardPreviewUpdateReply::~CommandPacke… | confirmed | Stores CommandPacketRequestRewardPreviewUpdateReply::vftable, calls 0103d010 on the embedded reward-bundle member (0103d010's body restores RewardBundleBase::v… |
| `00aa1d00` | CommandPacketRequestRewardPreviewUpdateReply::CommandPacket… 🌐 | confirmed | Constructor: calls the command-family constructor FUN_008a2820 with sub-opcode 0x26 (38) and 0, stores CommandPacketRequestRewardPreviewUpdateReply::vftable at… |
| `00a989a0` | CommandPacketRequestRewardPreviewUpdateReply::ScalarDeletin… | likely | Calls 00a98930 on this, then if the low bit of the flag byte is set calls _free on the object; returns the object pointer. |

**CommandPacketRequestStorageCheck** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1aa20` | CommandPacketRequestStorageCheck::ScalarDeletingDtor | confirmed | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |

**CommandPacketSelectPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a29a0` | CommandPacketSelectPlayer::~CommandPacketSelectPlayer | confirmed | Destructor body: restores BaseCommandPacket::vftable then BasePacket::vftable; the two callers that matter are 00934870 and 0093bff0, which both construct Comm… |
| `008b6af0` | CommandPacketSelectPlayer::CommandPacketSelectPlayer 🌐 | likely | Calls the command family constructor 008a2820(0x13, 0) and installs CommandPacketSelectPlayer::vftable; no field is initialised. |
| `008b6b50` | CommandPacketSelectPlayer::ScalarDeletingDestructor | likely | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct calle… |

**CommandPacketSetActiveVehicleGuid** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a10340` | CommandPacketSetActiveVehicleGuid_Ctor 🌐 | likely | Constructor: calls FUN_008a2820(0x1f,0) to set sub-opcode 31 on a command packet, installs CommandPacketSetActiveVehicleGuid::vftable, and calls FUN_0101cd40 w… |

**CommandPacketSetChatBubbleColor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b6a20` | CommandPacketSetChatBubbleColor::CommandPacketSetChatBubble… 🌐 | confirmed | Calls the command family constructor 008a2820(0x12, 0) and installs CommandPacketSetChatBubbleColor::vftable; no field is initialised. |
| `008b6a80` | CommandPacketSetChatBubbleColor::ScalarDeletingDestructor | likely | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct calle… |

**CommandPacketSetProfile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a29f0` | CommandPacketSetProfile::CommandPacketSetProfile 🌐 | confirmed | Constructor: calls FUN_008a2820(0xd,0) so the packet is command opcode 26 with sub-opcode 13, then installs CommandPacketSetProfile::vftable. |
| `008a2a50` | CommandPacketSetProfile::~CommandPacketSetProfile | likely | Destructor body: restores CommandPacketSetProfile::vftable, then BaseCommandPacket::vftable, then BasePacket::vftable; its only non-self caller is EH funclet 0… |
| `008a2ab0` | CommandPacketSetProfile::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the SetProfile, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |

**CommandPacketSetProfileByItemDefinitionId** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2b20` | CommandPacketSetProfileByItemDefinitionId::CommandPacketSet… 🌐 | confirmed | Constructor: calls FUN_008a2820(0x24,0) so the packet is command opcode 26 with sub-opcode 36, then installs CommandPacketSetProfileByItemDefinitionId::vftable… |
| `008a2b80` | CommandPacketSetProfileByItemDefinitionId::~CommandPacketSe… | likely | Destructor body: restores the derived vtable, then BaseCommandPacket::vftable and BasePacket::vftable. |
| `008a2be0` | CommandPacketSetProfileByItemDefinitionId::ScalarDeletingDe… | likely | Scalar deleting destructor: restores the SetProfileByItemDefinitionId, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit … |

**CommandPacketShowDialog** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a99bd0` | CommandPacketShowDialog::CommandPacketShowDialog 🌐 | confirmed | Calls 008a2820 with sub-opcode 3 and zero extra word, stores CommandPacketShowDialog::vftable, then fills the fields from +0x10 to +0x8c with default constants… |
| `00a9c110` | CommandPacketShowDialog::~CommandPacketShowDialog | confirmed | Stores CommandPacketShowDialog::vftable, then calls 00a9bc20 on the intrusive-list member at this+0x20 (the bytes load lea ecx,[esi+0x20] before the call) to u… |
| `00a9c180` | CommandPacketShowDialog::ScalarDeletingDtor | likely | Calls 00a9c110 (the ShowDialog destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |

**CommandPacketShowRecipeWindow** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a987a0` | CommandPacketShowRecipeWindow::CommandPacketShowRecipeWindow 🌐 | confirmed | Calls 008a2820 with sub-opcode 0x19 and zero extra word, then stores CommandPacketShowRecipeWindow::vftable and zeroes the words at +0x10 and +0x14. |
| `00a98810` | CommandPacketShowRecipeWindow::~CommandPacketShowRecipeWind… | confirmed | Stores CommandPacketShowRecipeWindow::vftable, then BaseCommandPacket::vftable, then BasePacket::vftable into the object. Called from EH unwind funclet 015c138… |
| `00a98870` | CommandPacketShowRecipeWindow::ScalarDeletingDtor | confirmed | Stores the CommandPacketShowRecipeWindow, BaseCommandPacket and BasePacket vtables in order, then if the low bit of the flag byte is set calls _free on the obj… |

**CommandPacketStartFlashGame** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9b850` | CommandPacketStartFlashGame::CommandPacketStartFlashGame 🌐 | confirmed | Calls 008a2820 with sub-opcode 0xc (12) and zero extra word, stores CommandPacketStartFlashGame::vftable, sets two empty SoeUtil::IString<char> members at +0x1… |
| `00a9b8d0` | CommandPacketStartFlashGame::~CommandPacketStartFlashGame | likely | Stores the IString<char> vtable at +0x20, releases the string at +0x24 when its count at +0x2c is positive, stores the IString<char> vtable at +0x10, releases … |
| `00a9c250` | CommandPacketStartFlashGame::ScalarDeletingDtor | likely | Calls 00a9b8d0 (the StartFlashGame destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |

**CommandPacketWhoReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a99b60` | CommandPacketWhoReply::CommandPacketWhoReply 🌐 | confirmed | Calls 008a2820 with sub-opcode 2 and zero extra word, stores CommandPacketWhoReply::vftable, and zeroes the words at +0x10, +0x14 and +0x18. |
| `00aa05f0` | CommandPacketWhoReply::~CommandPacketWhoReply | confirmed | Destructor body: rewrites the object vtable to CommandPacketWhoReply::vftable, then frees the player list through FUN_00a9f4d0, and finally rewrites the object… |
| `00aa0660` | CommandPacketWhoReply::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00aa05f0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |

**PacketDialogResponse** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f5890` | PacketDialogResponse::PacketDialogResponse | confirmed | Constructor: calls the BaseCommandPacket family constructor FUN_008a2820(6,0), installs PacketDialogResponse::vftable into *this, then stores -1 into the dword… |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00938e10` | PlayClient::ChangeProfile 🌐 | confirmed | Constructs and submits CommandPacketSetProfile and CommandPacketRequestRewardPreviewUpdate, then raises the ActivityPortal:OnProfileChange event. |

**(no class)** (29)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa0920` | HandleCommandCase25 🌐 | confirmed | Handler for command sub-opcode 25: constructs the packet object with FUN_00a987a0 (whose class vtable is CommandPacketShowRecipeWindow), parses the body with F… |
| `00aa0a10` | HandleCommandCase12 🌐 | confirmed | Handler for command sub-opcode 12: constructs the packet with FUN_00a9b850, parses the body with FUN_00a9d180, then calls FUN_009bd650 with three parsed slots … |
| `00aa1ff0` | CommandPacketWhoReply_Handle 🌐 | confirmed | Handler for command sub-opcode 2: constructs CommandPacketWhoReply with FUN_00a99b60, reads the two header words, builds the player list with FUN_00aa0380, req… |
| `00a1e580` | SendSetActiveVehicleGuidCommand 🌐 | likely | Builds a CommandPacketSetActiveVehicleGuid on the stack via its constructor 00a10340, stores the two dword arguments into the packet fields at +0x10 and +0x14,… |
| `00a9b760` | DtorMembersInteractionDataArrayAndString | likely | Calls 00a9af50 (the Array<InteractionData,16,1> destructor body) on this, stores the IString<char> vtable at +0xc, and when the count at +0x18 is positive rele… |
| `00a9d3c0` | BodyRead_i16x2_i32x2_i32 🌐 | likely | Reads a packet body from a stream object (cursor at +8, end at +0xc, error byte at +0x10): first delegates to FUN_00a9cb50 to store two 16-bit header words at … |
| `00a9d430` | BodyRead_i16x2_i32x3_i32x2 🌐 | likely | Reads a body from the stream object: delegates the two 16-bit header words to FUN_00a9cb50, then copies three 32-bit values into destination +0x10, +0x14 and +… |
| `00a9d840` | BodyRead_i32x2_blob_bool3_str 🌐 | likely | Reads two 32-bit values into destination +0/+4, then an int32 length and the following bytes into the string member at destination +8 (the byte assign runs thr… |
| `00a9ef10` | BodyRead_Command_SubOp3_Payload 🌐 | likely | Large body reader: header pair through FUN_00a9cb50, two 32-bit values into +0x10/+0x14, 8 bytes into +0x18/+0x1c, boolean at +0x2c, float at +0x30 (a NaN also… |
| `00a9f5e0` | ParseCommandBody_Case22 🌐 | likely | Body parse entry of the four-argument idiom (destination, body pointer, length, allow-trailing): guards destination and pointer, sets up the local stream recor… |
| `00a9f680` | ParseCommandBody_Case31 🌐 | likely | Body parse entry of the four-argument idiom: sets up the local stream record over (body pointer, length), calls the field reader FUN_00a9cc10 with the destinat… |
| `00aa0210` | BodyRead_i16x2_i32x2_block_bool 🌐 | likely | Reads the header pair through FUN_00a9cb50, a 32-bit pair into destination +0x10/+0x14, then a larger block through FUN_008e7290 (a reader that itself uses the… |
| `00aa0380` | ReadIntrusiveListOf3cByteEntries 🌐 | likely | Empties the list at param_2 with FUN_00a9f4d0, reads a 32-bit count, then per element allocates 0x3c bytes with FUN_00d833a6, constructs the node with FUN_00a9… |
| `00aa04d0` | ReadIntrusiveListOfRadialEntries 🌐 | likely | Reads a 32-bit count; when it is negative or larger than the remaining bytes it sets the error byte and moves the cursor to the end, otherwise it grows the arr… |
| `00aa0870` | CommandPacketEndDialog_Handle 🌐 | likely | Handles the end-dialog command: inside an SEH frame it constructs a CommandPacketEndDialog with FUN_009f5760, reads the two header words with FUN_00a9cb50, req… |
| `00aa0b00` | HandleCommandCase7 🌐 | likely | Handler for command sub-opcode 7: constructs the packet with FUN_00a9b650, parses the body into four local slots with FUN_00a9d250, and when the whole body was… |
| `00aa0be0` | HandleCommandCase39 🌐 | likely | Handler for command sub-opcode 39: constructs the packet with FUN_00a98680, then requires the body to be fully consumed by FUN_00a9d360 (which validates a run … |
| `00aa10b0` | ReadIntrusiveListOfPositionEntries 🌐 | likely | Drains the list reached through param_2+4 with FUN_00a85d60, reads a 32-bit count, then per element constructs a node with FUN_00a85cd0 and fills it through FU… |
| `00aa12e0` | CommandPacketShowDialog_Handle 🌐 | likely | Handler for command sub-opcode 3: constructs CommandPacketShowDialog through FUN_00a99bd0 (the constructor the NPC-dialogue note pins), parses the dialog paylo… |
| `00aa13b0` | ParseCommandBody_SubOp16 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00aa0210 (header pair, 32-bit pair, block an… |
| `00aa1420` | ParseCommandBody_SubOp38 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), reads the two header words through FUN_00a9cb50 and th… |
| `00aa1550` | BodyRead_i16x2_i32x2_bool2_list 🌐 | likely | Reads the header pair through FUN_00a9cb50, a 32-bit pair into destination +0x10/+0x14, two boolean bytes at +0x18 and +0x19, and then builds an entry list at … |
| `00aa1600` | BodyRead_i32x2_bool_list_string_bool 🌐 | likely | Reads two 32-bit values into destination +0/+4, one boolean byte at +8, then a count-prefixed entry list at +0x20 through FUN_00aa04d0, then an int32 length wi… |
| `00aa21b0` | ParseCommandBody_SubOp35 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00aa1550 (header pair, 32-bit pair, two flag… |
| `00aa2220` | BodyRead_Command9_Payload 🌐 | likely | Reads the two header words through FUN_00a9cb50, then the rest of the object through FUN_00aa1600 (two 32-bit values, a flag, a count-prefixed entry list at +0… |
| `00aa2360` | HandleCommandCase9 🌐 | likely | Handler for command sub-opcode 9: constructs the packet with FUN_00a9c1c0, parses the body into a 592-byte (0x250) local object with FUN_00aa2220 requiring ful… |
| `00c0a350` | CommandPacketQuestAbandon_SerializeBody | likely | Packet body writer: calls FUN_0090ac70(buffer) for the command-packet preamble, then appends the 32-bit quest id at +0x10 through the bounded copy FUN_007cc400. |
| `00c0a3d0` | SendCommandPacketQuestAbandon_Gateway | likely | ClientGateway send template for the quest-abandon command: serialises the packet through FUN_00c0a350 into a local buffer, finalises it, constructs ClientGatew… |
| `00c0ad30` | SendClearMerchantInteraction_Gateway | likely | ClientGateway send template for the clear-merchant command: serialises the source object through FUN_0090ac70 into a local buffer, finalises it, constructs Cli… |

### `net.packet.encounter`

**BaseEncounterPacket** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2f60` | BaseEncounterPacket::BaseEncounterPacket 🌐 | confirmed | Encounter family constructor with a fixed zero sub-opcode: installs BasePacket::vftable, stores 0x29 (41) in the packet id slot param_1[1], installs BaseEncoun… |
| `008a2fd0` | BaseEncounterPacket::BaseEncounterPacketSub 🌐 | confirmed | Encounter family constructor taking the sub-opcode: installs BasePacket::vftable, stores 0x29 (41) in slot param_1[1], installs BaseEncounterPacket::vftable, s… |
| `008a3040` | BaseEncounterPacket::~BaseEncounterPacket | confirmed | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; reached from a block of EH unwind funclets around 0157d2xx, which is why it ac… |
| `00a862a0` | BaseEncounterPacketDtorBody | confirmed | Destructor body of BaseEncounterPacket: chains the two vtable stores that MSVC emits for a destructor of a class deriving from BasePacket (own vtable first, th… |
| `00aa2f20` | BaseEncounterPacket::~BaseEncounterPacket | confirmed | Calls FUN_009b40e0 with ECX=this+0x2f0 and no stack arguments, then calls FUN_00a284d0 with ECX=this+0x18 and no stack arguments, and finally stores BaseEncoun… |
| `008a3090` | BaseEncounterPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the encounter and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `008a32b0` | BaseEncounterPacket::~BaseEncounterPacket | likely | Restores BaseEncounterPacket::vftable at +0 and BasePacket::vftable at +0, then returns. |
| `00a22360` | BaseEncounterPacket::Dtor | likely | Destructor body that rewrites the object vtable through BaseEncounterPacket::vftable and then BasePacket::vftable; no members are released and no memory is fre… |

**ClientCommandProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa36c0` | ClientCommandProcessor::DispatchEncounterPacket 🌐 | confirmed | Reads the encounter message header and dispatches sub-opcodes 2, 3, 4, 5, 6, 0x66, 0x68, 0x69, 0x6a, 0x6b, 0x72, 0x78, 0x79, 0x82, 0x84 and 0x85. Case 0x72 con… |
| `00aa2980` | ClientCommandProcessor::ParseMatchmakingPacket 🌐 | likely | Reads a sub-opcode into local_14c and dispatches cases 2, 4, 5, 9, 0xc, 0xe, 0xf and 0x12. Case 9 builds an SoeUtil::IString and formats 'String#%d not found' … |

**EncounterDataCommon** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a69b0` | EncounterDataCommon::EncounterDataCommon | confirmed | Copy constructor for an EncounterDataCommon object (a BasePacket-derived encounter data record): installs BasePacket::vftable, copies the packet id word from t… |
| `008a6ae0` | EncounterDataCommon::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: restores BasePacket::vftable, conditionally calls _free(this) when param_2 bit 0 is set, and returns this. |

**EncounterDetailsCommon** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29120` | EncounterDetailsCommon::ReadRecord 🌐 | confirmed | Reads integers, nested team and participant lists, flags, objective data and trailing booleans from bounded stream. |
| `00a28430` | EncounterDetailsCommon::Ctor | likely | Initializes participant and team lists, record fields and embedded objective state, then sets trailing flags. |
| `00a284d0` | EncounterDetailsCommon::Dtor | likely | Destroys embedded objective state, clears team nodes and destroys nested participant list. |
| `00a28e20` | WriteEncounterDetailsCommonRecord | likely | Writes two identifier dwords, serializes teams through 00a27750 and participant-map lists through 00a27110, writes scalar fields and flags, emits nested object… |

**EncounterDetailsResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa2e90` | EncounterDetailsResponsePacket::EncounterDetailsResponsePac… 🌐 | confirmed | Calls FUN_008a2fd0(0x72) (encounter base constructor with sub-opcode 114), stores EncounterDetailsResponsePacket::vftable, clears the byte at object+0x2e8 and … |
| `00aa3050` | EncounterDetailsResponsePacket::ScalarDeletingDtor | confirmed | Calls FUN_00aa2f20 on this, tests flag bit 0, frees this when the bit is set, and otherwise returns this. |

**EncounterDuelInvitationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9a200` | EncounterDuelInvitationPacket::EncounterDuelInvitationPacket 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x68, stores EncounterDuelInvitationPacket::vftable, and copies the DAT_01be7b10/DAT_01be7b14 pair into both +0x18/+0x1c and +0x… |
| `00a9a280` | EncounterDuelInvitationPacket::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98df0` | EncounterDuelInvitationPacket::~EncounterDuelInvitationPack… | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x68 of the … |

**EncounterDuelInvitationResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a879f0` | EncounterDuelInvitationResponsePacketCtor 🌐 | confirmed | Constructor of the duel-invitation response packet: runs the BaseEncounterPacket family constructor 008a2fd0(0x69) (class id 41 plus sub-opcode 0x69), stores t… |
| `00a97c10` | EncounterDuelInvitationResponsePacket::BuildAndSend 🌐 | likely | Thiscall with a byte and two dword arguments. Constructs the packet in the stack record at buffer+0 via callee 00a879f0 (which stores EncounterDuelInvitationRe… |

**EncounterInvitationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9a0b0` | EncounterInvitationPacket::EncounterInvitationPacket 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x66, stores EncounterInvitationPacket::vftable, sets the words at +0x18 and +0x1c from DAT_01be7b10 and DAT_01be7b14, and zeroe… |
| `00a9a120` | EncounterInvitationPacket::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98cf0` | EncounterInvitationPacket::~EncounterInvitationPacket | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x66 of the … |

**EncounterInvitationResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3100` | EncounterInvitationResponsePacket::~EncounterInvitationResp… | likely | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; 009471a0 destroys the response object with it in a case whose branch first con… |
| `008b6da0` | EncounterInvitationResponsePacket::EncounterInvitationRespo… 🌐 | likely | Calls the encounter family constructor 008a2fd0(0x67) and installs EncounterInvitationResponsePacket::vftable; no field is initialised. |
| `008b6e00` | EncounterInvitationResponsePacket::ScalarDeletingDestructor | likely | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct cal… |

**EncounterMatchmaking::AddMatchRequestPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c08850` | AddMatchRequestPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of EncounterMatchmaking::AddMatchRequestPacket::vftable: runs the shared teardown 00c087f0 and calls _free on the object whe… |

**EncounterMatchmaking::BaseMatchmakingPacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7f0d0` | EncounterMatchmaking::BaseMatchmakingPacket::BaseMatchmakin… | confirmed | Common constructor for the matchmaking packet family: stores BasePacket::vftable at +0, writes 0x8d (141) at +4, replaces the vtable with BaseMatchmakingPacket… |
| `00a7f190` | EncounterMatchmaking::BaseMatchmakingPacket::ScalarDeleting… | confirmed | Scalar deleting destructor (slot 0 of the packet vtable): rewrites the vtable pointer to BaseMatchmakingPacket::vftable, then to BasePacket::vftable, and frees… |
| `00a98fa0` | EncounterMatchmaking::BaseMatchmakingPacket::BaseMatchmakin… 🌐 | confirmed | Stores BasePacket::vftable, writes the family byte 0x8d at +4, stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and zeroes the word at +8. |
| `00c087f0` | EncounterMatchmaking::BaseMatchmakingPacket::Dtor | confirmed | Base-class destructor body: destroys the embedded member via FUN_00a82b30, then rewrites the vtable to EncounterMatchmaking::BaseMatchmakingPacket::vftable and… |
| `00a7f140` | EncounterMatchmaking::BaseMatchmakingPacket::~BaseMatchmaki… | likely | Destructor body for a matchmaking packet object: rewrites the object's vtable pointer to EncounterMatchmaking::BaseMatchmakingPacket::vftable and then to BaseP… |

**EncounterMatchmaking::CancelMatchRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7fc60` | EncounterMatchmaking::CancelMatchRequestPacket::CancelMatch… 🌐 | confirmed | Constructor: calls the matchmaking packet base constructor 00a7f0d0 with sub-opcode 6 and installs the CancelMatchRequestPacket vtable. |
| `00a7fcc0` | EncounterMatchmaking::CancelMatchRequestPacket::ScalarDelet… | confirmed | Scalar deleting destructor: rewrites the vtable pointer to BaseMatchmakingPacket::vftable and then BasePacket::vftable and frees the object when bit 0 of param… |

**EncounterMatchmaking::ListQueuesRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7fb90` | EncounterMatchmaking::ListQueuesRequestPacket::ListQueuesRe… 🌐 | confirmed | Constructor: calls the matchmaking packet base constructor 00a7f0d0 with sub-opcode 1 (that base ctor writes BasePacket::vftable, the base opcode word 0x8d at … |
| `00a7fbf0` | EncounterMatchmaking::ListQueuesRequestPacket::ScalarDeleti… | confirmed | Scalar deleting destructor of the list-queues request packet: rewrites the vtable pointer to BaseMatchmakingPacket::vftable and then BasePacket::vftable, and f… |

**EncounterMatchmaking::MatchInvitationRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9a5e0` | EncounterMatchmaking::MatchInvitationRequestPacket::ScalarD… | confirmed | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0… |
| `00a991c0` | EncounterMatchmaking::MatchInvitationRequestPacket::~MatchI… | likely | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of… |
| `00a9a580` | EncounterMatchmaking::MatchInvitationRequestPacket::MatchIn… 🌐 | likely | Calls 00a7f0d0 with sub-opcode 9 (matchmaking-family base constructor) and stores EncounterMatchmaking::MatchInvitationRequestPacket::vftable. |

**EncounterMatchmaking::MatchmakingQueueDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a994c0` | EncounterMatchmaking::MatchmakingQueueDefinitionInit | likely | Moves its this argument into ESI and calls the MatchmakingQueueDefinition constructor 0106e850 with this in ECX, then returns this. The only value the decompil… |

**EncounterMatchmaking::MatchmakingServerStatusPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9a440` | EncounterMatchmaking::MatchmakingServerStatusPacket::Scalar… | confirmed | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0… |
| `00a990c0` | EncounterMatchmaking::MatchmakingServerStatusPacket::Matchm… 🌐 | likely | Calls 00a7f0d0 with sub-opcode 0xf (matchmaking-family base constructor) and stores EncounterMatchmaking::MatchmakingServerStatusPacket::vftable. |
| `00a99120` | EncounterMatchmaking::MatchmakingServerStatusPacket::~Match… | likely | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of… |

**EncounterMatchmaking::QueueStatsRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7fd30` | EncounterMatchmaking::QueueStatsRequestPacket::QueueStatsRe… 🌐 | confirmed | Constructor: calls the matchmaking packet base constructor 00a7f0d0 with sub-opcode 0xd (13) and installs the QueueStatsRequestPacket vtable. |
| `00a7fd90` | EncounterMatchmaking::QueueStatsRequestPacket::ScalarDeleti… | confirmed | Scalar deleting destructor: rewrites the vtable pointer to BaseMatchmakingPacket::vftable and then BasePacket::vftable and frees the object when bit 0 of param… |

**EncounterMatchmaking::RemoveParticipantFromQueueResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9a510` | EncounterMatchmaking::RemoveParticipantFromQueueResponsePac… | confirmed | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0… |
| `00a99170` | RemoveParticipantFromQueueResponsePacket::~RemoveParticipan… | likely | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of… |
| `00a9a4b0` | RemoveParticipantFromQueueResponsePacket::RemoveParticipant… 🌐 | likely | Calls 00a7f0d0 with sub-opcode 0x12 (matchmaking-family base constructor) and stores EncounterMatchmaking::RemoveParticipantFromQueueResponsePacket::vftable. |

**EncounterMatchmaking::SelectQueueForUserPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9a3d0` | EncounterMatchmaking::SelectQueueForUserPacket::ScalarDelet… | confirmed | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0… |
| `00a99010` | EncounterMatchmaking::SelectQueueForUserPacket::SelectQueue… 🌐 | likely | Calls 00a7f0d0 with sub-opcode 0xc (matchmaking-family base constructor), stores EncounterMatchmaking::SelectQueueForUserPacket::vftable and zeroes the word at… |
| `00a99070` | EncounterMatchmaking::SelectQueueForUserPacket::~SelectQueu… | likely | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of… |

**EncounterOverworldCombatPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98e40` | EncounterOverworldCombatPacket::EncounterOverworldCombatPac… 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x84, stores EncounterOverworldCombatPacket::vftable and clears the byte at +0x14. |
| `00a9a2f0` | EncounterOverworldCombatPacket::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98ea0` | EncounterOverworldCombatPacket::~EncounterOverworldCombatPa… | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x84 of the … |

**EncounterPacketEncounterComplete** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa1d70` | EncounterPacketEncounterComplete::EncounterPacketEncounterC… 🌐 | confirmed | Constructor taking (byte, a, b, c): calls the encounter-family constructor FUN_008a2fd0 with sub-opcode 3, stores the byte argument at +0x14, installs Encounte… |
| `00aa1e20` | EncounterPacketEncounterComplete::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00a98ac0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |
| `00a98ac0` | EncounterPacketEncounterComplete::~EncounterPacketEncounter… | likely | Calls 0103d010 three times on three embedded members, then stores BaseEncounterPacket::vftable and BasePacket::vftable into the object. Called from the end of … |

**EncounterPacketEncounterLaunchFailed** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a989c0` | EncounterPacketEncounterLaunchFailed::EncounterPacketEncoun… 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 4 (the shared BaseEncounterPacket constructor), then stores EncounterPacketEncounterLaunchFailed::vftable and zeroes the word at… |
| `00a99da0` | EncounterPacketEncounterLaunchFailed::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98a20` | EncounterPacketEncounterLaunchFailed::~EncounterPacketEncou… | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 4 of the inb… |

**EncounterPacketEncounterShutdown** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a99f60` | EncounterPacketEncounterShutdown::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98b40` | EncounterPacketEncounterShutdown::~EncounterPacketEncounter… | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 6 of the inb… |
| `00a99ef0` | EncounterPacketEncounterShutdown::EncounterPacketEncounterS… 🌐 | likely | Calls 008a2fd0 with sub-opcode 6, stores EncounterPacketEncounterShutdown::vftable, sets the words at +0x18 and +0x1c from DAT_01be7b10 and DAT_01be7b14, and z… |

**EncounterPacketIsFighting** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98ef0` | EncounterPacketIsFighting::EncounterPacketIsFighting 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x85, stores EncounterPacketIsFighting::vftable and clears the byte at +0x14. |
| `00a9a360` | EncounterPacketIsFighting::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98f50` | EncounterPacketIsFighting::~EncounterPacketIsFighting | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x85 of the … |

**EncounterPacketObjectiveState** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9c300` | EncounterPacketObjectiveState::EncounterPacketObjectiveState 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 5, stores EncounterPacketObjectiveState::vftable, then builds an inline SoeUtil::Array<ObjectiveData,0,1> member at +0x14 (vftab… |
| `00a9df10` | EncounterPacketObjectiveState::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00a9c370 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |
| `00a9c370` | EncounterPacketObjectiveState::~EncounterPacketObjectiveSta… | likely | Stores the Array<ObjectiveData,0,1> vtable at +0x14, calls 009ae540 on the word at +0x1c, releases the buffer at +0x18 through 00401c00 with alignment 8, zeroe… |

**EncounterPacketPlayerEnter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a99e10` | EncounterPacketPlayerEnter::EncounterPacketPlayerEnter 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 2, stores EncounterPacketPlayerEnter::vftable, sets the words at +0x18 and +0x1c from DAT_01be7b10 and DAT_01be7b14 and clears t… |
| `00a99e80` | EncounterPacketPlayerEnter::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a9ccc0` | EncounterPacketPlayerEnter::ReadBody 🌐 | confirmed | Reads the shared 12-byte encounter header through 008d6690 into object+4, +8, +0xc and +0x10, then reads 8 bytes into +0x18/+0x1c under one bound check and 1 b… |
| `00a98a70` | EncounterPacketPlayerEnter::~EncounterPacketPlayerEnter | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 2 of the inb… |

**EncounterParticipantCancelPendingEncounterPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3300` | EncounterParticipantCancelPendingEncounterPacket 🌐 | confirmed | Constructor: calls FUN_008a2fd0(0x7c) so the packet is encounter opcode 41 with sub-opcode 124, then installs EncounterParticipantCancelPendingEncounterPacket:… |
| `008a3360` | EncounterParticipantCancelPendingEncounterPacket::Dtor | likely | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; caller 00938fe0 constructs the cancel packet and destroys it with this functio… |
| `008b7030` | EncounterParticipantCancelPendingEncounterPacket::ScalarDel… | likely | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct cal… |

**EncounterParticipantMessagePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9b990` | EncounterParticipantMessagePacket::EncounterParticipantMess… 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x78, stores EncounterParticipantMessagePacket::vftable, then builds an inline SoeUtil::IString<char> member: IString<char>::vft… |
| `00a9ba00` | EncounterParticipantMessagePacket::~EncounterParticipantMes… | likely | Stores the IString<char> vtable at +0x20, releases the string at +0x24 when its count at +0x2c is positive, then stores BaseEncounterPacket::vftable and BasePa… |
| `00a9c400` | EncounterParticipantMessagePacket::ScalarDeletingDtor | likely | Calls 00a9ba00 (the EncounterParticipantMessagePacket destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and retur… |

**EncounterParticipantRequestEntrancePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b6e70` | EncounterParticipantRequestEntrancePacket::EncounterPartici… 🌐 | likely | Calls the encounter family constructor 008a2fd0(0x6c), installs EncounterParticipantRequestEntrancePacket::vftable and stores the global pair DAT_01be4850/DAT_… |
| `008b6ee0` | EncounterParticipantRequestEntrancePacket::ScalarDeletingDe… | likely | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct cal… |

**EncounterParticipantRequestExitPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a31a0` | EncounterParticipantRequestExitPacket::EncounterParticipant… 🌐 | confirmed | Constructor: calls FUN_008a2fd0(0x6d) so the packet is encounter opcode 41 with sub-opcode 109, then installs EncounterParticipantRequestExitPacket::vftable. |
| `008a3200` | EncounterParticipantRequestExitPacket::~EncounterParticipan… | likely | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; 009bd240 destroys the request-exit object with it right after constructing the… |
| `008b6f50` | EncounterParticipantRequestExitPacket::ScalarDeletingDestru… | likely | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct cal… |

**EncounterParticipantResumePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3250` | EncounterParticipantResumePacket::EncounterParticipantResum… 🌐 | confirmed | Constructor: calls FUN_008a2fd0(0x7a) so the packet is encounter opcode 41 with sub-opcode 122, installs EncounterParticipantResumePacket::vftable and zeroes o… |
| `008b6fc0` | EncounterParticipantResumePacket::ScalarDeletingDestructor | likely | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct cal… |

**EncounterParticipantTerminateEncounterPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98d40` | EncounterParticipantTerminateEncounterPacket::EncounterPart… 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x79, stores EncounterParticipantTerminateEncounterPacket::vftable and clears the byte at +0x14. |
| `00a9a190` | EncounterParticipantTerminateEncounterPacket::ScalarDeletin… | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98da0` | EncounterParticipantTerminateEncounterPacket::~EncounterPar… | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x79 of the … |

**EncounterRunningListRequestPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a22ec0` | EncounterRunningListRequestPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseEncounterPacket::vftable and BasePacket::vftable and calls _free on the object when bit 0 of the flag argument is set,… |

**EncounterRunningListResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa3500` | EncounterRunningListResponsePacket::EncounterRunningListRes… 🌐 | confirmed | Calls FUN_008a2fd0(0x82) (encounter base constructor with sub-opcode 130), stores EncounterRunningListResponsePacket::vftable, installs the EncounterDetailsCom… |
| `00aa3620` | EncounterRunningListResponsePacket::ScalarDeletingDtor | confirmed | Calls FUN_00aa3570 on this, frees this when param_2 bit 0 is set, and otherwise returns this. |
| `00aa3570` | EncounterRunningListResponsePacket::~EncounterRunningListRe… | likely | Stores the EncounterDetailsCommon list vftable at object+0x14, unlinks head nodes at +0x18 through FUN_00aa3220, then stores BaseEncounterPacket::vftable and B… |

**EncounterShowRespawnWindowPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a33b0` | EncounterShowRespawnWindowPacket::EncounterShowRespawnWindo… 🌐 | confirmed | Constructor: calls FUN_008a2fd0(0x7d) so the packet is encounter opcode 41 with sub-opcode 125, installs EncounterShowRespawnWindowPacket::vftable and zeroes p… |
| `008a3410` | EncounterShowRespawnWindowPacket::~EncounterShowRespawnWind… | likely | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; it is the last call of the respawn-window case in dispatcher 0090d830 and also… |
| `008b70a0` | EncounterShowRespawnWindowPacket::ScalarDeletingDestructor | likely | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct cal… |

**EncounterStatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98b90` | EncounterStatePacket::EncounterStatePacket 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x6a, then stores EncounterStatePacket::vftable and zeroes the word at +0x14. |
| `00a99fd0` | EncounterStatePacket::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98bf0` | EncounterStatePacket::~EncounterStatePacket | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x6a of the … |

**EncounterZoneIsReadyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a98c40` | EncounterZoneIsReadyPacket::EncounterZoneIsReadyPacket 🌐 | confirmed | Calls 008a2fd0 with sub-opcode 0x6b and stores EncounterZoneIsReadyPacket::vftable; no other field is initialised. |
| `00a9a040` | EncounterZoneIsReadyPacket::ScalarDeletingDtor | confirmed | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set… |
| `00a98ca0` | EncounterZoneIsReadyPacket::~EncounterZoneIsReadyPacket | likely | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x6b of the … |

**(no class)** (37)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093bd80` | SendEncounterResume 🌐 | confirmed | Constructs an encounter participant resume packet and, when the gateway exists, sends it through the tunneled client-packet path. |
| `00a9f760` | ParseEncounterPlayerEnterBody 🌐 | confirmed | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ccc0 (which reads the shared 12-byte enc… |
| `008d6690` | ReadEncounterHeader_008d6690 🌐 | likely | Reads the 12-byte header from the bounded reader in arg1: int16 sign-extended to this+4, int16 sign-extended to this+8, int32 to this+0xc and int32 to this+0x1… |
| `008d6730` | ReadEncounterHeaderThenI32x2_008d6730 🌐 | likely | Calls 008d6690 on the reader (12-byte encounter header), then reads two int32 into this+0x14 and this+0x18, each with its own bounds check. A short read stores… |
| `00938f00` | RequestEncounterEntrance 🌐 | likely | Only when this+0x700 equals 1, constructs an entrance request, copies four values from client state, and submits it when the gateway pointer at this+0x280 is n… |
| `00938fe0` | CancelPendingEncounter 🌐 | likely | Copies two shared globals into this+0x6f8/+0x6fc and calls two helpers. If this+0x700 equals 2, calls the ordinary minigame quit helper and skips the event and… |
| `0093be30` | SendEncounterResumeWithFlag 🌐 | likely | Constructs EncounterParticipantResumePacket, stores the supplied byte at packet offset 0x14, and sends through the tunneled client-packet path when the gateway… |
| `00a27340` | WriteEncounterTeamMemberList | likely | Writes the member count at team+0x10, walks the member head at team+8 through next links at node+0x80, writes each member key at node+0x88, then calls 00a26bf0… |
| `00a28610` | ConstructEncounterDetailsEntry | likely | Allocates a keyed record, constructs it and appends it to owner's list and hash bucket. |
| `00a28800` | WriteEncounterDetailsNestedPayload | likely | Writes leading scalar fields and flags, calls 00a263d0 three times, serializes the counted detail list through 00a286f0, then writes trailing bytes, integers, … |
| `00a28c70` | EncounterStatePacketRemoveByKeyHandler 🌐 | likely | Selects bucket from key low ten bits, removes matching record and calls virtual notifications on registry and owner. |
| `00a28d00` | ResetAndRequestRunningList 🌐 | likely | Constructs an EncounterRunningListRequestPacket temporary; when owner field +0x280 is nonzero, passes it to 00a27500 for tunneled sending, then repeatedly remo… |
| `00a7f200` | MatchmakingPacketDtorBody_00a7f200 | likely | Destructor body that rewrites the object's vtable pointer to BaseMatchmakingPacket::vftable and then to BasePacket::vftable; the 76-byte body is identical to 0… |
| `00a7f250` | MatchmakingPacketDtorBody_00a7f250 | likely | Destructor body identical to 00a7f200 (rewrites BaseMatchmakingPacket::vftable then BasePacket::vftable) reached only from the EH unwind funclets 015bc850 and … |
| `00a7f2a0` | MatchmakingPacketDtorBody_00a7f2a0 | likely | Destructor body identical to 00a7f200/00a7f250 (BaseMatchmakingPacket::vftable then BasePacket::vftable) reached only from the EH unwind funclet 015bc6e0. |
| `00a9ec30` | BodyRead_EncHeader_i32_at14 🌐 | likely | Reads the shared 12-byte encounter header through FUN_008d6690 (two 16-bit words plus two 32-bit values into destination +4, +8, +0xc, +0x10) and then one more… |
| `00a9ecd0` | BodyRead_EncHeader_bool_at14 🌐 | likely | Reads the shared 12-byte encounter header via FUN_008d6690 and then one boolean byte normalised with a nonzero compare into destination +0x14. |
| `00a9ed20` | BodyRead_EncHeader_i32_at14_2 🌐 | likely | Second emission of the encounter-header plus 32-bit-field reader: FUN_008d6690 fills +4, +8, +0xc, +0x10 and an extra 32-bit value is copied into destination +… |
| `00a9ed60` | BodyRead_EncHeader_bool_at14_2 🌐 | likely | Reads the shared encounter header with FUN_008d6690 and then a single boolean byte into destination +0x14 (normalised nonzero compare), same shape as 00a9ecd0 … |
| `00a9eda0` | BodyRead_EncHeader_bool_at14_3 🌐 | likely | Third emission of the header-plus-boolean reader: shared encounter header via FUN_008d6690 then one boolean byte into destination +0x14. |
| `00a9f6f0` | ParseEncounterBody_Case4 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ec30 (shared 12-byte encounter header pl… |
| `00a9f7d0` | ParseEncounterBody_Case6 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9cd30 (encounter header reader variant), … |
| `00a9f840` | ParseEncounterBody_Case104 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9cdc0, and returns 1 only when the whole … |
| `00a9f8b0` | ParseEncounterBody_Case105 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ce40, and returns 1 only when the whole … |
| `00a9f920` | ParseEncounterBody_Case102 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ceb0, and returns 1 only when the whole … |
| `00a9f990` | ParseEncounterBody_SubOp107 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls the shared encounter header reader FUN_008d6690 … |
| `00a9fa00` | ParseEncounterBody_Case121 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ecd0 (encounter header plus one boolean … |
| `00a9fa70` | ParseEncounterBody_Case120 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9cf90 (encounter header plus a length-pre… |
| `00a9fae0` | ParseEncounterBody_Case106 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ed20 (encounter header plus one 32-bit v… |
| `00a9fb50` | ParseEncounterBody_SubOp132 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9ed60 (encounter header plus one boolean … |
| `00a9fbc0` | ParseEncounterBody_Case133 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00a9eda0 (encounter header plus one boolean … |
| `00aa02b0` | BodyRead_EncHeader_bool_i32x3_reward3 🌐 | likely | Reads the shared 12-byte encounter header with FUN_008d6690, a boolean byte (nonzero compare) into destination +0x14, three 32-bit values into +0x18, +0x1c and… |
| `00aa14a0` | ParseEncounterBody_SubOp3 🌐 | likely | Body parse entry of the four-argument idiom: builds the local stream record over (body pointer, length), calls FUN_00aa02b0 (encounter header, boolean, three 3… |
| `00aa32d0` | ReadEncounterDetailsFields 🌐 | likely | Uses FUN_008d6690 and FUN_00a29120 on the reader, reads a bool into object+0x2e8 and a u32 into +0x2ec, then reads a nested list into +0x2f0 through FUN_009b07… |
| `00aa3430` | ReadEncounterDetailsBody 🌐 | likely | Builds a reader over the supplied body range, calls FUN_00aa32d0 for one record, and returns success only when the reader has no error and the range is consume… |
| `00aa3640` | ReadCountedNodeListBody 🌐 | likely | Builds a reader over the supplied body range, calls FUN_008d6690 on the reader, then calls FUN_00aa3390 to load the counted list into object+0x14; returns succ… |
| `00c08cf0` | SendAddMatchRequest_Gateway | likely | ClientGateway send template (world variant) for the matchmaking add-match request: serialises the request through FUN_00c08910 into a local buffer, finalises i… |

### `net.packet.factory`

**FactoryPacketRequestRecipeReward** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c14d40` | FactoryPacketRequestRecipeReward::FactoryPacketRequestRecip… 🌐 | confirmed | Constructor: calls the factory base constructor FUN_009d90d0 with sub-opcode 0xe (14), stores FactoryPacketRequestRecipeReward::vftable at *this and zeroes the… |
| `00c1ac60` | FactoryPacketRequestRecipeReward::ScalarDeletingDtor | confirmed | Resets the inherited factory and packet vptrs, then frees the allocation when the deleting flag bit is set. |
| `00c14da0` | FactoryPacketRequestRecipeReward::~FactoryPacketRequestReci… | likely | Destructor body: stores BaseFactoryPacket::vftable and then BasePacket::vftable at *this and returns. |

### `net.packet.fotomat`

**BaseFotomatPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a23f0` | BaseFotomatPacket::BaseFotomatPacket | confirmed | Fotomat family constructor: installs BasePacket::vftable, stores 0x9c (156) in the packet id slot param_1[1], installs BaseFotomatPacket::vftable and stores th… |
| `008a2460` | BaseFotomatPacket::~BaseFotomatPacket | confirmed | Destructor body: restores BaseFotomatPacket::vftable then BasePacket::vftable. |
| `0091d920` | BaseFotomatPacket::~BaseFotomatPacket 🌐 | confirmed | Destructor: runs the embedded-buffer cleanup FUN_0091d7d0 and then restores BaseFotomatPacket::vftable followed by BasePacket::vftable. |
| `00bd1c30` | BaseFotomatPacket::Ctor | confirmed | Constructor body: stores the BasePacket vftable at this+0, sets the opcode dword at +4 to 0x9C (156), then stores the BaseFotomatPacket vftable and clears the … |
| `00bd2d00` | BaseFotomatPacket::DtorBody | confirmed | Destructor body: calls the string member destructor 00709B80 on this, then stores the BaseFotomatPacket vftable and finally the BasePacket vftable at this+0. N… |
| `008a24b0` | BaseFotomatPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the Fotomat and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |

**PacketPlayerImageData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd3b40` | PacketPlayerImageData::PacketPlayerImageData | confirmed | Thiscall ctor: calls the BaseFotomatPacket ctor 008a23f0 on this with stack arg 3, stores PacketPlayerImageData::vftable at +0, the SoeUtil::IString<char> vfta… |
| `00bd3fd0` | PacketPlayerImageData::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor: calls the PacketPlayerImageData dtor body 00bd3be0 on this, then calls _free(this) (thunk 00d82463) when bit 0 of the flag… |
| `00bd4180` | PacketPlayerImageData::Read 🌐 | likely | Parses the PacketPlayerImageData body from a reader (reader cursor +8, end +0xc, error flag +0x10): 00bd31d0 reads two 16-bit header values into this+4 and thi… |

**PacketPortraitDataRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd3230` | PacketPortraitDataRequest::Ctor | confirmed | Constructor body: calls BaseFotomatPacket::Ctor 008A23F0 with the argument 2, stores the PacketPortraitDataRequest vftable at this+0, then initialises an embed… |
| `00bd3290` | PacketPortraitDataRequest::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls destructor body 00bd2d00 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |

**PacketSnapshotRequest** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091d8c0` | PacketSnapshotRequest::Ctor 🌐 | confirmed | Constructor: runs the BaseFotomatPacket family constructor FUN_008a23f0(4) (opcode 156 with sub-type 4), installs PacketSnapshotRequest::vftable and then initi… |
| `009229f0` | PacketSnapshotRequest_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 0091d920 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `0091d710` | PacketSnapshotRequest::InitFields | likely | Initializes three SoeUtil::IString<char> members of a large object at this+8, this+0x700e0 and this+0x700f0 (vtable plus empty-buffer pointer) and calls FUN_00… |
| `0091d7d0` | PacketSnapshotRequest::CleanupFields | likely | Releases the three SoeUtil::IString<char> members of the large object (+0x700f0, +0x700e0 via refcounted buffer release and +8 last), with a call to FUN_009193… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0094a1a0` | BuildAndSendSnapshotRequest 🌐 | likely | Constructs a PacketSnapshotRequest, copies two 16-dword blocks and string fields into its local data, walks the collection referenced at param_1+0xc0 and copie… |
| `00bd37e0` | ReadSubRecordFields | likely | Thiscall (ECX = record at PacketPlayerImageData+0x30) reading from a reader: a dword at +0, a dword at +4, two dwords at +8/+0xc, a dword at +0x10; calls 00913… |
| `00bd4d30` | FotomatRecvDispatchType3 🌐 | likely | Receive path for the Fotomat family: reads a header with 00bd31d0 from a reader struct built on the stack, and when the error flag is clear and the compared va… |

### `net.packet.group`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093daf0` | BaseClient::HandleGroupPacket | confirmed | Reads the group-family payload, switches on its subtype, and dispatches invite, leave, accept, kick, member, rename, encounter, job, map-ping, and minigame cas… |

**BaseGroupPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3560` | BaseGroupPacket::BaseGroupPacket 🌐 | confirmed | Group family constructor with a fixed zero sub-opcode: installs BasePacket::vftable, stores 0x28 (40) in the packet id slot param_1[1], installs BaseGroupPacke… |
| `008a35c0` | BaseGroupPacket::BaseGroupPacketSub | confirmed | Group family constructor taking the sub-opcode: installs BasePacket::vftable, stores 0x28 (40) in param_1[1], installs BaseGroupPacket::vftable and stores the … |
| `00c0afb0` | BaseGroupPacket_DtorBody | confirmed | Base-class destructor body: rewrites the vtable to BaseGroupPacket::vftable then BasePacket::vftable; called from the SetChangingJob handler 00c0b300 after the… |
| `008a3690` | BaseGroupPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the group and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set; it has no external cal… |

**GroupChangingJobPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0b0a0` | GroupChangingJobPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of GroupChangingJobPacket::vftable: rewrites the vtable to BaseGroupPacket::vftable then BasePacket::vftable and calls _free… |

**GroupChangingJobUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b7400` | GroupChangingJobUpdatePacket::GroupChangingJobUpdatePacket 🌐 | confirmed | Calls the group family initializer for top-level opcode 40 008a35c0(0xf), installs GroupChangingJobUpdatePacket::vftable, stores param_2 into +0x10, param_3 in… |
| `008a3a10` | GroupChangingJobUpdatePacket::~GroupChangingJobUpdatePacket | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the changing-job update with it after constructing th… |
| `008b7470` | GroupChangingJobUpdatePacket::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupInMinigamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b75d0` | GroupInMinigamePacket::GroupInMinigamePacket 🌐 | confirmed | Calls the group family initializer for top-level opcode 40 008a35c0(0x11), installs GroupInMinigamePacket::vftable, stores param_2 into +0x10, param_3 into +0x… |
| `008a3ab0` | GroupInMinigamePacket::~GroupInMinigamePacket | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the packet with it after constructing the class from … |
| `008b7640` | GroupInMinigamePacket::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupMapPingPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b74e0` | GroupMapPingPacket::GroupMapPingPacket 🌐 | confirmed | Calls the group family initializer for top-level opcode 40 008a35c0(0x10), installs GroupMapPingPacket::vftable and stores param_2..param_5 into +0x10..+0x1c. |
| `008b7560` | GroupMapPingPacket::ScalarDeletingDestructor | confirmed | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
| `008a3a60` | GroupMapPingPacket::~GroupMapPingPacket | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the map ping with it after constructing the class fro… |

**GroupPacketAnnounceEncounterReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b72f0` | GroupPacketAnnounceEncounterReply::GroupPacketAnnounceEncou… 🌐 | confirmed | Calls the group family initializer for top-level opcode 40 008a35c0(0xd) and installs GroupPacketAnnounceEncounterReply::vftable; no field is initialised. |
| `008b7370` | GroupPacketAnnounceEncounterReply::CopyPayloadFields10And14 | confirmed | Copies the dwords at param_2+0x10 and param_2+0x14 into this+0x10 and this+0x14; a two-field copy with no other effect. |
| `008b7390` | GroupPacketAnnounceEncounterReply::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupPacketGroupAccept** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcf30` | GroupPacketGroupAccept::GroupPacketGroupAccept 🌐 | confirmed | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 4, installs GroupPacketGroupAccept::vftable, calls 008dce10 with ECX=this+0x10,… |
| `008dcfc0` | GroupPacketGroupAccept::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d1610 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008d1610` | GroupPacketGroupAccept::~GroupPacketGroupAccept | likely | Destructor body of the group accept packet: runs the shared payload teardown FUN_008d1550 (members at +0x98 and +8), then rewrites the object vtable to BaseGro… |
| `008eb7c0` | GroupPacketGroupAccept::CopyPayloadAtOffsets124 | likely | Copies two source words to object offsets +0xa0 and +0xa4, invokes the IString copy helper for source offsets 0, +0x24, and +0x48, and copies three trailing wo… |
| `008eb830` | GroupPacketGroupAccept::CopyPayloadAtOffsets94 | likely | Copies two source words to object offsets +0x10 and +0x14, invokes the IString copy helper for source offsets 0, +0x24, and +0x48, and copies three trailing wo… |

**GroupPacketGroupAcceptReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a37b0` | GroupPacketGroupAcceptReply::GroupPacketGroupAcceptReply 🌐 | confirmed | Constructor: calls FUN_008a35c0(5) so the packet is group opcode 40 with sub-opcode 5, then installs GroupPacketGroupAcceptReply::vftable. |
| `008a3810` | GroupPacketGroupAcceptReply::~GroupPacketGroupAcceptReply | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the accept reply with it. |
| `008b71a0` | GroupPacketGroupAcceptReply::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupPacketGroupInvite** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dce80` | GroupPacketGroupInvite::GroupPacketGroupInvite 🌐 | confirmed | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 1 (family 40), installs GroupPacketGroupInvite::vftable, calls 008dce10 with EC… |
| `008dcf10` | GroupPacketGroupInvite::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d15b0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008d15b0` | GroupPacketGroupInvite::~GroupPacketGroupInvite | likely | Destructor body of the group invite packet: runs the shared payload teardown FUN_008d1550 (members at +0x98 and +8), then rewrites the object vtable to BaseGro… |

**GroupPacketGroupInviteReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3700` | GroupPacketGroupInviteReply::GroupPacketGroupInviteReply 🌐 | confirmed | Constructor: calls FUN_008a35c0(2) so the packet is group opcode 40 with sub-opcode 2, then installs GroupPacketGroupInviteReply::vftable. |
| `008a3760` | GroupPacketGroupInviteReply::~GroupPacketGroupInviteReply | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the invite reply with it after it is serialized. |
| `008b7130` | GroupPacketGroupInviteReply::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupPacketGroupKick** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd060` | GroupPacketGroupKick::GroupPacketGroupKick 🌐 | confirmed | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 6, installs GroupPacketGroupKick::vftable, stores DAT_01be4850/DAT_01be4854 at … |
| `008dd110` | GroupPacketGroupKick::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d16d0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008d16d0` | GroupPacketGroupKick::~GroupPacketGroupKick | likely | Destructor body of the group kick packet: destroys two members through FUN_0070a790, the sub-object at +0xa8 first (SEH state stepped from 1 to 0) and then the… |

**GroupPacketGroupKickReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3860` | GroupPacketGroupKickReply::GroupPacketGroupKickReply 🌐 | likely | Constructor: calls FUN_008a35c0(7) so the packet is group opcode 40 with sub-opcode 7, then installs GroupPacketGroupKickReply::vftable. |
| `008a38c0` | GroupPacketGroupKickReply::~GroupPacketGroupKickReply | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the kick reply with it. |
| `008b7210` | GroupPacketGroupKickReply::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupPacketGroupLeave** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dcfe0` | GroupPacketGroupLeave::GroupPacketGroupLeave 🌐 | confirmed | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 3, installs GroupPacketGroupLeave::vftable and calls embedded member initialize… |
| `008dd040` | GroupPacketGroupLeave::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d1670 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008d1670` | GroupPacketGroupLeave::~GroupPacketGroupLeave | likely | Destructor body of the group leave packet: destroys the member at +0x18 through FUN_0070a790, then rewrites the object vtable to BaseGroupPacket::vftable and B… |
| `008eb890` | GroupPacketGroupLeave::CopyPayloadAtOffsets94 | likely | Copies two source words to object offsets +0x10 and +0x14, invokes the IString copy helper for source offsets 0, +0x24, and +0x48, and copies three trailing wo… |

**GroupPacketGroupMemberUpdate** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd130` | GroupPacketGroupMemberUpdate::GroupPacketGroupMemberUpdate 🌐 | confirmed | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 9, installs GroupPacketGroupMemberUpdate::vftable and calls Ctor_GroupMemberRec… |
| `008dd190` | GroupPacketGroupMemberUpdate::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d1790 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008d1790` | GroupPacketGroupMemberUpdate::~GroupPacketGroupMemberUpdate | likely | Destructor body of the group member-update packet: destroys the member at +0x18 (reached as +8 of the sub-object at +0x10, with a zero byte stored into the SEH… |

**GroupPacketGroupUpdate** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3910` | GroupPacketGroupUpdate::GroupPacketGroupUpdate 🌐 | confirmed | Constructor: calls FUN_008a35c0(8) so the packet is group opcode 40 with sub-opcode 8, installs GroupPacketGroupUpdate::vftable and zeroes param_1[3]. |
| `008a3970` | GroupPacketGroupUpdate::~GroupPacketGroupUpdate | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the group update with it. |
| `008b7280` | GroupPacketGroupUpdate::ScalarDeletingDestructor | likely | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |

**GroupPacketRenamePlayer** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd1b0` | GroupPacketRenamePlayer::GroupPacketRenamePlayer 🌐 | confirmed | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 0xb (11), installs GroupPacketRenamePlayer::vftable and calls embedded member i… |
| `008dd210` | GroupPacketRenamePlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body 008d1800 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008a3630` | GroupPacketRenamePlayer::~GroupPacketRenamePlayer | likely | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; the group dispatcher 0093daf0 destroys the rename packet with it immediately after… |
| `008d1800` | GroupPacketRenamePlayer::~GroupPacketRenamePlayer | likely | Destructor body of the group rename-player packet: destroys the member at +0x18 through FUN_0070a790, then rewrites the object vtable to BaseGroupPacket::vftab… |

**(no class)** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a34d0` | CopyGroupMemberFields | likely | Field-copy routine that copies ten member fields from param_2 to param_1 at offsets 0x94, 0x98, 0x9c, 0xa0, 0xa4, 0xa8, 0xc0, 0xc4, 0xc5 and 0xc8 (two of them … |
| `008d1550` | Dtor_MembersAt8And98 | likely | Member destructor of a shared record: destroys the sub-object at +0x98 through FUN_0070a790, steps the SEH state word to 0xffffffff, then destroys the sub-obje… |
| `008d1740` | Dtor_MemberAt8 | likely | Member destructor: adds 8 to the incoming this pointer (raw bytes: 83 c1 08) and calls FUN_0070a790 on it, destroying exactly one sub-object at +8; it rewrites… |
| `008d3440` | ScalarDeletingDtor_Via008d1550 | likely | MSVC scalar deleting destructor: runs the shared group payload record teardown FUN_008d1550 (sub-objects at +0x98 and +8) and frees the object through the CRT … |
| `008d3590` | DeleteMemberAt6e8 | likely | Reads the member pointer at this+0x6e8 and, when non-null, runs the shared group payload record teardown FUN_008d1550 on it and frees it through the CRT _free … |
| `008dcd60` | Ctor_GroupMemberRecord | likely | Initialises a record: stores DAT_01be4850/DAT_01be4854 at +0/+4, calls the embedded member initializer 0070bac0 with ECX=this+8, then writes fields +0x90..+0xa… |
| `008f9450` | ReadBodyVia_008d8ed0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f94c0` | ReadBodyVia_008d8f40 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f9530` | ReadBodyVia_008d8fb0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f95a0` | ReadBodyVia_008d9020 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `00908fd0` | SpanReadDwordToThis0xC | likely | Thiscall-style (ECX = output object, stack: start, length, out pointer; ret 4 pattern not used, plain ret). Builds an iterator over the span, calls 008d8e70 wi… |
| `00909030` | SpanReadDwordToThis0xC_B | likely | Byte-identical logic to 00908fd0 (96 bytes; call target 008d8e70 shared): reads a 32-bit span field into [out+0xc] when cursor+4 <= end, otherwise sets [out+0x… |
| `00909090` | SpanReadDwordToThis0xC_C | likely | Byte-identical logic to 00908fd0 (96 bytes; call target 008d8e70 shared): reads a 32-bit span field into [out+0xc] when cursor+4 <= end, otherwise sets [out+0x… |
| `00c0b150` | GroupChangingJobPacket_SerializeBody | likely | Packet body writer: calls FUN_0090ae60(buffer) for the group preamble, then appends the single byte at +0xc (the changing-job flag) through the bounded copy FU… |
| `00c0b1d0` | SendGroupChangingJob_Gateway | likely | ClientGateway send template (world variant) for the changing-job packet: serialises the source through FUN_00c0b150 into a local buffer, finalises it, construc… |

### `net.packet.guild`

**AdminGuildServerVersionPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19d70` | AdminGuildServerVersionPacket::AdminGuildServerVersionPacket 🌐 | confirmed | Constructor: calls the admin guild base constructor 00b174c0 with sub-opcode 0x29 (41), installs AdminGuildServerVersionPacket::vftable, initializes the embedd… |
| `00b1a4f0` | AdminGuildServerVersionPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of AdminGuildServerVersionPacket::vftable: calls the destructor body 00b19de0 and frees the object with _free when bit 0 of… |

**BaseAdminGuildPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b17580` | BaseAdminGuildPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of BaseAdminGuildPacket::vftable: under an exception frame it restores BaseAdminGuildPacket::vftable and then BasePacket::v… |
| `00b174c0` | BaseAdminGuildPacket::BaseAdminGuildPacket 🌐 | likely | Base constructor of the admin guild packet family: stores BasePacket::vftable, writes 0x83 (131) into the packet id slot at +4, installs BaseAdminGuildPacket::… |

**BaseGuildPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09be0` | BaseGuildPacket::~BaseGuildPacket | confirmed | Destructor body: rewrites the vtable to BaseGuildPacket::vftable, then to BasePacket::vftable; no members released. |
| `00a09c30` | BaseGuildPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of the guild base packet: resets the two vtables and frees this when the delete flag is set. |
| `00a09ca0` | BaseGuildPacket::~BaseGuildPacket_B | confirmed | Second emission of the guild base destructor body (BaseGuildPacket then BasePacket vtable reset). |
| `00a09cf0` | BaseGuildPacket::~BaseGuildPacket_C | confirmed | Third emission of the guild base destructor body (BaseGuildPacket then BasePacket vtable reset). |
| `00a09d40` | BaseGuildPacket::~BaseGuildPacket_D | confirmed | Fourth emission of the guild base destructor body (BaseGuildPacket then BasePacket vtable reset). |
| `00b17220` | BaseGuildPacket::BaseGuildPacket 🌐 | confirmed | Base constructor of the guild packet family: stores BasePacket::vftable into *this, writes 0x81 (129) into the packet id slot at +4, installs BaseGuildPacket::… |

**GuildCanCreateGuildPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b17ec0` | GuildCanCreateGuildPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildCanCreateGuildPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::v… |

**GuildCommand_TransferGuild** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014cfef0` | GuildCommand_TransferGuild::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8) with the stream, writes 'Starting GuildCommand_TransferGuild' trace text through the no-op 013c26e0, … |
| `014d0170` | GuildCommand_TransferGuild::~GuildCommand_TransferGuild | confirmed | Stores GuildCommand_TransferGuild::vftable at *this, calls vtable [0] with argument 1 on each non-null pointer in the vector at +0xc/+0x10, moves the range wit… |
| `014d02c0` | GuildCommand_TransferGuild::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'GuildCommand_TransferGuild' (mov eax, imm32; ret). |
| `014d0330` | GuildCommand_TransferGuild::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014d0170, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014d03c0` | GuildCommand_TransferGuild::DumpText | confirmed | Virtual slot [19]. Writes 'Transfer Guild Guild Command' with a newline through 01241650 (0x1d characters), then calls 014d6a10 with its argument. |
| `014d03f0` | GuildCommand_TransferGuild::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Transfer Guild Guild Command' (0x1c characters) through 01241650. |

**GuildCreatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0ee30` | GuildCreatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildCreatePacket): calls the destructor body FUN_00c0eae0 (which releases the three embedded strings and resets t… |
| `00c0ea50` | GuildCreatePacket::GuildCreatePacket 🌐 | likely | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 1, stores GuildCreatePacket::vftable at *this, and initialises three SoeUtil:… |

**GuildDataFullPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b17f30` | GuildDataFullPacket::GuildDataFullPacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x16 (22), installs GuildDataFullPacket::vftable and zeroes the member at +0x18 (param_1… |
| `00b17f90` | GuildDataFullPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildDataFullPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable… |

**GuildDeleteNotificationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19660` | GuildDeleteNotificationPacket::GuildDeleteNotificationPacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1d (29), installs GuildDeleteNotificationPacket::vftable, zeroes the two dwords at +0x… |
| `00b1a410` | GuildDeleteNotificationPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildDeleteNotificationPacket::vftable: calls the destructor body 00b196d0 and frees the object with _free when bit 0 of… |
| `00b1ace0` | GuildDeleteNotificationPacket::ReadBody | likely | Reads the body of a guild deletion notification: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and a length-prefixed block skipped… |

**GuildDeletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0e510` | GuildDeletePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildDeletePacket): resets the object's vtables to BaseGuildPacket::vftable and then BasePacket::vftable, and when… |
| `00c0e300` | GuildDeletePacket::~GuildDeletePacket | likely | Destructor body of a guild packet with no owned members: writes BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns (single vtable reset… |

**GuildDemotePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0e660` | GuildDemotePacket::GuildDemotePacket 🌐 | confirmed | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 5, stores GuildDemotePacket::vftable at *this, clears the four scalar fields … |
| `00c0e6d0` | GuildDemotePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildDemotePacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object t… |
| `00c0e3a0` | GuildDemotePacket::~GuildDemotePacket | likely | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |

**GuildErrorPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1a4d0` | GuildErrorPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildErrorPacket::vftable: calls the destructor body 00b19ce0 and frees the object with _free when bit 0 of the flag arg… |
| `00b19c70` | GuildErrorPacket::GuildErrorPacket 🌐 | likely | Constructor: calls the base guild constructor 00b17220 (which writes opcode 129 and leaves the sub-opcode word at zero), installs GuildErrorPacket::vftable, in… |
| `00b1ad70` | GuildErrorPacket::ReadBody 🌐 | likely | Reads the body of a guild error packet: two 16-bit header values through 00b1a810 and then a length-prefixed block skipped through 00894b10, flagging the reade… |

**GuildInviteAcceptNotificationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1b180` | GuildInviteAcceptNotificationPacket::GuildInviteAcceptNotif… 🌐 | confirmed | Constructor: calls the base guild constructor 00b17220 (opcode 129, sub-opcode word zeroed), installs GuildInviteAcceptNotificationPacket::vftable and builds o… |
| `00b1b1e0` | GuildInviteAcceptNotificationPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildInviteAcceptNotificationPacket::vftable: calls the destructor body 00b1a2d0 and frees the object with _free when bi… |
| `00b1a2d0` | GuildInviteAcceptNotificationPacket::Destruct | likely | Destructor body of the guild invite-accept notification: releases its name record through 0070a790 under an exception frame, then restores BaseGuildPacket::vft… |

**GuildInviteAcceptPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09e20` | GuildInviteAcceptPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets vtables to BaseGuildPacket and BasePacket and frees this when the delete flag is set. |
| `00c0e8f0` | GuildInviteAcceptPacket::GuildInviteAcceptPacket 🌐 | likely | Constructor: calls the guild invite-packet base constructor FUN_00b17220 and then stores GuildInviteAcceptPacket::vftable at *this (no extra member initialisat… |

**GuildInviteCancelPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19460` | GuildInviteCancelPacket::GuildInviteCancelPacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1a (26), installs GuildInviteCancelPacket::vftable, initializes the embedded IString a… |

**GuildInviteDeclineNotificationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1b200` | GuildInviteDeclineNotificationPacket::GuildInviteDeclineNot… 🌐 | confirmed | Constructor: calls the base guild constructor 00b17220 (opcode 129, sub-opcode word zeroed), installs GuildInviteDeclineNotificationPacket::vftable and builds … |
| `00b1b260` | GuildInviteDeclineNotificationPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildInviteDeclineNotificationPacket::vftable: calls the destructor body 00b1a330 and frees the object with _free when b… |
| `00b1a330` | GuildInviteDeclineNotificationPacket::Destruct | likely | Destructor body of the guild invite-decline notification: releases its name record through 0070a790 under an exception frame, then restores BaseGuildPacket::vf… |

**GuildInviteDeclinePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09f00` | GuildInviteDeclinePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets vtables to BaseGuildPacket and BasePacket and frees this when the delete flag is set. |
| `00c0e950` | GuildInviteDeclinePacket::GuildInviteDeclinePacket 🌐 | likely | Constructor: calls the guild invite-packet base constructor FUN_00b17220 and then stores GuildInviteDeclinePacket::vftable at *this; returns the object. |

**GuildInviteNotificationPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1b0d0` | GuildInviteNotificationPacket::GuildInviteNotificationPacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x18 (24), installs GuildInviteNotificationPacket::vftable, builds the two embedded name… |
| `00b1b160` | GuildInviteNotificationPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildInviteNotificationPacket::vftable: calls the destructor body 00b1a210 and frees the object with _free when bit 0 of… |

**GuildInvitePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0ebd0` | GuildInvitePacket::GuildInvitePacket 🌐 | confirmed | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 3, stores GuildInvitePacket::vftable at *this, clears the four scalar fields … |
| `00c0ec50` | GuildInvitePacket::~GuildInvitePacket | confirmed | Destructor body: releases the embedded SoeUtil::IString member at +0x20 (interlocked reference decrement on the buffer header at ptr-4, virtual slot +8 when it… |
| `00c0ee50` | GuildInvitePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildInvitePacket): calls the destructor body FUN_00c0ec50 (releases the embedded string, resets the vtables) and … |

**GuildInviteTimeOutPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09fe0` | GuildInviteTimeOutPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets vtables to BaseGuildPacket and BasePacket and frees this when the delete flag is set. |

**GuildMemberLocationRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19760` | GuildMemberLocationRequest::GuildMemberLocationRequest 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0xf (15), installs GuildMemberLocationRequest::vftable, stores the two header arguments … |
| `00b1a430` | GuildMemberLocationRequest::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildMemberLocationRequest::vftable: calls the destructor body 00b197e0 and frees the object with _free when bit 0 of th… |
| `00b197e0` | GuildMemberLocationRequest::Destruct | likely | Destructor body of the member location request: rewrites the embedded guid-array vtable at +0x18, zeroes the array count, frees the array storage at +0x1c with… |

**GuildMemberLocationUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19860` | GuildMemberLocationUpdatePacket::GuildMemberLocationUpdateP… 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x19 (25), installs GuildMemberLocationUpdatePacket::vftable, stores the two header argu… |
| `00b1a450` | GuildMemberLocationUpdatePacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildMemberLocationUpdatePacket::vftable: calls the destructor body 00b198e0 and frees the object with _free when bit 0 … |
| `00b198e0` | GuildMemberLocationUpdatePacket::Destruct | likely | Destructor body of the member location update: rewrites the entry-array vtable at +0x18, zeroes the array count, frees the entry storage at +0x1c with alignmen… |

**GuildMemberStatusUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1b280` | GuildMemberStatusUpdatePacket::GuildMemberStatusUpdatePacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x11 (17), installs GuildMemberStatusUpdatePacket::vftable, zeroes the four dwords at +0… |
| `00b1b320` | GuildMemberStatusUpdatePacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildMemberStatusUpdatePacket::vftable: calls the destructor body 00b1a390 and frees the object with _free when bit 0 of… |
| `00b1a390` | GuildMemberStatusUpdatePacket::Destruct | likely | Destructor body of the guild member status update: releases its name record through 0070a790 under an exception frame, then restores BaseGuildPacket::vftable a… |

**GuildMotdUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19560` | GuildMotdUpdatePacket::GuildMotdUpdatePacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0xb (11), installs GuildMotdUpdatePacket::vftable, zeroes the two dwords at +0x10 and +0… |
| `00b1a3f0` | GuildMotdUpdatePacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildMotdUpdatePacket::vftable: calls the destructor body 00b195d0 and frees the object with _free when bit 0 of the fla… |

**GuildNameAcceptedPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19a70` | GuildNameAcceptedPacket::GuildNameAcceptedPacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x13 (19), installs GuildNameAcceptedPacket::vftable, zeroes the dwords at +0x10 and +0x… |
| `00b1a490` | GuildNameAcceptedPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildNameAcceptedPacket::vftable: calls the destructor body 00b19ae0 and frees the object with _free when bit 0 of the f… |
| `00b1a870` | GuildNameAcceptedPacket::ReadBody | likely | Reads the body of a name-accepted guild packet: two 16-bit header values through 00b1a810, then an 8-byte pair into +0x10/+0x14, then a 32-bit length followed … |

**GuildNameRejectedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b18160` | GuildNameRejectedPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildNameRejectedPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vft… |
| `00b18100` | GuildNameRejectedPacket::GuildNameRejectedPacket 🌐 | likely | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x14 (20), installs GuildNameRejectedPacket::vftable and zeroes the two member dwords at… |

**GuildNameRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b18000` | GuildNameRequestPacket::GuildNameRequestPacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0xe (14), installs GuildNameRequestPacket::vftable and zeroes the two member dwords at +… |
| `00b18060` | GuildNameRequestPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildNameRequestPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vfta… |

**GuildNameUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b19b70` | GuildNameUpdatePacket::GuildNameUpdatePacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x12 (18), installs GuildNameUpdatePacket::vftable, zeroes the dwords at +0x10 and +0x14… |
| `00b1a4b0` | GuildNameUpdatePacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildNameUpdatePacket::vftable: calls the destructor body 00b19be0 and frees the object with _free when bit 0 of the fla… |

**GuildPaidRenameCheckReplyPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1a470` | GuildPaidRenameCheckReplyPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildPaidRenameCheckReplyPacket::vftable: calls the destructor body 00b199e0 and frees the object with _free when bit 0 … |
| `00b19960` | GuildPaidRenameCheckReplyPacket::GuildPaidRenameCheckReplyP… 🌐 | likely | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1e (30), installs GuildPaidRenameCheckReplyPacket::vftable, zeroes the dwords at +0x10… |

**GuildPlayerStatusUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b182b0` | GuildPlayerStatusUpdatePacket::GuildPlayerStatusUpdatePacket 🌐 | confirmed | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1f (31) and installs GuildPlayerStatusUpdatePacket::vftable; no further members are in… |
| `00b18310` | GuildPlayerStatusUpdatePacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildPlayerStatusUpdatePacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacke… |
| `00b1add0` | GuildPlayerStatusUpdatePacket::ReadBody 🌐 | likely | Reads the body of a player status update: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14, a second 8-byte pair into +0x18/+0x1c and… |

**GuildPromotePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0e580` | GuildPromotePacket::GuildPromotePacket 🌐 | confirmed | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 4, stores GuildPromotePacket::vftable at *this, clears the four scalar fields… |
| `00c0e5f0` | GuildPromotePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildPromotePacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object … |
| `00c0e350` | GuildPromotePacket::~GuildPromotePacket | likely | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |

**GuildQuitPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0e820` | GuildQuitPacket::GuildQuitPacket 🌐 | confirmed | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 7, stores GuildQuitPacket::vftable at *this, clears the two scalar fields at … |
| `00c0e880` | GuildQuitPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildQuitPacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object thr… |
| `00c0e440` | GuildQuitPacket::~GuildQuitPacket | likely | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |

**GuildRemovePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0e7b0` | GuildRemovePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildRemovePacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object t… |
| `00c0e3f0` | GuildRemovePacket::~GuildRemovePacket | likely | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |

**GuildRenameGuildPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0ee70` | GuildRenameGuildPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuildRenameGuildPacket): calls the destructor body FUN_00c0ed70 (releases the two embedded strings, resets the vta… |
| `00c0ece0` | GuildRenameGuildPacket::GuildRenameGuildPacket 🌐 | likely | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 0xd (13), stores GuildRenameGuildPacket::vftable at *this, clears the two sca… |

**GuildRoleNameChangedPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b18240` | GuildRoleNameChangedPacket::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of GuildRoleNameChangedPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::… |
| `00b181d0` | GuildRoleNameChangedPacket::GuildRoleNameChangedPacket 🌐 | likely | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x15 (21), installs GuildRoleNameChangedPacket::vftable and initializes four member dwor… |
| `00b1aa90` | GuildRoleNameChangedPacket::ReadBody | likely | Reads the body of a role-rename notification: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and then two 32-bit values into +0x18 … |

**(no class)** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b172e0` | GuildPacketDtorBody_00b172e0 | likely | Memberless guild-packet destructor body: rewrites *this first with BaseGuildPacket::vftable and then with BasePacket::vftable and returns; it is called by 00b1… |
| `00b17330` | GuildPacketDtorBody_00b17330 | likely | Memberless guild-packet destructor body: rewrites *this with BaseGuildPacket::vftable and then BasePacket::vftable and returns; its only caller is the unwind f… |
| `00b17380` | GuildPacketDtorBody_00b17380 | likely | Memberless guild-packet destructor body: rewrites *this with BaseGuildPacket::vftable and then BasePacket::vftable and returns; it is called by 00b1e960 and by… |
| `00b173d0` | GuildPacketDtorBody_00b173d0 | likely | Memberless guild-packet destructor body: rewrites *this with BaseGuildPacket::vftable and then BasePacket::vftable and returns; it is called by 00b1ee10 and by… |
| `00b17420` | GuildPacketDtorBody_00b17420 | likely | Memberless guild-packet destructor body: rewrites *this with BaseGuildPacket::vftable and then BasePacket::vftable and returns; it is called by 00b1ee10 and by… |
| `00b17470` | GuildPacketDtorBody_00b17470 | likely | Memberless guild-packet destructor body: rewrites *this with BaseGuildPacket::vftable and then BasePacket::vftable and returns; its only caller is the unwind f… |
| `00b17530` | BaseAdminGuildPacket_DtorBody | likely | Admin-guild packet destructor body: rewrites *this with BaseAdminGuildPacket::vftable and then BasePacket::vftable and returns; both callers are unwind funclet… |
| `00b1a9b0` | GuildMemberStatusOrNameUpdate_ReadBody 🌐 | likely | Reads a guild body of the form two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and a length-prefixed block skipped through 00894b10,… |
| `00b1bcb0` | ReadU16PairThenU64 | likely | First steps FUN_00b1a810, which reads two 16-bit values into the destination's +4 and +8; then copies the next 8 reader bytes into the destination's +0x10/+0x1… |
| `00b1bd30` | ReadU16PairThenBool | likely | Steps FUN_00b1a810 (two 16-bit values into the destination's +4 and +8), then reads one byte into the destination's +0xc as a boolean (nonzero becomes 1), zero… |
| `00b1bdb0` | ReadCountAdjustArray | likely | Reads a 32-bit count from the reader and bounds-checks it against the remaining bytes; when the destination's stored count at +8 is smaller it grows the array … |
| `00b1be30` | Read4xU64Dword2Skips | likely | Copies four 8-byte values from the reader into the destination's +0/+4, +8/+0xc, +0x10/+0x14 and +0x20/+0x24, a single dword into +0x18, zeroing each field and… |
| `00b1bfc0` | ReadU64DwordBool4xDwords | likely | Reads an 8-byte value into the destination's +0x10/+0x14, a dword into +0x18, skips one variable-length item (FUN_008e7290), a byte flag into +0xb4 and four dw… |
| `00b1c230` | ReadGuildNameAcceptedParams 🌐 | likely | Reader-over-buffer wrapper: builds a stack reader over the (param_2, param_3) byte range, calls FUN_00b1a870 to fill the destination (two 16-bit fields at +4/+… |
| `00b1c2a0` | ReadGuildPaidRenameReplyParams 🌐 | likely | Reader-over-buffer wrapper: sets up a stack reader over the (param_2, param_3) byte range, calls FUN_00b1a900 (two 16-bit fields at +4/+8, an 8-byte value at +… |
| `00b1c310` | ReadGuildNameUpdateParams 🌐 | likely | Reader-over-buffer wrapper: builds a stack reader over (param_2, param_3), calls FUN_00b1a9b0 (two 16-bit fields at +4/+8, an 8-byte value at +0x10/+0x14 and a… |
| `00b1c380` | ReadGuildNameRejectedParams 🌐 | likely | Reader-over-buffer wrapper: builds a stack reader over (param_2, param_3), calls the typed reader FUN_00b1bcb0 (two 16-bit fields at +4/+8 and an 8-byte value … |
| `00b1c540` | ReadGuildCanCreateParams 🌐 | likely | Reader-over-buffer wrapper: builds a stack reader over (param_2, param_3), calls FUN_00b1bd30 (two 16-bit fields plus a single boolean byte) and returns 1 on s… |
| `00b1c770` | ReadU64ThenCountedArray 🌐 | likely | Steps FUN_00b1a810 (two 16-bit fields into the destination's +4/+8), copies 8 reader bytes into the destination's +0x10/+0x14, then calls the counted-array rea… |
| `00b1d210` | ReadStructThenBlob 🌐 | likely | Steps FUN_00b1a810 (two 16-bit fields), calls the fixed-record reader FUN_00b1be30, then reads a 32-bit length and copies that many reader bytes with FUN_00894… |
| `00b1d2d0` | ReadCountedItemsIntoList 🌐 | likely | Empties the destination's linked list by repeatedly calling FUN_00b19170 on the head stored at +8 until it is null, reads a 32-bit count from the reader, then … |

### `net.packet.housing`

**AdminHousingServerVersionPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aad040` | AdminHousingServerVersionPacket::ctor 🌐 | confirmed | Constructor of the admin housing server-version packet: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x39 (57, base opcode 127), … |
| `00aad0b0` | AdminHousingServerVersionPacket::~AdminHousingServerVersion… | likely | Destructor of the admin housing server-version packet: releases the SoeUtil::IString<char> member at +0xc (vtable reset, refcount decrement on the buffer, rele… |

**BaseHousingPacket** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4930` | BaseHousingPacket::BaseHousingPacket | confirmed | Stores BasePacket::vftable, sets the family opcode field to 0x7f (127), then stores BaseHousingPacket::vftable and zeroes the sub-opcode field. |
| `00aa4990` | BaseHousingPacket::BaseHousingPacketSub | confirmed | Stores BasePacket::vftable, sets the family opcode field to 0x7f (127), stores BaseHousingPacket::vftable, then writes the sub-opcode argument into object+0x8. |
| `00aa4a00` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4a50` | BaseHousingPacket::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable and BasePacket::vftable, then calls _free on the object when bit 0 of param_2 is set. |
| `00aa4b20` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4bd0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4c20` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4c70` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4d20` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4dd0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4e20` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4ed0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4f80` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5030` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa50f0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa51a0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa51f0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5240` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa52f0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa53a0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5450` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa54a0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5550` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa55a0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5660` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5710` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5760` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5810` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5930` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa59e0` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5a90` | BaseHousingPacket::~BaseHousingPacket | confirmed | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aba910` | BaseHousingPacket::WriteHeader 🌐 | confirmed | Thiscall on a housing packet object: writes its base opcode at +4 as a u16, then its inner opcode at +8 as a u16 into the writer argument, clipping each chunk … |
| `00aad1a0` | BaseHousingPacket::~BaseHousingPacket | likely | Base destructor: calls the base-packet destructor FUN_00777340 and resets the object's vtable to BaseHousingPacket::vftable and then BasePacket::vftable. |

**ClientHousingPacketApplyCustomizationToFixtureGroupAndType** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3dd0` | ClientHousingPacketApplyCustomizationToFixtureGroupAndType:… 🌐 | confirmed | Constructs subtype 59, installs its request packet vtable and initializes an embedded value and two fixed 64-byte strings. |
| `00ab3e50` | ClientHousingPacketApplyCustomizationToFixtureGroupAndType:… | confirmed | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |

**ClientHousingPacketChangeHouseName** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5140` | ClientHousingPacketChangeHouseName::ClientHousingPacketChan… 🌐 | confirmed | Calls FUN_00aa4990(9) to build the family 127 header with sub-opcode 9, then stores ClientHousingPacketChangeHouseName::vftable. |
| `00aa8060` | ClientHousingPacketChangeHouseName::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketDeclineInvite** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa7b10` | ClientHousingPacketDeclineInvite::ClientHousingPacketDeclin… 🌐 | confirmed | Calls FUN_00aa4990(0x10) to build the family 127 header with sub-opcode 16, stores ClientHousingPacketDeclineInvite::vftable and copies two default dwords from… |
| `00aa7b80` | ClientHousingPacketDeclineInvite::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketEnterRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa83c0` | ClientHousingPacketEnterRequest::ClientHousingPacketEnterRe… 🌐 | confirmed | Calls FUN_00aa4990(0x13) to build the family 127 header with sub-opcode 19, stores ClientHousingPacketEnterRequest::vftable, copies two default dwords from DAT… |
| `00aa8430` | ClientHousingPacketEnterRequest::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abca80` | ClientHousingPacketEnterRequest::WriteBody 🌐 | confirmed | Thiscall on the enter-request packet: writes the framing through FUN_00aba910, then a u64 from +0x10 through FUN_008fd1a0 (two u32 words in order), a u32 from … |

**ClientHousingPacketExplosionResetRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5a30` | ClientHousingPacketExplosionResetRequest::ClientHousingPack… 🌐 | confirmed | Calls FUN_00aa4990(0x12) to build the family 127 header with sub-opcode 18, then stores ClientHousingPacketExplosionResetRequest::vftable. |
| `00aa8a00` | ClientHousingPacketExplosionResetRequest::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketInvitePlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa7bf0` | ClientHousingPacketInvitePlayer::ClientHousingPacketInviteP… 🌐 | confirmed | Calls FUN_00aa4990(0xf) to build the family 127 header with sub-opcode 15, stores ClientHousingPacketInvitePlayer::vftable and copies four default dwords from … |
| `00aa7c70` | ClientHousingPacketInvitePlayer::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketLeaveHouse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4ac0` | ClientHousingPacketLeaveHouse::ClientHousingPacketLeaveHouse 🌐 | confirmed | Calls FUN_00aa4990(10) to build the family 127 header with sub-opcode 10, then stores ClientHousingPacketLeaveHouse::vftable. |
| `00aa7a30` | ClientHousingPacketLeaveHouse::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketLockHouse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa7dc0` | ClientHousingPacketLockHouse::ClientHousingPacketLockHouse 🌐 | confirmed | Calls FUN_00aa4990(0x16) to build the family 127 header with sub-opcode 22, stores ClientHousingPacketLockHouse::vftable, copies two default dwords from DAT_01… |
| `00aa7e30` | ClientHousingPacketLockHouse::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketPayUpkeep** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4b70` | ClientHousingPacketPayUpkeep::ClientHousingPacketPayUpkeep 🌐 | confirmed | Calls FUN_00aa4990(7) to build the family 127 header with sub-opcode 7, then stores ClientHousingPacketPayUpkeep::vftable. |
| `00aa7aa0` | ClientHousingPacketPayUpkeep::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketPickupAllFixtures** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5290` | ClientHousingPacketPickupAllFixtures::ClientHousingPacketPi… 🌐 | confirmed | Calls FUN_00aa4990(4) to build the family 127 header with sub-opcode 4, then stores ClientHousingPacketPickupAllFixtures::vftable. |
| `00aa8270` | ClientHousingPacketPickupAllFixtures::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketPickupFixture** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa81a0` | ClientHousingPacketPickupFixture::ClientHousingPacketPickup… 🌐 | confirmed | Calls FUN_00aa4990(3) to build the family 127 header with sub-opcode 3, then stores ClientHousingPacketPickupFixture::vftable. |
| `00aa8200` | ClientHousingPacketPickupFixture::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketPlaceFixture** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa53f0` | ClientHousingPacketPlaceFixture::ClientHousingPacketPlaceFi… 🌐 | confirmed | Calls FUN_00aa4990(2) to build family 127 with inner subtype 2, stores ClientHousingPacketPlaceFixture::vftable, then zeroes the embedded dword at +0xc through… |
| `00aa8350` | ClientHousingPacketPlaceFixture::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abc9a0` | ClientHousingPacketPlaceFixture::WriteBody 🌐 | likely | Thiscall on a ClientHousingPacketPlaceFixture object: writes the BaseHousingPacket framing, then a 4-byte value from +0xc, two 16-byte groups from +0x10 and +0… |

**ClientHousingPacketPlaceFixtureRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5340` | ClientHousingPacketPlaceFixtureRequest::ClientHousingPacket… 🌐 | confirmed | Calls FUN_00aa4990(1) to build family 127 with inner subtype 1, stores ClientHousingPacketPlaceFixtureRequest::vftable, then zeroes the embedded dword at +0xc … |
| `00aa82e0` | ClientHousingPacketPlaceFixtureRequest::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketPreviewByItemDefinitionIdRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa54f0` | ClientHousingPacketPreviewByItemDefinitionIdRequest::Ctor 🌐 | confirmed | Calls FUN_00aa4990(0x14) to build the family 127 header with sub-opcode 20, stores ClientHousingPacketPreviewByItemDefinitionIdRequest::vftable and zeroes the … |
| `00aa84a0` | ClientHousingPacketPreviewByItemDefinitionIdRequest::Scalar… | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketRemoveCustomizationFromFixtureGroupAndType** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3e70` | ClientHousingPacketRemoveCustomizationFromFixtureGroupAndTy… 🌐 | confirmed | Constructs subtype 60, installs its request packet vtable and initializes two fixed 64-byte string fields. |
| `00ab3ee0` | ClientHousingPacketRemoveCustomizationFromFixtureGroupAndTy… | confirmed | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |

**ClientHousingPacketRequestGrant** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4fd0` | ClientHousingPacketRequestGrant::ClientHousingPacketRequest… 🌐 | confirmed | Calls FUN_00aa4990(0x17) to build the family 127 header with sub-opcode 23, then stores ClientHousingPacketRequestGrant::vftable. |
| `00aa7f80` | ClientHousingPacketRequestGrant::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketRequestName** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aad540` | ClientHousingPacketRequestName::ctor 🌐 | confirmed | Constructor: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x11 (17, base opcode 127), stamps ClientHousingPacketRequestName::vfta… |
| `00aad5b0` | ClientHousingPacketRequestName::~ClientHousingPacketRequest… | confirmed | Destructor: resets the vtable to ClientHousingPacketRequestName::vftable, releases the SoeUtil::IString<char> member at +0xc (refcount decrement, release throu… |
| `00aad640` | ClientHousingPacketRequestName::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the class destructor FUN_00aad5b0 and frees the object through _free when the low bit of param_2 is set. |

**ClientHousingPacketRequestPlayerHouses** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3f00` | ClientHousingPacketRequestPlayerHouses::Ctor 🌐 | confirmed | Constructs subtype 14, installs the request packet vtable, initializes two words and assigns its default string member. |
| `00ab3f90` | ClientHousingPacketRequestPlayerHouses::ScalarDeletingDtor | confirmed | Calls the request packet destructor and conditionally frees the object when the deleting flag's low bit is set. |
| `00aad310` | ClientHousingPacketRequestPlayerHouses::~ClientHousingPacke… | likely | Destructor of the request-player-houses packet: releases the SoeUtil::IString<char> member at +0x18 (vtable reset, refcount decrement, release through vtable s… |

**ClientHousingPacketRequestVisitToNeighbor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa8510` | ClientHousingPacketRequestVisitToNeighbor::ClientHousingPac… 🌐 | confirmed | Calls FUN_00aa4990(0x15) to build the family 127 header with sub-opcode 21, stores ClientHousingPacketRequestVisitToNeighbor::vftable and copies two default dw… |
| `00aa8580` | ClientHousingPacketRequestVisitToNeighbor::ScalarDeletingDt… | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketSaveFixture** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aad140` | ClientHousingPacketSaveFixture::ctor 🌐 | confirmed | Constructor of the save-fixture request: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 5 (base opcode 127), stamps ClientHousingPa… |

**ClientHousingPacketSetEditMode** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4cc0` | ClientHousingPacketSetEditMode::ClientHousingPacketSetEditM… 🌐 | confirmed | Calls FUN_00aa4990(6) to build the family 127 header with sub-opcode 6, then stores ClientHousingPacketSetEditMode::vftable. |
| `00aa7ce0` | ClientHousingPacketSetEditMode::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abcb30` | ClientHousingPacketSetEditMode::WriteBody 🌐 | confirmed | Thiscall on the edit-mode packet: writes the framing through FUN_00aba910, then one u8 from +0xc (the edit-mode flag) into the writer argument. |

**ClientHousingPacketToggleFloraAllowed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4e70` | ClientHousingPacketToggleFloraAllowed::ClientHousingPacketT… 🌐 | confirmed | Calls FUN_00aa4990(0xc) to build the family 127 header with sub-opcode 12, then stores ClientHousingPacketToggleFloraAllowed::vftable. |
| `00aa7ea0` | ClientHousingPacketToggleFloraAllowed::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketToggleLocked** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4d70` | ClientHousingPacketToggleLocked::ClientHousingPacketToggleL… 🌐 | confirmed | Calls FUN_00aa4990(0xb) to build the family 127 header with sub-opcode 11, then stores ClientHousingPacketToggleLocked::vftable. |
| `00aa7d50` | ClientHousingPacketToggleLocked::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**ClientHousingPacketTogglePetAutospawn** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa4f20` | ClientHousingPacketTogglePetAutospawn::ClientHousingPacketT… 🌐 | confirmed | Calls FUN_00aa4990(0xd) to build the family 127 header with sub-opcode 13, then stores ClientHousingPacketTogglePetAutospawn::vftable. |
| `00aa7f10` | ClientHousingPacketTogglePetAutospawn::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketDeclineNotification** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3c60` | HousingPacketDeclineNotification::Ctor 🌐 | confirmed | Constructs housing subtype 48, installs its vtable and initializes fixed-width embedded payload fields. |
| `00ab3cc0` | HousingPacketDeclineNotification::ScalarDeletingDtor | confirmed | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |

**HousingPacketExplosion** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa88c0` | HousingPacketExplosion::HousingPacketExplosion 🌐 | confirmed | Calls FUN_00aa4990(0x37) to build the family 127 header with sub-opcode 55, then stores HousingPacketExplosion::vftable. |
| `00aa8920` | HousingPacketExplosion::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketExplosionReset** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5980` | HousingPacketExplosionReset::HousingPacketExplosionReset 🌐 | confirmed | Calls FUN_00aa4990(0x38) to build the family 127 header with sub-opcode 56, then stores HousingPacketExplosionReset::vftable. |
| `00aa8990` | HousingPacketExplosionReset::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketFixtureAsset** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3fb0` | HousingPacketFixtureAsset::Ctor 🌐 | confirmed | Constructs subtype 42, installs its vtable, initializes fixed strings and a strong-type array, and stores a supplied word. |
| `00ab4080` | HousingPacketFixtureAsset::ScalarDeletingDtor | confirmed | Runs packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |

**HousingPacketFixtureItemList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3ce0` | HousingPacketFixtureItemList::Ctor 🌐 | confirmed | Constructs subtype 43, stores two arguments, installs the packet vtable and initializes an embedded list. |
| `00ab7360` | HousingPacketFixtureItemList::ScalarDeletingDtor | confirmed | Calls the packet destructor, then conditionally frees the object when the deleting flag's low bit is set. |
| `00aae640` | HousingPacketFixtureItemList::ReleaseGuidArray | likely | Releases the embedded SoeUtil::Array<StrongType<unsigned __int64,SoeUtilSignatureCharacterGuid,0>,0,1> member at object+0x14: resets the member vtable, clears … |
| `00ab3d60` | HousingPacketFixtureItemList::Dtor | likely | Destroys the embedded list, then restores the BaseHousingPacket and BasePacket vtables during cleanup. |

**HousingPacketFixturePlacementDenied** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aad660` | HousingPacketFixturePlacementDenied::ctor 🌐 | confirmed | Constructor of the placement-denied packet: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x35 (53, base opcode 127), stamps Housi… |
| `00aad6d0` | HousingPacketFixturePlacementDenied::~HousingPacketFixtureP… | likely | Destructor: releases the SoeUtil::IString<char> member at +0xc (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtabl… |

**HousingPacketFixtureRemove** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa80d0` | HousingPacketFixtureRemove::HousingPacketFixtureRemove 🌐 | confirmed | Calls FUN_00aa4990(0x29) to build the family 127 header with sub-opcode 41 and stores HousingPacketFixtureRemove::vftable. |
| `00aa8130` | HousingPacketFixtureRemove::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketFixtureUpdate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aad200` | HousingPacketFixtureUpdate::ctor 🌐 | confirmed | Constructor of the fixture-update packet: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x28 (40, base opcode 127), stores the thr… |
| `00aad290` | HousingPacketFixtureUpdate::~HousingPacketFixtureUpdate | likely | Destructor of the fixture-update packet: clears the dword at +0x24, releases the SoeUtil::Array<StrongType<unsigned __int64,SoeUtilSignatureCharacterGuid,0>,0,… |

**HousingPacketHousingError** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa56b0` | HousingPacketHousingError::HousingPacketHousingError 🌐 | confirmed | Calls FUN_00aa4990(0x2e) to build the family 127 header with sub-opcode 46, stores HousingPacketHousingError::vftable and zeroes the dword at object+0xc. |
| `00aa8660` | HousingPacketHousingError::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketInstanceData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa55f0` | HousingPacketInstanceData::HousingPacketInstanceData 🌐 | confirmed | Calls FUN_00aa4990(0x1c) to build the family 127 header with sub-opcode 28, stores HousingPacketInstanceData::vftable and writes the constructor argument into … |
| `00aa85f0` | HousingPacketInstanceData::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketInviteNotification** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab3be0` | HousingPacketInviteNotification::Ctor 🌐 | confirmed | Constructs housing subtype 47, installs the invite-notification vtable, and initializes the embedded payload fields. |
| `00ab3c40` | HousingPacketInviteNotification::ScalarDeletingDtor | confirmed | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |

**HousingPacketNotifyHouseAdded** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa86d0` | HousingPacketNotifyHouseAdded::HousingPacketNotifyHouseAdded 🌐 | confirmed | Calls FUN_00aa4990(0x31) to build the family 127 header with sub-opcode 49, stores HousingPacketNotifyHouseAdded::vftable, zeroes the dword at +0xc and copies … |
| `00aa8750` | HousingPacketNotifyHouseAdded::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketSetHeadSize** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa57b0` | HousingPacketSetHeadSize::HousingPacketSetHeadSize 🌐 | confirmed | Calls FUN_00aa4990(0x32) to build the family 127 header with sub-opcode 50, stores HousingPacketSetHeadSize::vftable and sets the dword at object+0xc to 10. |
| `00aa87c0` | HousingPacketSetHeadSize::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketUpdateFixturePosition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5860` | HousingPacketUpdateFixturePosition::~HousingPacketUpdateFix… | confirmed | Stores HousingPacketUpdateFixturePosition::vftable, then BaseHousingPacket::vftable, then BasePacket::vftable into the object. |
| `00aa58c0` | HousingPacketUpdateFixturePosition::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00aa8830` | HousingPacketUpdateFixturePosition::HousingPacketUpdateFixt… 🌐 | confirmed | Calls FUN_00aa4990(0x33) to build the family 127 header with sub-opcode 51, stores HousingPacketUpdateFixturePosition::vftable and copies two groups of four de… |

**HousingPacketUpdateHouseInfo** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5080` | HousingPacketUpdateHouseInfo::HousingPacketUpdateHouseInfo 🌐 | confirmed | Calls FUN_00aa4990(0x2c) to build the family 127 header with sub-opcode 44, stores HousingPacketUpdateHouseInfo::vftable, and sets the flag bytes at +0xc..+0xf… |
| `00aa7ff0` | HousingPacketUpdateHouseInfo::ScalarDeletingDtor | confirmed | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |

**HousingPacketZoneData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aacdc0` | ZoneDataPayloadInit | likely | Initialises the zone-data payload: takes four dwords from globals at +0/+4/+8/+0xc, zeroes dwords at +0x10/+0x34/+0x38 and bytes at +0x48/+0x49/+0x4a, construc… |
| `00aad3a0` | ZoneDataPayloadRelease | likely | Releases the five SoeUtil::IString<char> members of the zone-data payload at +0x14, +0x24, +0x4c, +0x5c and +0x6c: for each it resets the member vtable, and wh… |
| `00aad4e0` | HousingPacketZoneData::~HousingPacketZoneData | likely | Destructor of the zone-data packet: releases the five embedded SoeUtil::IString<char> members through FUN_00aad3a0 and then resets the object's vtable to BaseH… |

**HousingProcessor** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab8ee0` | HousingProcessor::HandleSubtype44 🌐 | confirmed | Thiscall (this = HousingProcessor, body pointer, length). Checks the body bounds (FUN_00ab2d70), decodes a fixed house-info body through FUN_00ab0a80 into the … |
| `00ab9080` | HousingProcessor::HandleSubtype46 🌐 | confirmed | Thiscall (this = HousingProcessor, body pointer, length). Reads one 32-bit value through FUN_00ab6950 and rejects bodies that are not fully consumed. For a cle… |
| `00ac2a10` | HousingProcessor::HandleInstanceList 🌐 | confirmed | Parses HousingPacketInstanceList through FUN_00abf210 and compares its owner u64 with the client guid. For the client's own list it builds and sends RatingPack… |
| `00ac42d0` | HousingProcessor::PickUpSelectedFixture 🌐 | confirmed | Pickup callback: looks the selection up by its two words through FUN_0105bfa0 and, when the object is found, sets mode 3 in this+0x4140, stores the selection w… |
| `00aca5d0` | HousingProcessor::DispatchMessage 🌐 | confirmed | Virtual method of HousingProcessor: constructs the housing packet wrapper (FUN_00aa4930, reset to BasePacket::vftable at exit) over the caller buffer (param_2,… |
| `00abae70` | HousingProcessor::HandleSubtype41 🌐 | likely | When this+0x41b0 is nonzero, constructs a local subtype-41 HousingPacketFixtureRemove and, for a non-null body accepted by FUN_00ab6900, calls FUN_00cbc630 and… |
| `00ac7f40` | HousingProcessor::HandleFixtureAsset 🌐 | likely | Handles housing subtype 42 from the dispatcher and returns if this+0x41b0 is null. It allocates a record, initializes HousingPacketFixtureAsset with FUN_00ab3f… |

**(no class)** (38)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab6f40` | ReadHousingFixtureUpdateRecord | confirmed | Reads fixed scalar fields, three embedded records, variable strings and map data, byte ranges, and final scalar/boolean values from the bounded reader. |
| `00aba9e0` | ReadCountedDictionary 🌐 | confirmed | Drains the existing map list through FUN_00ab6d20, reads a 32-bit entry count, then processes that many entries from the bounded reader. At each loop start it … |
| `00ac3fe0` | SendEnterRequestByGuid 🌐 | confirmed | Sender for housing subtype 19 (ClientHousingPacketEnterRequest) with an explicit house guid: constructs the packet with FUN_00aa83c0, writes param_2/param_3 as… |
| `00ac40b0` | SendEnterRequestWithDefaultGuid 🌐 | confirmed | Sender for housing subtype 19 (ClientHousingPacketEnterRequest) that copies the two .data globals DAT_01be7be0/DAT_01be7be4 into the packet's u64 guid field an… |
| `00ac4160` | SendSetEditModeToggle 🌐 | confirmed | Sender for housing subtype 6 (ClientHousingPacketSetEditMode): reads the current edit-mode byte at this+0x16, and when it is 1 first calls FUN_00ab0a80(0), the… |
| `00ac4200` | SendPlaceFixtureRequest 🌐 | confirmed | Sender for housing subtype 1 (ClientHousingPacketPlaceFixtureRequest) gated on this+0x41b0 != 0 (a loaded housing instance) and this+0x16 != 0 (edit mode): con… |
| `00ac4cd0` | SendInvitePlayer 🌐 | confirmed | Sender for housing subtype 15 (ClientHousingPacketInvitePlayer) gated on this+0x41b0 != 0: constructs the packet with FUN_00aa7bf0, stores param_2 and param_3 … |
| `00ac7af0` | SendRequestGrantOnce 🌐 | confirmed | One-shot property grant request: when the latch byte at this+0x2880 is clear it sets it to 1, constructs the payloadless ClientHousingPacketRequestGrant with F… |
| `00ac7ce0` | SendSaveFixture 🌐 | confirmed | Serialises and enqueues a ClientHousingPacketSaveFixture (subtype 5): it opens the shared transport scratch scope with FUN_008d4a40(0x10000000), runs the housi… |
| `00aacd30` | SaveFixturePayloadInit | likely | Initialises the payload record embedded in the save-fixture request: zeroes the dword at +0 and the dword at +0x24, and constructs SoeUtil::IString<char> membe… |
| `00ab3030` | ReadLengthPrefixedText | likely | Reads a signed length, verifies the requested byte span fits, advances the cursor and marks failure for an invalid span. |
| `00ab3440` | ZoneDataPayloadScalarDeletingDtor | likely | Releases zone-data payload strings and conditionally frees the object when the deleting flag's low bit is set. |
| `00ab3a40` | CopyZoneDataFields 🌐 | likely | Copies five leading words and an embedded field, then copies later scalar and byte fields from a parsed zone-data payload. |
| `00ab6c80` | ReadZoneDataFlagsAndWord 🌐 | likely | Reads two boolean bytes and a 32-bit value into offsets 0x0c, 0x0d and 0x10 from the bounded body reader. |
| `00ab7250` | ReadHousingInstanceList 🌐 | likely | Reads a signed entry count, appends one list node per entry and fills each node with two four-word blocks. |
| `00ab7f50` | BoundedBodyParseGate_via_00ab2c20 🌐 | likely | Returns 1 only when the param_1 object is non-null, param_2 is non-null, a bounded parse FUN_00ab2c20 over (param_2, param_3) sets no error byte, and the curso… |
| `00ab7fc0` | BoundedBodyParseGate_via_00ab2c80 🌐 | likely | Same shape as 00ab7f50 but parses through FUN_00ab2c80: returns 1 when the body is non-null, the parse sets no error and the cursor reached the end (or the fla… |
| `00ab8c50` | ReadCountedKeyedRecords 🌐 | likely | Thiscall on a table object with a bounded reader (param_2: cursor +8, end +0xc, error +0x10). Reads one 32-bit count; grows the table at this+0xd8 through FUN_… |
| `00ab8d10` | ReadCountedRecordList_ClearThenRead 🌐 | likely | Takes a bounded reader (param_1) and a map (param_2). First unlinks every node of the map list at +8 through FUN_00aac730 (HashTableRemoveNode, accepted label)… |
| `00ab8dc0` | HandleHousingSubtype51 🌐 | likely | Called with a body pointer (param_1) and length (param_2). Constructs a local through FUN_00aa8830, checks the body bounds with FUN_00ab2d10, looks up a world … |
| `00ab99a0` | HandleHousingPacketExplosion 🌐 | likely | Constructs a local HousingPacketExplosion through FUN_00aa88c0, then parses the non-null body with FUN_00ab3090. Only when the reader error flag is clear and i… |
| `00ac42a0` | SendPendingPlaceFixture 🌐 | likely | Small (48 byte) method: bytes 56 8b f1 80 be ec 40 00 00 00 74 22 8b 46 10 8b 88 80 02 00 00 85 c9 74 0e 6a 01 8d 86 f0 40 00 00 50 e8 e9 bc ff ff c6 86 ec 40 … |
| `00ac4400` | SendApplyFixtureCustomization 🌐 | likely | Sender for housing subtype 59 (ClientHousingPacketApplyCustomizationToFixtureGroupAndType): constructs the packet with FUN_00ab3dd0 (which builds two StringFix… |
| `00ac44e0` | SendRemoveFixtureCustomization 🌐 | likely | Sender for housing subtype 60 (ClientHousingPacketRemoveCustomizationFromFixtureGroupAndType): constructs the packet with FUN_00ab3e70 (two StringFixed<64> mem… |
| `00ac45b0` | SendPayUpkeep 🌐 | likely | Sender for housing subtype 7 (ClientHousingPacketPayUpkeep) with no payload fields: constructs the packet with FUN_00aa4b70 and sends it through FUN_00ac0970 w… |
| `00ac4940` | SendToggleLocked 🌐 | likely | Sender for housing subtype 11 (ClientHousingPacketToggleLocked) gated on this+0x41b0 != 0 (a loaded housing instance): constructs the packet with FUN_00aa4d70 … |
| `00ac49d0` | SendLockHouse 🌐 | likely | Sender for housing subtype 22 (ClientHousingPacketLockHouse): constructs the packet with FUN_00aa7dc0, stores param_2 and param_3 into its two leading words an… |
| `00ac4a70` | SendToggleFloraAllowed 🌐 | likely | Sender for housing subtype 12 (ClientHousingPacketToggleFloraAllowed) gated on this+0x41b0 != 0: constructs the packet with FUN_00aa4e70 and sends it through F… |
| `00ac4b00` | SendTogglePetAutospawn 🌐 | likely | Sender for housing subtype 13 (ClientHousingPacketTogglePetAutospawn): constructs the packet with FUN_00aa4f20 and sends it through FUN_00ac0ec0 when the conne… |
| `00ac4b80` | SendLeaveHouse 🌐 | likely | Sender for housing subtype 10 (ClientHousingPacketLeaveHouse): constructs the packet with FUN_00aa4ac0 and sends it through FUN_00ac0fd0 when the connected fla… |
| `00ac4c40` | SendChangeHouseName 🌐 | likely | When this+0x41b0 is non-null, constructs ClientHousingPacketChangeHouseName with FUN_00aa5140, stores param_2 as the packet's 32-bit value at +0xc, and sends t… |
| `00ac4e00` | SendDeclineInvite 🌐 | likely | Sender for housing subtype 16 (ClientHousingPacketDeclineInvite): constructs the packet with FUN_00aa7b10, stores param_2 and param_3 into its two leading word… |
| `00ac4e90` | SendRequestNameFromSelectedEntries 🌐 | likely | When this+0x14 is set, FUN_009d9c10 returns zero, param_2/param_3/param_4 are each in [0,this+0x28), and this+0x1a34 is non-zero, constructs ClientHousingPacke… |
| `00ac51b0` | SendRequestName 🌐 | likely | Second name-request sender: gated on this+0x14 != 0 and FUN_009d9c10() == 0, it constructs ClientHousingPacketRequestName with FUN_00aad540, assigns the single… |
| `00ac7a50` | SendRequestVisitToNeighbor 🌐 | likely | Sender for housing subtype 21 (ClientHousingPacketRequestVisitToNeighbor): constructs the packet with FUN_00aa8510, stores the client's own guid pair read from… |
| `00ac9a90` | ParseHousingFixtureItemListFields 🌐 | likely | Builds a bounded reader over [param_2,param_2+param_3), then reads three fields into destination offsets +0x0c, +0x10 and +0x14 through FUN_00ab3620, FUN_00abf… |
| `00ac9dd0` | ParseHousingInstanceRecord 🌐 | likely | Reads the bounded housing instance body into the house object: u64 at +0x28, u64 at +0x30, a length-prefixed string, u64 at +0x48, u32 at +0x54, a second lengt… |
| `00aca2b0` | ParseHouseInstanceBody 🌐 | likely | Builds a bounded reader over [param_2,param_2+param_3), runs FUN_00ac9dd0 on the destination, and returns 1 only when the reader has no error and either param_… |

### `net.packet.ingamepurchase`

**InGamePurchaseBuyBundleFromMerchantRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2a320` | BuyBundleFromMerchantRequest::Dtor | confirmed | Restores the request and string vtables, releases the string buffer when its shared reference count reaches zero, then restores BasePacket. |
| `00b2c2a0` | BuyBundleFromMerchantRequest::ScalarDeletingDtor | confirmed | Calls the request destructor and conditionally frees the packet object. |

**InGamePurchaseBuyBundleFromMerchantResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2a440` | BuyBundleFromMerchantResponse::Dtor | confirmed | Restores response and string vtables, releases the string buffer if unshared, then restores BasePacket. |
| `00b2c2c0` | BuyBundleFromMerchantResponse::ScalarDeletingDtor | confirmed | Calls the response destructor and conditionally frees the packet object. |

**InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1fe70` | InGamePurchaseUpdateMerchantSellVftableRestore | confirmed | Stores InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase::vftable at [param_1] and then BasePacket::vftable; those two stores are the whole bo… |

**InGamePurchaseUpdateSaleDisplay** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2c1a0` | UpdateSaleDisplay::Constructor 🌐 | confirmed | Initializes the purchase packet base with subtype 0x28, writes the update vtable, and initializes an empty SaleDisplayInfo list. |
| `00b2c210` | UpdateSaleDisplay::Dtor | confirmed | Restores the update and list vtables, drains the sale-display nodes, then restores BasePacket's vtable. |
| `00b2c280` | UpdateSaleDisplay::ScalarDeletingDtor | confirmed | Calls the update packet destructor and conditionally frees the object according to the deleting flag. |

**PacketInGamePurchasePlaceOrderClientTicket<GameCommerce::ClientInGamePurchaseOrder>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2a860` | PlaceOrderClientTicket::Dtor | confirmed | Releases the owned byte-array allocation and restores the BasePacket vtable. |
| `00b2a920` | PlaceOrderClientTicket::ScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object, and returns its pointer when not deleting. |
| `00b2a7f0` | PlaceOrderClientTicket::Constructor 🌐 | likely | Initializes the purchase base with subtype 0x24, stores the supplied field at offset 0x18, and initializes a byte-array member. |

**PacketInGamePurchasePreviewOrder<GameCommerce::ClientInGamePurchaseOrder>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2c610` | PreviewOrder::Constructor 🌐 | confirmed | Initializes the purchase packet base with subtype 1, stores the supplied field at offset 0x18, and initializes the order member. |
| `00b2c680` | PreviewOrder::ScalarDeletingDtor | confirmed | Calls the preview-order destructor and conditionally frees the packet object. |

**PacketInGamePurchaseStoreBundleContentResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2c120` | StoreBundleContentResponse::Constructor 🌐 | confirmed | Initializes the purchase base packet with subtype 0x1c, writes the response vtable, and initializes its inline data array. |
| `00b2c180` | StoreBundleContentResponse::ScalarDeletingDtor | confirmed | Calls the packet destructor and conditionally frees the object according to the deleting flag. |

**SoeUtil::List<InGamePurchaseUpdateItemRequirementsReply::UpdateData,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2c810` | UpdateItemRequirementsReplyDataList::Dtor | confirmed | Restores the update-data list vtable and drains its nodes. |
| `00b2cba0` | UpdateItemRequirementsReplyDataList::ScalarDeletingDtor | confirmed | Restores the list vtable, drains all records, and conditionally frees the list object. |

### `net.packet.inspect`

**BaseInspectPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3b00` | BaseInspectPacket::BaseInspectPacket 🌐 | confirmed | Inspect family constructor: installs BasePacket::vftable, stores 0x95 (149) in the packet id slot param_1[1], installs BaseInspectPacket::vftable and stores th… |
| `008a3b70` | BaseInspectPacket::~BaseInspectPacket | likely | Destructor body: writes BasePacket::vftable only; it is the last call of the inspect case in 009471a0 and is reached from several EH funclets (0157d740 and nei… |

**CommandPacketDirectInspect** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1abf0` | CommandPacketDirectInspect::ScalarDeletingDtor | confirmed | Resets the inherited command and packet vptrs, then frees the allocation when the deleting flag bit is set. |

**StartInspectPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d1860` | StartInspectPacket::StartInspectPacket 🌐 | confirmed | Constructor of the start-inspect packet: calls the inspect family constructor FUN_008a3b00 with sub-opcode 1, installs StartInspectPacket::vftable, stores a ze… |
| `008d18f0` | StartInspectPacket::~StartInspectPacket | likely | Destructor body of the start-inspect packet: rewrites the embedded SoeUtil::IString<char> member at +0x18 (param_1[6], buffer +0x1c, capacity +0x24) to its vta… |

**StopInspectPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3bd0` | StopInspectPacket::StopInspectPacket 🌐 | confirmed | Constructor: calls FUN_008a3b00(2) so the packet is inspect opcode 149 with sub-opcode 2, then installs StopInspectPacket::vftable; its only caller is 009347d0. |
| `008a3c30` | StopInspectPacket::~StopInspectPacket | likely | Destructor body: writes BasePacket::vftable only; its only caller is EH unwind funclet 015918c0. |

**WorldInspectReplyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b7790` | WorldInspectReplyPacket::WorldInspectReplyPacket | confirmed | Calls the inspect packet base constructor with 4, installs WorldInspectReplyPacket::vftable, and initializes field +0x18 to 0xffffffff. |
| `008b77f0` | WorldInspectReplyPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a3c80` | WorldInspectReplyPacket::~WorldInspectReplyPacket | likely | Destructor body: writes BasePacket::vftable only; 009471a0 destroys the reply object with it inside the branch that constructs WorldInspectReplyPacket (FUN_008… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009347d0` | StopInspect 🌐 | confirmed | If this+0xbdc is non-null, destroys and clears that object, constructs StopInspectPacket, then sends it when the gateway pointer at this+0x280 is non-null. |

### `net.packet.inventory`

**BaseInventoryPacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098ea30` | BaseInventoryPacket::Constructor | confirmed | Installs BasePacket, stores packet type 0x2a, installs BaseInventoryPacket, clears field +8, and returns. |
| `0098ea90` | BaseInventoryPacket::Destructor | confirmed | Restores BaseInventoryPacket then BasePacket vtables at object offset zero and returns. |
| `0098eae0` | BaseInventoryPacket::ScalarDeletingDestructor | confirmed | Restores BaseInventoryPacket and BasePacket vtables, frees this only when deleting flag bit 0 is set, and otherwise returns this. |
| `00bd5920` | BaseInventoryPacket::~BaseInventoryPacket | confirmed | Destructor body: stores BaseInventoryPacket::vftable then BasePacket::vftable at +0; no other work; returns nothing. |
| `00c09000` | BaseInventoryPacket_DtorBody | confirmed | Base-class destructor body: rewrites the vtable to BaseInventoryPacket::vftable then BasePacket::vftable; called from the inventory request's deleting destruct… |

**InventoryPacketCheckImmediateActivationItem** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c11320` | InventoryPacketCheckImmediateActivationItem::ScalarDeleting… | confirmed | Scalar deleting destructor (vtable slot 0 of InventoryPacketCheckImmediateActivationItem): resets the vtables to BaseInventoryPacket::vftable and BasePacket::v… |
| `00c10a50` | InventoryPacketCheckImmediateActivationItem::~InventoryPack… | likely | Destructor body: resets the object's vtables to BaseInventoryPacket::vftable and BasePacket::vftable and returns. |

**InventoryPacketItemActionBarAssign** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c14720` | InventoryPacketItemActionBarAssign::InventoryPacketItemActi… 🌐 | confirmed | Constructor taking a slot number and a pointer to the item instance: calls the inventory base constructor FUN_00a10400 with sub-opcode 6, stores InventoryPacke… |
| `00c1a850` | InventoryPacketItemActionBarAssign::ScalarDeletingDtor | confirmed | Resets the inherited inventory and packet vptrs, then frees the allocation when the deleting flag bit is set. |
| `00c14790` | InventoryPacketItemActionBarAssign::~InventoryPacketItemAct… | likely | Destructor body: resets the vtables to BaseInventoryPacket::vftable then BasePacket::vftable and returns. |

**InventoryPacketItemRequirementRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c096b0` | InventoryPacketItemRequirementRequest_ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of InventoryPacketItemRequirementRequest::vftable: rewrites the vtable to BaseInventoryPacket::vftable then BasePacket::vfta… |

**InventoryPacketMoveItemToStorage** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c147e0` | InventoryPacketMoveItemToStorage::~InventoryPacketMoveItemT… | confirmed | Destructor body: stores InventoryPacketMoveItemToStorage::vftable, then BaseInventoryPacket::vftable and finally BasePacket::vftable at *this and returns. |
| `00c14840` | InventoryPacketMoveItemToStorage::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of InventoryPacketMoveItemToStorage): resets the three vtables (own, BaseInventoryPacket, BasePacket) and frees the o… |

**InventoryPacketPreviewStyleCard** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd58c0` | InventoryPacketPreviewStyleCard::InventoryPacketPreviewStyl… | confirmed | Thiscall ctor (ECX = packet): calls the BaseInventoryPacket ctor 00a10400 with stack arg 0xb (sub-type 11), then stores InventoryPacketPreviewStyleCard::vftabl… |
| `00bd5c10` | InventoryPacketPreviewStyleCard::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor (virtual slot [0] of the preview-style-card packet): stores BaseInventoryPacket::vftable then BasePacket::vftable at +0, th… |

**InventoryPacketUseImmediateActivationItem** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c10940` | InventoryPacketUseImmediateActivationItem::InventoryPacketU… 🌐 | confirmed | Constructor: calls the inventory-packet base constructor FUN_00a10400 with sub-opcode 0xf (15), stores InventoryPacketUseImmediateActivationItem::vftable at *t… |
| `00c112b0` | InventoryPacketUseImmediateActivationItem::ScalarDeletingDt… | confirmed | Scalar deleting destructor (vtable slot 0 of InventoryPacketUseImmediateActivationItem): resets the vtables to BaseInventoryPacket::vftable and BasePacket::vft… |
| `00c109a0` | InventoryPacketUseImmediateActivationItem::~InventoryPacket… | likely | Destructor body: resets the object's vtables to BaseInventoryPacket::vftable and BasePacket::vftable and returns. |

**InventoryPacketUseStyleCard** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c10890` | InventoryPacketUseStyleCard::InventoryPacketUseStyleCard 🌐 | confirmed | Constructor: calls the inventory-packet base constructor FUN_00a10400 with sub-opcode 10 (0xa), stores InventoryPacketUseStyleCard::vftable at *this and return… |
| `00c11240` | InventoryPacketUseStyleCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of InventoryPacketUseStyleCard): resets the vtables to BaseInventoryPacket::vftable and BasePacket::vftable and frees… |
| `00c108f0` | InventoryPacketUseStyleCard::~InventoryPacketUseStyleCard | likely | Destructor body: resets the object's vtables to BaseInventoryPacket::vftable and then BasePacket::vftable and returns (all members are scalar, no releases). |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c20a30` | SerializeItemActionBarAssign 🌐 | confirmed | Writes two 16-bit packet header values, then serializes the fields at source offsets +0xc and +0x10 into the bounded writer. |
| `00c11600` | WriteUseStyleCardPacketBody 🌐 | likely | Body writer for the use-style-card request: initialises the caller's serialiser buffer with FUN_00a1ada0 and appends the packet's single 4-byte field at +0xc i… |
| `00c11660` | WriteUseImmediateActivationItemPacketBody | likely | Body writer for the immediate-activation request: initialises the caller's serialiser buffer with FUN_00a1ada0 and appends the packet's 4-byte field at +0xc th… |
| `00c116c0` | WriteCheckImmediateActivationItemPacketBody | likely | Body writer for the check-immediate-activation request: initialises the caller's serialiser buffer with FUN_00a1ada0 and appends the packet's 4-byte field at +… |
| `00c11780` | SendUseStyleCard 🌐 | likely | Serialises the use-style-card packet (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000)), writes the body with FUN_00c… |
| `00c11890` | SendUseImmediateActivationItem | likely | Serialises the immediate-activation packet into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000)), writes the body with FUN_00c1166… |
| `00c119a0` | SendCheckImmediateActivationItem | likely | Serialises the check-immediate-activation packet into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000)), writes the body with FUN_0… |

### `net.packet.job_customization`

**BeginJobCustimzationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4f50` | BeginJobCustimzationPacket::BeginJobCustimzationPacket 🌐 | confirmed | Constructor: calls FUN_008a4e90(1) so the packet is opcode 155 with sub-opcode 1, then installs BeginJobCustimzationPacket::vftable; SEH frame restores Excepti… |
| `008b8040` | BeginJobCustimzationPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4fb0` | BeginJobCustimzationPacket::~BeginJobCustimzationPacket | likely | Destructor body: writes BasePacket::vftable and returns; the 009471a0 router destroys the begin-customization packet with it at the end of the case, and funcle… |

**FinalizeJobCustomzationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a842a0` | FinalizeJobCustomzationPacketCtor 🌐 | confirmed | Constructor of the finalize-job-customization packet: runs the JobCustomizationBasePacket family constructor 008a4e90(2) (which installs BasePacket's vtable an… |
| `00a843f0` | FinalizeJobCustomzationPacketScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of FinalizeJobCustomzationPacket: rewrites the object's vtable to ClientGateway::BasePacket and frees it with _fre… |
| `00a84c00` | PackJobCustomizationPayload 🌐 | likely | Packs the caller packet into the bounded byte writer: calls 00a84a30 for the header, writes the dword at this+0xc, then writes 0x12 dwords from this+0x10 throu… |

**JobCustomizationBasePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4e90` | JobCustomizationBasePacket::JobCustomizationBasePacket 🌐 | confirmed | Job-customization family constructor: installs BasePacket::vftable, stores opcode 155 (0x9b) at +4, installs JobCustomizationBasePacket::vftable and stores the… |
| `008b7fe0` | JobCustomizationBasePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4f00` | JobCustomizationBasePacket::~JobCustomizationBasePacket | likely | Destructor body: writes BasePacket::vftable and returns; seven EH funclets (0157e110 and neighbours) call it. |

### `net.packet.lobby`

**BaseLobbyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3ce0` | BaseLobbyPacket::BaseLobbyPacket 🌐 | confirmed | Lobby packet-family constructor: installs BasePacket::vftable at +0, stores the base opcode 101 (0x65) at +4, installs BaseLobbyPacket::vftable, then stores th… |
| `008b7880` | BaseLobbyPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a3d50` | BaseLobbyPacket::~BaseLobbyPacket | likely | Destructor body restores BasePacket::vftable and returns. Dispatcher 00923210 invokes it for its packet temporary; that caller does not free the temporary here… |

**ClientLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb1890` | ClientLobby::SendJoinGame 🌐 | confirmed | Virtual method at ClientLobby::vftable slot 12: constructs a JoinLobbyGamePacket in a 20-byte stack buffer (008a3da0 with the dword at this+0x1a4), fetches the… |
| `00bb1920` | ClientLobby::SendLeaveGame 🌐 | confirmed | Virtual method at ClientLobby::vftable slot 13: builds a LeaveLobbyGamePacket in a 20-byte stack buffer (008a3e60 with this+0x1a4), then, when the client singl… |
| `00bb19b0` | ClientLobby::SendStartGame 🌐 | confirmed | Virtual method at ClientLobby::vftable slot 14: constructs a StartLobbyGamePacket in a 20-byte stack buffer (008a3f20 with this+0x1a4) and, when [[00769710()+4… |
| `00bb1a40` | ClientLobby::SendLeaveLobby 🌐 | likely | Virtual method at ClientLobby::vftable slot 16: constructs a LeaveLobbyPacket with id 0 (00bafe60, base opcode 101 sub-opcode 9), overwrites the packet dword a… |
| `00bb1cd0` | ClientLobby::SendCreateGame 🌐 | likely | Virtual method at ClientLobby::vftable slot 11: when the argument is non-zero it takes the client singleton (00769710), calls vtable slot 0x10 of the object re… |

**JoinLobbyGamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3da0` | JoinLobbyGamePacket::JoinLobbyGamePacket 🌐 | confirmed | Constructor for lobby sub-opcode 1: calls the opcode-101 family constructor FUN_008a3ce0(1), installs JoinLobbyGamePacket::vftable and stores the 2 payload wor… |
| `008b78e0` | JoinLobbyGamePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a3e10` | JoinLobbyGamePacket::~JoinLobbyGamePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**LeaveLobbyGamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3e60` | LeaveLobbyGamePacket::LeaveLobbyGamePacket 🌐 | confirmed | Constructor for lobby sub-opcode 2: calls the opcode-101 family constructor FUN_008a3ce0(2), installs LeaveLobbyGamePacket::vftable and stores the 2 payload wo… |
| `008b7940` | LeaveLobbyGamePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a3ed0` | LeaveLobbyGamePacket::~LeaveLobbyGamePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**LobbyCommand_AddBooster** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014cf050` | LobbyCommand_AddBooster::LobbyCommand_AddBooster | confirmed | Stores LobbyCommand_AddBooster::vftable at *this and zeroes +0x8 and +0xc. Returns this. |
| `014cf0d0` | LobbyCommand_AddBooster::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AddBooster' (mov eax, imm32; ret). |
| `014cf110` | LobbyCommand_AddBooster::~LobbyCommand_AddBooster | confirmed | Stores LobbyCommand_AddBooster::vftable at *this and calls 013b2820 for base cleanup. |
| `014cf2e0` | LobbyCommand_AddBooster::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8), writes 'Starting LobbyCommand_AddBooster' trace text through no-op 013c26e0, then 013b2960, writes '… |
| `014cf440` | LobbyCommand_AddBooster::DumpText | confirmed | Virtual slot [19]. Writes 'Add Boooster Lobby Command' with a newline (spelling as in the binary) through 01241650, then calls 013b2af0 with its argument. |
| `014cf470` | LobbyCommand_AddBooster::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Add Booster Lobby Command' (0x19 characters) through 01241650. |
| `014cf220` | LobbyCommand_AddBooster::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AddBooster (00d8d382); on failure asserts 'clone' through 012f5a60 (line 0x36) and copies this+0x… |
| `014cf280` | LobbyCommand_AddBooster::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10), then 013b2910; reads leagueID into +0x8 and the booster week into +0xc with 013c24d0, then calls vta… |
| `014cf360` | LobbyCommand_AddBooster::Factory | likely | Allocates 0x10 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AddBooster via 014cf050. Returns the object or 0. |
| `014cf3e0` | LobbyCommand_AddBooster::DeletingDtor | likely | Stores LobbyCommand_AddBooster::vftable at *this, calls 013b2820 for base cleanup, then frees the object when bit 0 of the flag argument is set (_free), and re… |

**LobbyCommand_AddBuddies** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014cebf0` | LobbyCommand_AddBuddies::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8), writes 'Starting LobbyCommand_AddBuddies' trace text through no-op 013c26e0, then 013b2960, writes '… |
| `014cec60` | LobbyCommand_AddBuddies::~LobbyCommand_AddBuddies | confirmed | Stores LobbyCommand_AddBuddies::vftable at *this, frees the list buffer at +0xc when non-null, zeroes +0xc..+0x14, and calls 013b2820 for base cleanup. |
| `014cecf0` | LobbyCommand_AddBuddies::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AddBuddies' (mov eax, imm32; ret). |
| `014ced30` | LobbyCommand_AddBuddies::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014cec60, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014cee10` | LobbyCommand_AddBuddies::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Add Buddies Lobby Command' (0x19 characters) through 01241650. |
| `014cee30` | LobbyCommand_AddBuddies::LobbyCommand_AddBuddies | confirmed | Stores LobbyCommand_AddBuddies::vftable at *this and zeroes the list header fields at +0xc, +0x10 and +0x14. |
| `014cef10` | LobbyCommand_AddBuddies::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AddBuddies (00d8d382); on failure asserts 'clone' through 012f5a60 (line 0x3e). After the cast an… |
| `014cef70` | LobbyCommand_AddBuddies::Factory | likely | Allocates 0x18 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AddBuddies via 014cee30. Returns the object or 0. |
| `014cf000` | LobbyCommand_AddBuddies::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10), then 013b2910; reads the integer list into +0x8 with 01300540, then calls vtable +0x14 (slot [5]). R… |

**LobbyCommand_AddIgnore** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ce770` | LobbyCommand_AddIgnore::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8) with the stream, writes 'Starting LobbyCommand_AddIgnore' trace text through no-op 013c26e0, then 013… |
| `014ce7e0` | LobbyCommand_AddIgnore::~LobbyCommand_AddIgnore | confirmed | Stores LobbyCommand_AddIgnore::vftable at *this, frees the buffer at +0xc when non-null, zeroes +0xc..+0x14, and calls 013b2820 for base cleanup. |
| `014ce870` | LobbyCommand_AddIgnore::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AddIgnore' (mov eax, imm32; ret). |
| `014ce8b0` | LobbyCommand_AddIgnore::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014ce7e0, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014ce990` | LobbyCommand_AddIgnore::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Add Ignore Lobby Command' (0x18 characters) through 01241650. |
| `014ce9b0` | LobbyCommand_AddIgnore::LobbyCommand_AddIgnore | confirmed | Stores LobbyCommand_AddIgnore::vftable at *this and zeroes the list header fields at +0xc, +0x10 and +0x14. |
| `014cea90` | LobbyCommand_AddIgnore::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AddIgnore (00d8d382); on failure asserts 'clone' through 012f5a60 (line 0x3e). After the cast and… |
| `014ceaf0` | LobbyCommand_AddIgnore::Factory | likely | Allocates 0x18 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AddIgnore via 014ce9b0. Returns the object or 0. |
| `014ceb80` | LobbyCommand_AddIgnore::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10), then 013b2910; reads the account id list into +0x8 with 01300540, then calls vtable +0x14 (slot [5])… |

**LobbyCommand_AddIgnoreByName** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ce350` | LobbyCommand_AddIgnoreByName::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8), writes 'Starting LobbyCommand_AddIgnoreByName' trace text through no-op 013c26e0, then 013b2960, wri… |
| `014ce3b0` | LobbyCommand_AddIgnoreByName::LobbyCommand_AddIgnoreByName | confirmed | Calls 013b2760 (base-object init), stores LobbyCommand_AddIgnoreByName::vftable at *this, and initialises the string header (capacity 15 at +0x20). |
| `014ce440` | LobbyCommand_AddIgnoreByName::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AddIgnoreByName' (mov eax, imm32; ret). |
| `014ce480` | LobbyCommand_AddIgnoreByName::~LobbyCommand_AddIgnoreByName | confirmed | Stores LobbyCommand_AddIgnoreByName::vftable at *this, frees the name buffer (pointer at +0xc) when the capacity field at +0x20 exceeds 15, resets the string h… |
| `014ce5f0` | LobbyCommand_AddIgnoreByName::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014ce480, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014ce690` | LobbyCommand_AddIgnoreByName::DumpText | confirmed | Virtual slot [19]. Writes 'Add Ignore By Name Lobby Command' with a newline through 01241650, calls 013b2af0 with its argument, then writes 'Ignore Name: %s' f… |
| `014ce730` | LobbyCommand_AddIgnoreByName::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Add Ignore By Name Lobby Command' (0x20 characters) through 01241650. |
| `014ce300` | LobbyCommand_AddIgnoreByName::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10), then 013b2910; reads the name string into +0x8 with 013c28d0, then calls vtable +0x14 (slot [5]). Re… |
| `014ce570` | LobbyCommand_AddIgnoreByName::Factory | likely | Allocates 0x24 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AddIgnoreByName via 014ce3b0. Returns the object or 0. |
| `014ce630` | LobbyCommand_AddIgnoreByName::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AddIgnoreByName (00d8d382); on failure asserts 'clone' through 012f5a60 (line 0x2c). Copies the n… |

**LobbyCommand_AddIgnoreByNameResponse** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014cdd80` | LobbyCommand_AddIgnoreByNameResponse::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8), writes trace text through no-op 013c26e0, then 013b2960, writes the account id (+0x8, 013c23f0), the… |
| `014cde30` | LobbyCommand_AddIgnoreByNameResponse::LobbyCommand_AddIgnor… | confirmed | Calls 013b2760 (base-object init), stores LobbyCommand_AddIgnoreByNameResponse::vftable at *this, and initialises the string and container headers of the objec… |
| `014cded0` | LobbyCommand_AddIgnoreByNameResponse::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AddIgnoreByNameResponse' (mov eax, imm32; ret). |
| `014cdf10` | LobbyCommand_AddIgnoreByNameResponse::~LobbyCommand_AddIgno… | confirmed | Stores LobbyCommand_AddIgnoreByNameResponse::vftable at *this, frees the string buffer at +0x30 when its capacity (+0x44) exceeds 15, resets it, frees the stri… |
| `014ce0a0` | LobbyCommand_AddIgnoreByNameResponse::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014cdf10, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014ce180` | LobbyCommand_AddIgnoreByNameResponse::DumpText | confirmed | Virtual slot [19]. Writes 'Add Ignore By Name Response Lobby Command' through 01241650, calls 013b2af0 with its argument, then writes the name, 'Results: %d' f… |
| `014ce2c0` | LobbyCommand_AddIgnoreByNameResponse::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Add Ignore By Name Response Lobby Command' (0x29 characters) through 01241650. |
| `014cdd00` | LobbyCommand_AddIgnoreByNameResponse::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10) with the stream, then 013b2910. Reads the account id into +0x8 (013c24d0), the name string into +0xc … |
| `014ce020` | LobbyCommand_AddIgnoreByNameResponse::Factory | likely | Allocates 0x48 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AddIgnoreByNameResponse via 014cde30. Returns the object or 0. |
| `014ce100` | LobbyCommand_AddIgnoreByNameResponse::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AddIgnoreByNameResponse (00d8d382); when the cast fails asserts 'clone' through 012f5a60 (line 0x… |

**LobbyCommand_AddLeagueCards** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014cd510` | LobbyCommand_AddLeagueCards::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8), passes the 'Starting ...' text to no-op 013c26e0, writes leagueID (+0x8) with 013c23f0, writes the o… |
| `014cd580` | LobbyCommand_AddLeagueCards::~LobbyCommand_AddLeagueCards | confirmed | Stores LobbyCommand_AddLeagueCards::vftable at *this, calls 012c2de0 (member cleanup) and 013b2820 (base cleanup). |
| `014cd600` | LobbyCommand_AddLeagueCards::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AddLeagueCards' (mov eax, imm32; ret). |
| `014cd640` | LobbyCommand_AddLeagueCards::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014cd580, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014cd8d0` | LobbyCommand_AddLeagueCards::DumpText | confirmed | Virtual slot [19]. Writes the literal 'Add League Cards Lobby Command' followed by a newline through 01241650, then calls 013b2af0 with its argument. |
| `014cd900` | LobbyCommand_AddLeagueCards::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Add League Cards Lobby Command' (0x1e characters) through 01241650. |
| `014cd920` | LobbyCommand_AddLeagueCards::LobbyCommand_AddLeagueCards | confirmed | Calls 013b2760 (base-object init), stores LobbyCommand_AddLeagueCards::vftable at *this, and zeroes +0x8 and +0x10, +0x14, +0x18. |
| `014cd660` | LobbyCommand_AddLeagueCards::ApplyLeagueCards | likely | Virtual slot [15]. Gets a singleton through 012adee0 (asserts non-null with file LobbyCommandAddLeagueCards.cpp, line 0x2f), looks up the key 'leagueCollection… |
| `014cda00` | LobbyCommand_AddLeagueCards::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AddLeagueCards (00d8d382); on failure asserts 'clone' through 012f5a60 (line 0x5e). Copies this+0… |
| `014cda60` | LobbyCommand_AddLeagueCards::Factory | likely | Allocates 0x1c bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AddLeagueCards via 014cd920. Returns the object or 0. |
| `014cdb50` | LobbyCommand_AddLeagueCards::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10), then 013b2910; reads leagueID into +0x8 with 013c24d0, reads the object at +0xc with 013d53d0, and c… |

**LobbyCommand_AdjournedGame** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014cce70` | LobbyCommand_AdjournedGame::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8) on the stream, passes the 'Starting ...' text to the no-op 013c26e0, writes GameID (+0x8) and MatchID… |
| `014ccf00` | LobbyCommand_AdjournedGame::~LobbyCommand_AdjournedGame | confirmed | Stores LobbyCommand_AdjournedGame::vftable at *this, frees the buffer at +0x14 when non-null, zeroes +0x14..+0x1c, and calls 013b2820 for base cleanup. |
| `014ccf90` | LobbyCommand_AdjournedGame::GetTypeName | confirmed | Virtual slot [11]. Returns the string literal 'LobbyCommand_AdjournedGame' (mov eax, imm32; ret). |
| `014ccfd0` | LobbyCommand_AdjournedGame::DeletingDtor | confirmed | Virtual slot [0]. Calls destructor body 014ccf00, frees the object when bit 0 of its flag argument is set (_free), and returns this. |
| `014cd140` | LobbyCommand_AdjournedGame::DumpLobbyCommandName | confirmed | Virtual slot [20]. Writes the literal 'Adjourned Game Lobby Command' (0x1c characters) through 01241650. |
| `014cd160` | LobbyCommand_AdjournedGame::LobbyCommand_AdjournedGame | confirmed | Calls 013b2760 (base-object init), stores LobbyCommand_AdjournedGame::vftable at *this, and zeroes +0x8, +0xc, +0x14, +0x18 and +0x1c. |
| `014ccff0` | LobbyCommand_AdjournedGame::DumpAccounts | likely | Virtual slot [19]. Writes 'Adjourned Game Account Command' through 01241650, then formats 'MatchID: %d' from +0xc and 'GameID: %d' from +0x8 with _sprintf and … |
| `014cd390` | LobbyCommand_AdjournedGame::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_AdjournedGame (00d8d382); when the cast fails asserts 'clone' through 012f5a60 (line 0x4b). Copie… |
| `014cd3f0` | LobbyCommand_AdjournedGame::Factory | likely | Allocates 0x20 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_AdjournedGame via 014cd160. Returns the object or 0. |
| `014cd480` | LobbyCommand_AdjournedGame::DeserializeBody | likely | Virtual slot [9]. Calls base hook slot [4] (vtable +0x10) with the stream, then 013b2910; reads GameID into +0x8 and MatchID into +0xc with 013c24d0, reads the… |

**LobbyCommand_BulkJoin** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ca7e0` | LobbyCommand_BulkJoin::DumpText | confirmed | Virtual slot 19, 405 bytes. Writes 'Bulk Join Lobby Command' plus newline (24 bytes) through 01241650, calls 013b2af0(param_2), formats 'Group ID: %d' from thi… |
| `014ca980` | LobbyCommand_BulkJoin::WriteCommandName | confirmed | Virtual slot 20. Loads its stack argument into ECX, pushes the 23-byte literal 'Bulk Join Lobby Command', calls 01241650 with that receiver, and returns with r… |
| `014cbfa0` | LobbyCommand_BulkJoin::~LobbyCommand_BulkJoin | confirmed | Sets the vtable to LobbyCommand_BulkJoin::vftable. Calls vtable [0] with arg 1 on each non-null object pointer in the vector at +0x5c/+0x60, then frees the buf… |
| `014cc1e0` | LobbyCommand_BulkJoin::GetTypeName | confirmed | Virtual slot [11] of LobbyCommand_BulkJoin. Returns the string literal 'LobbyCommand_BulkJoin' (mov eax, imm32; ret). |
| `014cc360` | LobbyCommand_BulkJoin::DeletingDtor | confirmed | Virtual slot [0] of LobbyCommand_BulkJoin. Calls destructor body 014cbfa0, then frees the object when bit 0 of the flag argument is set (_free) and returns thi… |
| `014ccbc0` | LobbyCommand_BulkJoin::SerializeBody | confirmed | Virtual slot [10]. Calls base hook slot [2] (vtable +0x8) on the stream, then 013b2960, and writes five values (1..5 through 0130c4f0 and 012fbec0), one intege… |
| `014cc430` | LobbyCommand_BulkJoin::Factory | likely | Allocates 0x78 bytes via 00d833a6 and, when non-null, constructs LobbyCommand_BulkJoin in it via 014cc2a0. Returns the object, or 0 when allocation fails. |
| `014cc5f0` | LobbyCommand_BulkJoin::CloneInto | likely | Virtual slot [18]. Dynamic-casts the argument to LobbyCommand_BulkJoin (00d8d382 with NetworkCommand and LobbyCommand_BulkJoin RTTI descriptors); when the cast… |

**LobbyCommand_BuyLeagueActions** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c89c0` | LobbyCommand_BuyLeagueActions::DtorBody | confirmed | Destructor body. Stores LobbyCommand_BuyLeagueActions::vftable at this, frees the buffer at this+0x14 when non-null and zeroes this+0x14, +0x18 and +0x1c, then… |
| `014c8a50` | LobbyCommand_BuyLeagueActions::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_BuyLeagueActions' (bytes b8 imm32 c3). |
| `014c8a90` | LobbyCommand_BuyLeagueActions::SerializeFields | confirmed | Virtual slot 10. Calls base virtual vtable+8 on param_2, writes 'Starting LobbyCommand_BuyLeagueActions' with 013c26e0, calls 013b2960, writes 'leagueID ' and … |
| `014c8b20` | LobbyCommand_BuyLeagueActions::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls the dtor body 014c89c0, then frees this when bit 0 of the flag argument is set (ret 4), and returns this. |
| `014c8b40` | LobbyCommand_BuyLeagueActions::DumpHeader | confirmed | Virtual slot 19, stdcall with one stack argument (the sink, ret 4). Writes 'Buy League Actions Lobby Command' plus newline (33 bytes) through 01241650, then ca… |
| `014c8b70` | LobbyCommand_BuyLeagueActions::WriteCommandName | confirmed | Virtual slot 20. Loads its stack argument into ECX, pushes the 32-byte literal 'Buy League Actions Lobby Command', calls 01241650 with that receiver, and retur… |
| `014c8b90` | LobbyCommand_BuyLeagueActions::Ctor | confirmed | Constructor. Calls 013b2760 (base constructor), stores LobbyCommand_BuyLeagueActions::vftable at this, zeroes this+8, +0xc, +0x14, +0x18, +0x1c and +0x20, and … |
| `014c8c60` | LobbyCommand_BuyLeagueActions::Clone | confirmed | Virtual slot 17. Allocates 0x24 bytes (00d833a6); when non-null constructs them with 014c8b90 and returns the object, else 0. Then calls this->vtable[0x48] (sl… |
| `014c8cd0` | LobbyCommand_BuyLeagueActions::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to BuyLeagueActions) through 00d8d382; on failure asserts 'clone' with LobbyCommandBuyLeagueActions.cpp … |

**LobbyCommand_ChallengeLeaguePlayer** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c8420` | LobbyCommand_ChallengeLeaguePlayer::Ctor | confirmed | Constructor. Calls 013b2760 (base constructor), stores LobbyCommand_ChallengeLeaguePlayer::vftable at this, zeroes this+8, +0xc, +0x10 and the byte at +0x14, a… |
| `014c84a0` | LobbyCommand_ChallengeLeaguePlayer::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_ChallengeLeaguePlayer' (bytes b8 imm32 c3). |
| `014c84e0` | LobbyCommand_ChallengeLeaguePlayer::DtorBody | confirmed | Destructor body without a free. Stores LobbyCommand_ChallengeLeaguePlayer::vftable at this and calls 013b2820 (base cleanup), then returns. |
| `014c8640` | LobbyCommand_ChallengeLeaguePlayer::Clone | confirmed | Virtual slot 17. Allocates 0x18 bytes (00d833a6); when non-null constructs them with 014c8420 and returns the object, else 0. Then calls this->vtable[0x48] (sl… |
| `014c86b0` | LobbyCommand_ChallengeLeaguePlayer::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to ChallengeLeaguePlayer) through 00d8d382; on failure asserts 'clone' with LobbyCommandChallengeLeagueP… |
| `014c87a0` | LobbyCommand_ChallengeLeaguePlayer::SerializeFields | confirmed | Virtual slot 10. Calls base virtual vtable+8 on param_2, writes 'Starting LobbyCommand_ChallengeLeaguePlayer' with 013c26e0, calls 013b2960, writes 'leagueID '… |
| `014c88a0` | LobbyCommand_ChallengeLeaguePlayer::ScalarDeletingDtor | confirmed | Virtual slot 0, combined destructor: stores LobbyCommand_ChallengeLeaguePlayer::vftable at this, calls 013b2820 (base cleanup), frees this when bit 0 of the fl… |
| `014c8900` | LobbyCommand_ChallengeLeaguePlayer::DumpHeader | confirmed | Virtual slot 19, stdcall with one stack argument (the sink, ret 4). Writes 'Challenge League Player Lobby Command' plus newline (38 bytes) through 01241650 wit… |
| `014c8930` | LobbyCommand_ChallengeLeaguePlayer::WriteCommandName | confirmed | Virtual slot 20. Loads its stack argument into ECX, pushes the 37-byte literal 'Challenge League Player Lobby Command', calls 01241650 with that receiver, and … |

**LobbyCommand_ChallengeLeaguePlayerResponse** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c7e90` | LobbyCommand_ChallengeLeaguePlayerResponse::SerializeFields | confirmed | Virtual slot 10. Calls base virtual vtable+8 on param_2, writes 'Starting LobbyCommand_ChallengeLeaguePlayerResponse' with 013c26e0, calls 013b2960, writes 'le… |
| `014c8040` | LobbyCommand_ChallengeLeaguePlayerResponse::Ctor | confirmed | Constructor. Calls 013b2760 (base constructor), stores LobbyCommand_ChallengeLeaguePlayerResponse::vftable at this, makes the empty std::string at this+0x1c (f… |
| `014c80e0` | LobbyCommand_ChallengeLeaguePlayerResponse::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_ChallengeLeaguePlayerResponse' (6 bytes: mov eax, imm32; ret). |
| `014c8120` | LobbyCommand_ChallengeLeaguePlayerResponse::DtorBody | confirmed | Destructor body. Stores LobbyCommand_ChallengeLeaguePlayerResponse::vftable at this, calls slot 0 of the object at this+0x34 with flag 1 when it is non-null an… |
| `014c81b0` | LobbyCommand_ChallengeLeaguePlayerResponse::Clone | confirmed | Virtual slot 17. Allocates 0x3c bytes (00d833a6); when non-null constructs them with 014c8040 and returns the object, else 0. Then calls this->vtable[0x48] (sl… |
| `014c82a0` | LobbyCommand_ChallengeLeaguePlayerResponse::ScalarDeletingD… | confirmed | Virtual slot 0. Calls the dtor body 014c8120, then frees this when bit 0 of the flag argument is set (ret 4), and returns this. |
| `014c8330` | LobbyCommand_ChallengeLeaguePlayerResponse::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to ChallengeLeaguePlayerResponse) through 00d8d382; on failure asserts 'clone' with LobbyCommandChalleng… |
| `014c83d0` | LobbyCommand_ChallengeLeaguePlayerResponse::DumpHeader | confirmed | Virtual slot 19, stdcall with one stack argument (the sink, ret 4). Writes 'Challenge League Player Response Lobby Command' plus newline (47 bytes) through 012… |
| `014c8400` | LobbyCommand_ChallengeLeaguePlayerResponse::WriteCommandName | confirmed | Virtual slot 20. Loads its stack argument into ECX, pushes the 46-byte literal 'Challenge League Player Response Lobby Command', calls 01241650 with that recei… |

**LobbyCommand_ChangeBulkStatus** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c6e80` | LobbyCommand_ChangeBulkStatus::SerializeFields | confirmed | Virtual slot 10. Calls base virtual vtable+8 on param_2, writes 'Starting LobbyCommand_ChangeBulkStatus' and the newline with 013c26e0, calls 013b2960, calls 0… |
| `014c71c0` | LobbyCommand_ChangeBulkStatus::DtorBody | confirmed | Destructor body. Stores LobbyCommand_ChangeBulkStatus::vftable at this, frees the DWORD buffer at this+0x1c when non-null and zeroes this+0x1c..+0x24, frees th… |
| `014c7270` | LobbyCommand_ChangeBulkStatus::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_ChangeBulkStatus' (6 bytes: mov eax, imm32; ret). |
| `014c7320` | LobbyCommand_ChangeBulkStatus::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls the dtor body 014c71c0, then frees this when bit 0 of the one stack argument is set (bytes 56 8bf1 e8.. f644240801 74 09 56 e8.. 83c404 8… |
| `014c7340` | LobbyCommand_ChangeBulkStatus::DumpHeader | confirmed | Virtual slot 19, stdcall with one stack argument (the sink, ret 4). Writes 'Change Bulk Status Lobby Command' plus newline (33 bytes) through 01241650 with the… |
| `014c7370` | LobbyCommand_ChangeBulkStatus::WriteCommandName | confirmed | Virtual slot 20. Loads its stack argument into ECX, pushes the 32-byte literal 'Change Bulk Status Lobby Command', calls 01241650 with that receiver, and retur… |
| `014c7900` | LobbyCommand_ChangeBulkStatus::Ctor | confirmed | Constructor. Calls 013b2760 (base constructor), stores LobbyCommand_ChangeBulkStatus::vftable at this, zeroes the DWORD triples at +0xc..+0x14 and +0x1c..+0x24… |
| `014c7970` | LobbyCommand_ChangeBulkStatus::Clone | confirmed | Virtual slot 17. Allocates 0x28 bytes (00d833a6); when non-null constructs them with 014c7900 and returns the object, else 0. Then calls this->vtable[0x48] (sl… |
| `014c79e0` | LobbyCommand_ChangeBulkStatus::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to LobbyCommand_ChangeBulkStatus) through 00d8d382; on failure asserts 'clone' with LobbyCommandChangeBu… |

**LobbyCommand_ChangeLobbyDisplay** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c5e60` | LobbyCommand_ChangeLobbyDisplay::Ctor | confirmed | Constructor. Calls 013b2760 (base constructor), stores LobbyCommand_ChangeLobbyDisplay::vftable at this, zeroes this+8, and returns this. |
| `014c5ee0` | LobbyCommand_ChangeLobbyDisplay::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_ChangeLobbyDisplay' (bytes b8 imm32 c3, mov eax, 0x01902934; ret). |
| `014c5f20` | LobbyCommand_ChangeLobbyDisplay::DtorBody | confirmed | Destructor body without a free. Stores LobbyCommand_ChangeLobbyDisplay::vftable at this and calls 013b2820 (base cleanup) with the EH frame, then returns. |
| `014c6040` | LobbyCommand_ChangeLobbyDisplay::Clone | confirmed | Virtual slot 17. Allocates 0xc bytes (00d833a6), constructs them with 014c5e60 when non-null, then calls this->vtable[0x48] (slot 18, CloneInto) with the resul… |
| `014c60b0` | LobbyCommand_ChangeLobbyDisplay::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to LobbyCommand_ChangeLobbyDisplay) through 00d8d382; on failure asserts 'clone' with LobbyCommandChange… |
| `014c6150` | LobbyCommand_ChangeLobbyDisplay::SerializeFields | confirmed | Virtual slot 10. Calls base virtual vtable+8 on param_2, writes 'Starting LobbyCommand_ChangeLobbyDisplay' with 013c26e0, calls 013b2960, writes 'GroupID ' and… |
| `014c6230` | LobbyCommand_ChangeLobbyDisplay::ScalarDeletingDtor | confirmed | Virtual slot 0, combined destructor: stores LobbyCommand_ChangeLobbyDisplay::vftable, calls 013b2820 (base cleanup), frees this when bit 0 of the flag argument… |
| `014c6290` | LobbyCommand_ChangeLobbyDisplay::DumpText | confirmed | Virtual slot 19. Writes 'Change Lobby display Lobby Command' plus newline (35 bytes) through 01241650, calls 013b2af0 with param_2, then sprintf-formats 'Group… |
| `014c6320` | LobbyCommand_ChangeLobbyDisplay::WriteCommandName | confirmed | Virtual slot 20. Loads its first stack argument into ECX, pushes the 34-byte literal 'Change Lobby Display Lobby Command', calls 01241650 with that receiver, a… |

**LobbyCommand_ConfigureGroup** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c5830` | LobbyCommand_ConfigureGroup::Ctor | confirmed | Constructor. Calls 013b2760, stores LobbyCommand_ConfigureGroup::vftable at this, allocates 0x10 bytes (00d833a6); when non-null it calls 01312930 and stores t… |
| `014c58e0` | LobbyCommand_ConfigureGroup::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_ConfigureGroup' (bytes b8 imm32 c3). |
| `014c5920` | LobbyCommand_ConfigureGroup::DtorBody | confirmed | Destructor body. Stores LobbyCommand_ConfigureGroup::vftable at this, calls slot 0 of the object at this+8 with flag 1 (scalar deleting destructor of the sub-o… |
| `014c5ac0` | LobbyCommand_ConfigureGroup::PropertySetForward_013111d0 | confirmed | Virtual slot 22. If this+8 (mPropertySet) is null it reports an assert with 'mPropertySet' and LobbyCommandConfigureGroup.cpp line 0x58; then calls 013111d0 wi… |
| `014c5b00` | LobbyCommand_ConfigureGroup::PropertySetForward_01312760 | confirmed | Virtual slot 23. If this+8 (mPropertySet) is null it reports an assert with 'mPropertySet' and LobbyCommandConfigureGroup.cpp line 0x5f; then calls 01312380(pa… |
| `014c5b40` | LobbyCommand_ConfigureGroup::PropertySetForward_01311c50 | confirmed | Virtual slot 25. Asserts 'mPropertySet' at LobbyCommandConfigureGroup.cpp line 0x66 when this+8 is null, then calls 01311c50 with ECX = this+8 and one stack ar… |
| `014c5b70` | LobbyCommand_ConfigureGroup::PropertySetForward_013110e0 | confirmed | Virtual slot 24. Asserts 'mPropertySet' at LobbyCommandConfigureGroup.cpp line 0x6c when this+8 is null, then calls 013110e0 with ECX = this+8 and one stack ar… |
| `014c5ba0` | LobbyCommand_ConfigureGroup::Clone | confirmed | Virtual slot 17. Allocates 0x14 bytes (00d833a6); if non-null it constructs them with 014c5830 and returns the object, else 0. Then calls this->vtable[0x48] (s… |
| `014c5c10` | LobbyCommand_ConfigureGroup::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to LobbyCommand_ConfigureGroup) through 00d8d382; on failure asserts 'clone' with LobbyCommandConfigureG… |
| `014c5d00` | LobbyCommand_ConfigureGroup::SerializeFields | confirmed | Virtual slot 10. Calls base virtual vtable+8 on param_2, writes 'Starting LobbyCommand_ConfigureGroup' with 013c26e0, calls 013b2960(param_2), calls mPropertyS… |
| `014c5df0` | LobbyCommand_ConfigureGroup::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls the dtor body 014c5920, then frees this when bit 0 of the flag argument is set, and returns this. |
| `014c5e10` | LobbyCommand_ConfigureGroup::DumpHeader | confirmed | Virtual slot 19. Writes 'Configure Group Lobby Command' plus newline (30 bytes) through 01241650, then calls 013b2af0 with param_1 (the sink). |
| `014c5e40` | LobbyCommand_ConfigureGroup::WriteCommandName | confirmed | Virtual slot 20. Loads its first stack argument into ECX, pushes the 29-byte literal 'Configure Group Lobby Command', calls 01241650 with that receiver, and re… |

**LobbyCommand_DisplayStats** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c4b10` | LobbyCommand_DisplayStats::WriteCommandName | confirmed | Virtual slot 20. Loads its first stack argument into ECX, pushes the 26-byte literal 'DisplayStats Lobby Command', calls 01241650 with that receiver, returns w… |
| `014c4b30` | LobbyCommand_DisplayStats::SerializeFields | confirmed | Virtual slot 10. Calls the base virtual at vtable+8 on param_2, writes 'Starting LobbyCommand_DisplayStats' and field labels with values (groupID this+0x24, To… |
| `014c5460` | LobbyCommand_DisplayStats::GetClassName | confirmed | Virtual slot 11. Returns the address of the literal 'LobbyCommand_DisplayStats' (bytes b8 imm32 c3, mov eax, 0x01902724; ret). |
| `014c55d0` | LobbyCommand_DisplayStats::Ctor | confirmed | Constructor. Calls 013b2760 with the EH frame, stores LobbyCommand_DisplayStats::vftable, zeroes the three DWORDs at +0x10..+0x18 and the string header at +0x3… |
| `014c5670` | LobbyCommand_DisplayStats::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls the dtor body 014c5390, then frees this when bit 0 of the flag argument is set, and returns this. |
| `014c5690` | LobbyCommand_DisplayStats::Clone | confirmed | Virtual slot 17. Allocates 0x54 bytes (00d833a6), constructs them with 014c55d0 on success, then calls this->vtable[0x48] (slot 18, CloneInto) with the new obj… |
| `014c5700` | LobbyCommand_DisplayStats::CloneInto | confirmed | Virtual slot 18. dynamic_casts param_2 (NetworkCommand to LobbyCommand_DisplayStats) through 00d8d382; if the cast fails it reports an assert with 'clone' and … |
| `014c49a0` | LobbyCommand_DisplayStats::DumpText | likely | Virtual slot 19. Writes the header 'Change Lobby display Lobby Command' and calls 013b2af0(param_2), then sprintf-formats the 'Group ID', 'Tournament ID', 'Rou… |

**LobbyCommand_GameServiceError** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c4540` | LobbyCommand_GameServiceError::WriteCommandName | confirmed | Virtual slot 20. Loads its first stack argument into ECX, pushes the 32-byte literal 'Game Service Error Lobby Command', calls 01241650 with that receiver, and… |

**LobbyErrorMessagePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a42a0` | LobbyErrorMessagePacket::LobbyErrorMessagePacket 🌐 | confirmed | Constructor for lobby sub-opcode 11 (0xb): calls the opcode-101 family constructor FUN_008a3ce0(0xb), installs LobbyErrorMessagePacket::vftable and stores the … |
| `008b7b80` | LobbyErrorMessagePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4310` | LobbyErrorMessagePacket::~LobbyErrorMessagePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the caller releases the storage. |

**RemoveLobbyGamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a41f0` | RemoveLobbyGamePacket::RemoveLobbyGamePacket 🌐 | confirmed | Constructor for lobby sub-opcode 8: calls the opcode-101 family constructor FUN_008a3ce0(8), installs RemoveLobbyGamePacket::vftable; body is wrapped in an MSV… |
| `008b7b20` | RemoveLobbyGamePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4250` | RemoveLobbyGamePacket::~RemoveLobbyGamePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**SendLeaveLobbyToClientPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4140` | SendLeaveLobbyToClientPacket::SendLeaveLobbyToClientPacket 🌐 | confirmed | Constructor for lobby sub-opcode 7: calls the opcode-101 family constructor FUN_008a3ce0(7), installs SendLeaveLobbyToClientPacket::vftable; body is wrapped in… |
| `008b7ac0` | SendLeaveLobbyToClientPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a41a0` | SendLeaveLobbyToClientPacket::~SendLeaveLobbyToClientPacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**SendLobbyToClientPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4090` | SendLobbyToClientPacket::SendLobbyToClientPacket 🌐 | confirmed | Constructor for lobby sub-opcode 6: calls the opcode-101 family constructor FUN_008a3ce0(6), installs SendLobbyToClientPacket::vftable; body is wrapped in an M… |
| `008b7a60` | SendLobbyToClientPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a40f0` | SendLobbyToClientPacket::~SendLobbyToClientPacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**ShowLobbyUiPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4360` | ShowLobbyUiPacket::ShowLobbyUiPacket 🌐 | confirmed | Constructor for lobby sub-opcode 12 (0xc): calls the opcode-101 family constructor FUN_008a3ce0(0xc), installs ShowLobbyUiPacket::vftable and stores the two pa… |
| `008b7be0` | ShowLobbyUiPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a43d0` | ShowLobbyUiPacket::~ShowLobbyUiPacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the caller releases the storage. |

**StartLobbyGamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3f20` | StartLobbyGamePacket::StartLobbyGamePacket 🌐 | confirmed | Constructor for lobby sub-opcode 3: calls the opcode-101 family constructor FUN_008a3ce0(3), installs StartLobbyGamePacket::vftable and stores the 2 payload wo… |
| `008b79a0` | StartLobbyGamePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a3f90` | StartLobbyGamePacket::~StartLobbyGamePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**UpdateLobbyGamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a3fe0` | UpdateLobbyGamePacket::UpdateLobbyGamePacket 🌐 | confirmed | Constructor for lobby sub-opcode 4: calls the opcode-101 family constructor FUN_008a3ce0(4), installs UpdateLobbyGamePacket::vftable; body is wrapped in an MSV… |
| `008b7a00` | UpdateLobbyGamePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4040` | UpdateLobbyGamePacket::~UpdateLobbyGamePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008f9650` | ReadHeaderBlobErrByte_008d90b0 | likely | Three args (start, length, destination) with the destination on the stack, plain ret: builds the bounded reader {start, length, cursor=start, end=start+length,… |
| `008f96b0` | ReadHeaderBlobErrByte_008d90b0 | likely | Three args (start, length, destination) with the destination on the stack, plain ret: builds the bounded reader {start, length, cursor=start, end=start+length,… |
| `008f9710` | ReadHeaderU32ToDest | likely | Three args (start, length, destination) with the destination on the stack, plain ret: builds the bounded reader, calls 008d90b0 with ECX=destination for the lo… |
| `008f98b0` | ReadHeaderU32ToDest | likely | Three args (start, length, destination) with the destination on the stack, plain ret: builds the bounded reader, calls 008d90b0 with ECX=destination for the lo… |

### `net.packet.lobby_game_def`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091e060` | BaseClient::HandleLobbyGameDefinitions 🌐 | confirmed | Handles the opcode 102 sub 2 lobby-definition packet: builds a reader over the supplied buffer, calls FUN_008d67a0 which reads two i16 header words and rejects… |

**BaseLobbyGameDefinitionPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4970` | BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacket 🌐 | confirmed | Lobby-game-definition family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 102 (0x66) at +4, installs BaseLobbyGameDefinition… |
| `008a49d0` | BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacke… 🌐 | confirmed | Lobby-game-definition family constructor taking the sub-opcode: installs BasePacket::vftable, stores opcode 102 (0x66) at +4, installs BaseLobbyGameDefinitionP… |
| `008a4a40` | BaseLobbyGameDefinitionPacket::~BaseLobbyGameDefinitionPack… | confirmed | Destructor body: writes BaseLobbyGameDefinitionPacket::vftable then BasePacket::vftable into the object vtable slot and returns; eight EH funclets (0157def0 an… |
| `008a4a90` | BaseLobbyGameDefinitionPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: in an SEH frame restores BaseLobbyGameDefinitionPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 i… |

**LobbyGameDefinitionPacketDefinitionsRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a221d0` | LobbyGameDefinitionPacketDefinitionsRequest::LobbyGameDefin… 🌐 | confirmed | Constructs the packet on the stack or heap object passed in ecx: calls the lobby-game-definition family constructor 008a49d0 with sub-opcode 1 (which stores op… |
| `00a22290` | LobbyGameDefinitionPacketDefinitionsRequest::ScalarDeleting… | confirmed | Scalar deleting destructor: restores the LobbyGameDefinitionPacketDefinitionsRequest, BaseLobbyGameDefinitionPacket and BasePacket vtables and then calls _free… |
| `00a22230` | LobbyGameDefinitionPacketDefinitionsRequest::Dtor | likely | Destructor body: rewrites the object's vtable pointer back through LobbyGameDefinitionPacketDefinitionsRequest::vftable, BaseLobbyGameDefinitionPacket::vftable… |

**LobbyGameDefinitionPacketDefinitionsResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4b00` | LobbyGameDefinitionPacketDefinitionsResponse::LobbyGameDefi… 🌐 | confirmed | Constructor taking two payload words: calls FUN_008a49d0(2) so the packet is opcode 102 with sub-opcode 2, installs LobbyGameDefinitionPacketDefinitionsRespons… |
| `008a4b70` | LobbyGameDefinitionPacketDefinitionsResponse::~LobbyGameDef… | confirmed | Destructor body: restores LobbyGameDefinitionPacketDefinitionsResponse::vftable, then BaseLobbyGameDefinitionPacket::vftable, then BasePacket::vftable; caller … |
| `008a4bd0` | LobbyGameDefinitionPacketDefinitionsResponse::ScalarDeletin… | confirmed | Scalar deleting destructor: in an SEH frame restores the response, BaseLobbyGameDefinitionPacket and BasePacket vtables, then calls _free(this) when param_2 bi… |

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008f74c0` | ReadOpcodeSubThenBlob 🌐 | confirmed | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when the destination or start is null; builds the bounded reader, calls 008d67a0 with EC… |
| `00919b60` | ParseKartLobbyDefinitionTables 🌐 | confirmed | Plain function (no this) with a reader (param_1) and the client sub-object at param_2 (+0x110 of the caller). Calls five table parsers in fixed order on fields… |
| `008e5450` | ReadLobbyGamesTable 🌐 | likely | cdecl-style, two stack arguments: reader (param_1) and table (param_2). Reads a u32 row count; per row reads a u32 key, allocates 0x20 bytes via 00d833a6 and, … |
| `008e5580` | ReadLobbyOptionsTable 🌐 | likely | cdecl-style, two stack arguments: reader (param_1) and table (param_2). Reads a u32 row count; per row reads a u32 key, allocates 0x1c bytes via 00d833a6, zero… |
| `008e56c0` | ReadLobbyValuesTable 🌐 | likely | cdecl-style, two stack arguments: reader (param_1) and table (param_2). Reads a u32 row count; per row reads a u32 key, allocates 0x1c bytes via 00d833a6, zero… |
| `008e5800` | ReadLobbySettingsTable 🌐 | likely | cdecl-style, two stack arguments: reader (param_1) and table (param_2). Reads a u32 row count; per row reads a u32 key, allocates 0x24 bytes via 00d833a6, zero… |
| `00a273f0` | SendLobbyGameDefinitionRequest 🌐 | likely | Writes two 16-bit values to a bounded stream, wraps the bytes in a tunneled BasePacket, and enters the packet-send path. |
| `00a27790` | SendLobbyGameDefinitionPacketDefinitionsRequest 🌐 | likely | Constructs a LobbyGameDefinitionPacketDefinitionsRequest temporary and, when owner field +0x280 is nonzero, passes it to 00a273f0 for tunneled sending. |

### `net.packet.loyalty`

**BaseLoyaltyRewardPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5190` | BaseLoyaltyRewardPacket::BaseLoyaltyRewardPacket 🌐 | confirmed | Loyalty-reward family constructor: installs BasePacket::vftable, stores opcode 173 (0xad) at +4, installs BaseLoyaltyRewardPacket::vftable and stores the sub-o… |
| `008a5200` | BaseLoyaltyRewardPacket::~BaseLoyaltyRewardPacket | confirmed | Destructor body: restores BaseLoyaltyRewardPacket::vftable then BasePacket::vftable; four EH funclets (0157e290 and neighbours) call it. |
| `008a5250` | BaseLoyaltyRewardPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: in an SEH frame restores BaseLoyaltyRewardPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |

**SetLoyaltyRewardLogoutInformation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a52c0` | SetLoyaltyRewardLogoutInformation::SetLoyaltyRewardLogoutIn… 🌐 | confirmed | Constructor: calls FUN_008a5190(0) so the packet is opcode 173 with sub-opcode 0, installs SetLoyaltyRewardLogoutInformation::vftable and zeroes +0xc and +0x10… |
| `008a5330` | SetLoyaltyRewardLogoutInformation::~SetLoyaltyRewardLogoutI… | confirmed | Destructor body: restores SetLoyaltyRewardLogoutInformation::vftable, then BaseLoyaltyRewardPacket::vftable, then BasePacket::vftable; its only caller is EH fu… |
| `008a5390` | SetLoyaltyRewardLogoutInformation::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: in an SEH frame restores the logout-report, loyalty-reward and BasePacket vtables, then calls _free(this) when param_2 bit 0 is set. |

### `net.packet.matchmaking`

**EncounterMatchmaking::BaseMatchmakingPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a862f0` | EncounterMatchmakingBaseMatchmakingPacketDtorBody | likely | Destructor body of EncounterMatchmaking::BaseMatchmakingPacket: stores its own vtable and then the BasePacket vtable and returns; reached from an MSVC unwind f… |

**EncounterMatchmaking::MatchInvitationResponsePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87ad0` | EncounterMatchmaking::MatchInvitationResponsePacketCtor 🌐 | confirmed | Constructor of the matchmaking invitation response packet: runs the BaseMatchmakingPacket family constructor 00a7f0d0(10) (which stores class id 0x8d/141 and t… |

### `net.packet.minigame`

**BaseBattleMagesPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd4d00` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Stores BaseBattleMagesPacket::vftable at [ECX] and then BasePacket::vftable at [ECX]; returns void. |

**BaseMiniGameBossPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c47e20` | BaseMiniGameBossPacket_Destruct | likely | Destructor body of the boss packet base temporary: the only operation is writing BasePacket::vftable into the object at +0 (base subobject teardown); it is ref… |
| `00c480d0` | BaseMiniGameBossPacket_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: writes BasePacket::vftable into the object and frees it with _free when the low bit of the flag argument is set. |
| `00c48620` | BaseMiniGameBossPacket_ReadPayload | likely | Reads a mini-game message header with the shared header reader 009b2340 and then three more 32-bit payload fields into this+0x18, +0x1c and +0x20. Each read is… |

**BaseMiniGamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4420` | BaseMiniGamePacket::BaseMiniGamePacket 🌐 | confirmed | Minigame packet-family constructor: installs BasePacket::vftable, stores the family opcode 39 (0x27) at +4, stores the sub-opcode argument at +8 and the next a… |
| `008b7c40` | BaseMiniGamePacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a44a0` | BaseMiniGamePacket::~BaseMiniGamePacket | likely | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; 73 EH unwind funclets call it, and the 009471a0 router also calls it right… |

**BasePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b6140` | MiniGameGroupInfoPacketDtor | likely | Calls FUN_009b6070 with ECX=this+0x18 to destroy the embedded group-info record, then restores BasePacket::vftable. |
| `009b6220` | MiniGameGameEndScorePacketDtor | likely | Restores the score-array vtable at this+0x18, calls FUN_009b1660 with ECX=this+0x18 and count from this+0x20, frees the backing pointer at this+0x1c with align… |

**BaseShipCombatPacket** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca2c80` | BaseShipCombatPacket::Dtor | confirmed | Replaces the object's vtable first with BaseShipCombatPacket and then with BasePacket during base destruction. |
| `00ca2cd0` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2d20` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2d70` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2e30` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2ee0` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2f30` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2f90` | BaseShipCombatPacket::Dtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |

**BattleMagesPacketCommand** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd5d40` | BattleMagesPacketCommand::~BattleMagesPacketCommand | confirmed | Stores SoeUtil::IString<char>::vftable at +0x1c. When +0x28 is positive it atomically decrements the dword at (+0x20)-4; when the previous count was 1 or not p… |
| `00cd5dd0` | BattleMagesPacketCommand::ScalarDeletingDtor | confirmed | Virtual slot 0: calls destructor body 00cd5d40 on this, then frees this when bit 0 of the flag argument is set; returns this. |

**BattleMagesPacketUpdatePlayerState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd4d90` | BattleMagesPacketUpdatePlayerState::BattleMagesPacketUpdate… | confirmed | Calls the BaseBattleMagesPacket constructor 00b6f2b0 with sub-opcode 7; stores BattleMagesPacketUpdatePlayerState::vftable at this; stores DAT_01bf0a38 at +0x1… |
| `00cd4e00` | BattleMagesPacketUpdatePlayerState::RestoreBaseVtablesThenF… | confirmed | Virtual slot 0: stores BaseBattleMagesPacket::vftable and then BasePacket::vftable at this, frees this when bit 0 of the flag argument is set, and returns this… |

**MiniGameBossDeletePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c47f70` | MiniGameBossDeletePacket_Construct | confirmed | Constructs the boss-delete packet: runs the boss base constructor with discriminator 2 (stored at +0x14), installs MiniGameBossDeletePacket::vftable and stores… |
| `00c47fe0` | MiniGameBossDeletePacket_Destruct | likely | Destructor body of the boss-delete packet temporary: writes BasePacket::vftable into the object at +0 and returns; called from the delete branch of the handler… |
| `00c48190` | MiniGameBossDeletePacket_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: writes BasePacket::vftable into the object and frees it with _free when the low bit of the flag argument is set. |

**MiniGameBossUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c47e80` | MiniGameBossUpdatePacket_Construct | confirmed | Constructs the boss-update packet: runs the boss base constructor with discriminator 1 (stored at +0x14), installs MiniGameBossUpdatePacket::vftable and zeroes… |
| `00c47ef0` | MiniGameBossUpdatePacket_Destruct | likely | Destructor body of the boss-update packet temporary: writes BasePacket::vftable into the object at +0 and returns; the handler 00c48c90 calls it at the end of … |
| `00c48130` | MiniGameBossUpdatePacket_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: writes BasePacket::vftable into the object and frees it with _free when the low bit of the flag argument is set. |

**MiniGameDataLockedPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009afa50` | MiniGameDataLockedPacket::Ctor | confirmed | Calls minigame base constructor with values 1 and -1 sentinels, installs packet vtable, and initializes three integer arrays. |
| `009b0800` | MiniGameDataLockedPacket_ScalarDeletingDtor | confirmed | Calls packet destructor and conditionally frees the packet object when deletion flag bit zero is set. |
| `009afae0` | MiniGameDataLockedPacket::Dtor | likely | Restores array vtables, frees three integer-array buffers, clears their fields, and restores BasePacket vtable. |

**MiniGameFlashCommandAdminPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b3110` | MiniGameFlashCommandAdminPacketCtor 🌐 | confirmed | Calls the base admin packet constructor with sub-opcode 0x3f, installs MiniGameFlashCommandAdminPacket vtable, initializes an IString member and copies the pas… |
| `009b31a0` | MiniGameFlashCommandAdminPacket::ScalarDeletingDtor | confirmed | Calls FUN_009af9d0 to destroy members; when deletion flag bit 0 is set, calls free(param_1), then returns the object pointer otherwise. |
| `009af9d0` | MiniGameFlashCommandAdminPacket_Dtor | likely | Restores BasePacket vtable and releases its IString<char> member buffer. |

**MiniGameGameEndScorePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b61a0` | MiniGameGameEndScorePacketCtor 🌐 | confirmed | Calls base minigame packet constructor with sub-opcode 0x2f, installs concrete vtable, and initializes an empty MiniGameScoreInfo array. |
| `009b8610` | MiniGameGameEndScorePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor for the 39/47 end-score packet (MiniGameGameEndScorePacket): calls the class destructor body 009b6220, which destroys the SoeUtil::A… |

**MiniGameGroupInfoPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b8580` | MiniGameGroupInfoPacket::MiniGameGroupInfoPacket 🌐 | confirmed | Mini-game packet constructor: calls FUN_008a4420(0x32,0,-1,-1) so the packet is family 39 with sub-opcode 50 (0x32) and the three id words 0/-1/-1, installs Mi… |
| `009b85f0` | MiniGameGroupInfoPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor for the family-39 group-info packet: calls the class destructor body 009b6140, which resets BasePacket::vftable and, through 009b607… |

**MiniGameInfoPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9c70` | MiniGameInfoPacket::MiniGameInfoPacket 🌐 | confirmed | Constructs the family-39 sub-opcode 16 (0x10) info packet: calls FUN_008a4420(0x10,0,-1,-1), installs MiniGameInfoPacket::vftable, initialises the embedded sta… |
| `009b9cf0` | MiniGameInfoPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor for the 39/16 info packet: calls the packet destructor body 009b0900. Then frees the object through _free when the low bit of param_… |

**MiniGameLootWheelSetItemToLandOnPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9e20` | MiniGameLootWheelSetItemToLandOnPacket::MiniGameLootWheelSe… 🌐 | confirmed | Constructs the family-39 sub-opcode 45 (0x2d) loot-wheel packet: calls FUN_008a4420(0x2d,-1,-1,-1), installs MiniGameLootWheelSetItemToLandOnPacket::vftable an… |
| `009b9e90` | MiniGameLootWheelSetItemToLandOnPacket::ScalarDeletingDestr… | likely | Scalar deleting destructor for the 39/45 loot-wheel packet: calls the packet destructor body 009aa050, the same destructor the dispatcher arm for case 0x2d use… |

**MiniGamePayloadPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b0960` | MiniGamePayloadPacket::Ctor | confirmed | Calls minigame base constructor with subtype 0xe and sentinels, installs payload packet vtable, and initializes byte array. |
| `009b09d0` | MiniGamePayloadPacket::CtorWithPayload | confirmed | Calls minigame base constructor with subtype 0xe and supplied param_2, installs packet vtable, initializes byte array. |
| `009b0a40` | MiniGamePayloadPacket_ScalarDeletingDtor | confirmed | Calls payload packet destructor and conditionally frees the object when deletion flag bit zero is set. |

**MiniGameRewardStatusPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9d10` | MiniGameRewardStatusPacket::MiniGameRewardStatusPacket 🌐 | confirmed | Constructs the family-39 sub-opcode 22 (0x16) reward-status packet: calls FUN_008a4420(0x16,param_2,-1,-1), installs MiniGameRewardStatusPacket::vftable, initi… |
| `009b9e00` | MiniGameRewardStatusPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor for the 39/22 reward-status packet: calls the packet destructor body 009b31c0, the same destructor the handler 009ba720 uses to clos… |

**MiniGameUpdateRewardPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9eb0` | MiniGameUpdateRewardPacket::MiniGameUpdateRewardPacket 🌐 | confirmed | Constructs the family-39 sub-opcode 68 (0x44) update-reward packet: calls FUN_008a4420(0x44,-1,-1,-1), installs MiniGameUpdateRewardPacket::vftable and initial… |
| `009b9f20` | MiniGameUpdateRewardPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor for the 39/68 update-reward packet: calls the packet destructor body 009b0a60. Then frees the object through _free when the low bit … |

**ObjectiveUnhidePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9f40` | ObjectiveUnhidePacket::ObjectiveUnhidePacket 🌐 | confirmed | Constructs the objective-family packet: FUN_009aa4c0(5) stores the family opcode 0x2d (45) with sub-opcode 5 and installs BaseObjectivePacket::vftable; this co… |
| `009b9fa0` | ObjectiveUnhidePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor for the objective-family 45/5 unhide packet: calls the packet destructor body 009aa910 (destructor of a member owning RewardBundleBa… |

**PacketInviteAndStartMiniGameInvitePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87920` | PacketInviteAndStartMiniGameInvitePacketScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of PacketInviteAndStartMiniGameInvitePacket: rewrites the object's vtable to ClientGateway::BasePacket and frees i… |
| `00a97cc0` | PacketInviteAndStartMiniGameInvitePacket::SendFields | likely | Thiscall with five dword arguments and a byte flag. After callee 00a878c0 has constructed the packet at buffer+0, fills buffer+0x10 through buffer+0x20 with th… |

**RewardBundleBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9a70` | RewardBundleBase::RewardBundleBase | confirmed | Reward-bundle base constructor: installs RewardBundleBase::vftable, zeroes slots 0xe/0xf/0x10 and 0x24, installs the SoeUtil::List vftables for StringFixed<256… |

**ShipCombatPacketFireAtLocation** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca2dc0` | ShipCombatPacketFireAtLocation::Ctor | confirmed | Calls the common ship-combat packet base constructor with sub-opcode 0x14, installs the FireAtLocation vtable, initializes four default words, and clears one f… |
| `00ca3d10` | ShipCombatPacketFireAtLocation::ScalarDeletingDtor | confirmed | Restores the base packet vtables, frees the allocation if the low deleting flag is set, and returns the pointer otherwise. |

**ShipCombatPacketPlaySound** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca3f10` | ShipCombatPacketPlaySound::Ctor | confirmed | Calls the base packet constructor with sub-opcode 0x19, installs the PlaySound vtable, clears two fields, and initializes two sound words. |
| `00ca3f90` | ShipCombatPacketPlaySound::ScalarDeletingDtor | confirmed | Restores the packet base vtables, frees the packet allocation when the low deleting flag is set, and otherwise returns it. |

**ShipCombatPacketResetProxiedShip** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca3870` | ShipCombatPacketResetProxiedShip::Ctor | confirmed | Calls the shared packet base constructor with sub-opcode 0xb, installs the ResetProxiedShip vtable, initializes two identifier words, and clears the final fiel… |
| `00ca3910` | ShipCombatPacketResetProxiedShip::ScalarDeletingDtor | confirmed | Restores the BaseShipCombatPacket and BasePacket vtables, frees the packet allocation when the low deleting flag is set, and returns the object otherwise. |

**ShipCombatPacketShipHit** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca3df0` | ShipCombatPacketShipHit::Ctor | confirmed | Calls the shared base constructor with sub-opcode 0x17, installs the ShipHit vtable, and clears three payload fields. |
| `00ca3ea0` | ShipCombatPacketShipHit::ScalarDeletingDtor | confirmed | Restores the base packet vtables, frees the packet allocation when the low deleting flag is set, and returns it otherwise. |

**ShipCombatPacketSpecialMove** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca2e80` | ShipCombatPacketSpecialMove::Ctor | confirmed | Calls the shared packet base constructor with sub-opcode 0x16, installs the SpecialMove vtable, and clears its payload field. |
| `00ca3d80` | ShipCombatPacketSpecialMove::ScalarDeletingDtor | confirmed | Restores the base packet vtables, frees the allocation when the low deleting flag is set, and otherwise returns the packet pointer. |

**ShipCombatPacketUpdatePlayerState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca3ab0` | ShipCombatPacketUpdatePlayerState::Ctor | confirmed | Calls the base packet constructor with sub-opcode 0x11, installs the UpdatePlayerState vtable, and clears its payload words and flag. |
| `00ca3b50` | ShipCombatPacketUpdatePlayerState::ScalarDeletingDtor | confirmed | Restores the packet base vtables, frees the allocation when the low deleting flag is set, and otherwise returns the packet pointer. |

**ShipCombatPacketUpdateShipPhysics** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca3980` | ShipCombatPacketUpdateShipPhysics::Ctor | confirmed | Calls the base packet constructor with sub-opcode 0x10, installs the UpdateShipPhysics vtable, clears one byte, initializes a nested object and copies sixteen … |
| `00ca3a40` | ShipCombatPacketUpdateShipPhysics::ScalarDeletingDtor | confirmed | Restores the base packet vtables, frees the allocation when the low deleting flag is set, and otherwise returns the packet pointer. |

**ShipCombatPacketUpdateWorldForces** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca3bc0` | ShipCombatPacketUpdateWorldForces::Ctor | confirmed | Calls the shared base packet constructor with sub-opcode 0x12, installs the UpdateWorldForces vtable, initializes default vector words and clears trailing fiel… |
| `00ca3ca0` | ShipCombatPacketUpdateWorldForces::ScalarDeletingDtor | confirmed | Restores the base packet vtables, conditionally frees the packet when the low deleting flag is set, and returns it otherwise. |

**(no class)** (80)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b82d0` | ReadScoreRowArray | confirmed | Reads an i32 row count, resizes the SoeUtil array object passed as param_2 (grow through FUN_009b4ad0 when the requested size is larger, destroy-trailing throu… |
| `009bd450` | MiniGame_RequestCreateMiniGameGroup 🌐 | confirmed | Request builder exported to Lua as MiniGame:RequestCreateMiniGameGroup: logs the ids, constructs the family-39 sub-opcode 48 (0x30) packet through FUN_008a4420… |
| `009bd4f0` | MiniGame_RequestMiniGameGroup 🌐 | confirmed | Request builder exported to Lua as MiniGame:RequestMiniGameGroup: logs the ids, constructs the family-39 sub-opcode 49 (0x31) packet through FUN_008a4420(0x31,… |
| `009bf890` | DispatchMiniGamePacket | confirmed | Switches on the parsed mini-game sub-opcode and routes cases 0x0e, 0x11-0x13, 0x16, 0x28-0x29, 0x2f, 0x36, 0x41, and 0x42 to their corresponding message, state… |
| `009bfdb0` | HandleMiniGameJoinGame | confirmed | Constructs a temporary MiniGameInfoPacket, parses the join body, and after requiring the reader error flag clear proceeds when signed (end pointer - current po… |
| `009bfea0` | DispatchMiniGameMessage | confirmed | Constructs a temporary family-39 packet record, parses the shared header, handles explicit sub-opcodes 1, 2, 0x0f, 0x10, 0x12-0x15, 0x17, 0x2c-0x2d, 0x2f, 0x32… |
| `009b0820` | MiniGamePacketSharedMemberCleanup | likely | Releases the IString member at +0x270; cleans the ObjectiveData array object at +0x240 and frees its buffer at +0x244; calls 0103d010 with receivers at +0x188,… |
| `009b0900` | MiniGameInfoPacket_DtorWrapper | likely | Calls 009b0820 with this+0x18 in ECX, then restores BasePacket vtable at this. |
| `009b0a60` | MiniGameUpdateRewardPacket_DestructorBody | likely | Calls 009b0820 with this+0x18 in ECX, then restores BasePacket vtable at this. |
| `009b2340` | ReadMiniGamePacketHeader | likely | Reads a five-field header: sign-extends the 16-bit value to +4 and the 8-bit value to +8, then reads three i32 values into +0xc, +0x14 and +0x10. Each short re… |
| `009b24a0` | ReadMiniGameAdminHeader | likely | Sign-extends a 16-bit value into +4 and an 8-bit value into +8, then reads two i32 values into +0xc and +0x10. Each underflow zeroes that field, sets stream er… |
| `009b2550` | ReadHeaderI16I8 | likely | Reads the two-field prefix, sign-extending the 16-bit value into output +4 and the 8-bit value into +8. On underflow it zeroes that destination, sets stream er… |
| `009b25b0` | ReadI16I8TwoInts | likely | Calls the signed i16/i8 prefix reader, then reads two i32 values into output +0xc and +0x10; each underflow zeroes the destination, sets stream error and moves… |
| `009b2b30` | ReadScoreRow | likely | Reads a signed length and attempts to consume that many bytes. A negative or truncated length sets error and moves cursor to end, but the parser then still att… |
| `009b2c40` | ReadGroupElementFields | likely | Reads two i32 values, skips two signed length-prefixed byte spans, then reads the record's byte flags and remaining i32 fields with bounds checks that set stre… |
| `009b4560` | ReadMiniGameHeaderAndFloat | likely | Reads the shared five-field minigame header, reads a float at output+0x18, and marks stream flag +0x11 if that float is NaN. |
| `009b4600` | ReadHeaderAndI32 | likely | Reads the signed i16/i8 prefix through 009b2550, then one i32 into output +0xc; an underflow zeroes the field, sets stream error and moves cursor to end. |
| `009b4680` | ReadHeaderAndI32 | likely | Reads the signed i16/i8 prefix through 009b2550, then one i32 into output +0xc; an underflow zeroes the field, sets stream error and moves cursor to end. |
| `009b46c0` | ReadHeaderAndI32 | likely | Reads the signed i16/i8 prefix through 009b2550, then one i32 into output +0xc; an underflow zeroes the field, sets stream error and moves cursor to end. |
| `009b4700` | ReadMiniGameMessageBody | likely | Parses the common minigame header, an i32 at output+0x18 and a byte flag at +0x1c, checking bounds before each read. |
| `009b4770` | ReadMiniGameKnockOutBody | likely | Parses shared minigame header, then reads two i32 values into output+0x18 and +0x1c with truncation checks. |
| `009b4a10` | ReadRewardStatusBody | likely | Destroys old entries, reads a row count, then repeatedly reads each row's count/value and allocates/inserts linked nodes. |
| `009b5580` | TryReadMiniGameHeaderFloat | likely | Validates input pointers, wraps supplied bytes in a bounded reader, parses the common header and float body, then returns success only with no reader error and… |
| `009b56d0` | TryReadI16I8TwoI32 | likely | Validates inputs, builds a bounded reader, parses the signed i16/i8 prefix and two i32 fields through 009b25b0, then succeeds only when no error occurred and p… |
| `009b5740` | TryReadI16I8TwoI32 | likely | Validates inputs, parses the signed i16/i8 prefix and two i32 fields through 009b2620, then succeeds only when no error occurred and param_4 is nonzero or curs… |
| `009b57b0` | TryReadI16I8ThreeI32 | likely | Builds a bounded reader, parses the signed i16/i8 prefix and three i32 fields through 009b2690, then succeeds on no error when param_4 permits trailing bytes o… |
| `009b5820` | TryReadI16I8I32 | likely | Builds a bounded reader, parses the signed i16/i8 prefix and one i32 through 009b4600, then checks reader error and the optional full-consumption condition. |
| `009b5890` | TryReadI16I8ExtendedRecord | likely | Builds a bounded reader, parses the signed i16/i8 prefix and conditional-payload body through 009b2760, then checks reader error and the optional full-consumpt… |
| `009b5900` | TryReadI16I8LengthPrefixedBytes | likely | Builds a bounded reader, parses the signed i16/i8 prefix and validated length-prefixed byte span through 009b2860, then checks reader error and optional full c… |
| `009b5970` | TryReadI16I8I32Bool | likely | Builds a bounded reader, parses the signed i16/i8 prefix, one i32 at +0xc and one byte normalized to bool at +0x10 through 009b28f0, then checks error and opti… |
| `009b59e0` | TryReadI16I8ThreeI32 | likely | Builds a bounded reader, parses the signed i16/i8 prefix and three i32 fields through 009b2960, then succeeds on no error when param_4 permits trailing bytes o… |
| `009b5a50` | TryReadI16I8I32 | likely | Builds a bounded reader, parses the signed i16/i8 prefix and one i32 through 009b4680, then checks reader error and the optional full-consumption condition. |
| `009b5ac0` | TryReadI16I8I32 | likely | Builds a bounded reader, parses the signed i16/i8 prefix and one i32 through 009b46c0, then checks reader error and the optional full-consumption condition. |
| `009b5dd0` | ReadMiniGameHeaderBool | likely | Reads the common five-field minigame header then one byte converted to bool, setting reader error and advancing cursor to end when truncated. |
| `009b5ec0` | ReadMiniGameHeaderAndTwoI32 | likely | Parses the shared five-field header, then reads two consecutive i32 fields into output offsets +0x18 and +0x1c. |
| `009b5f10` | ReadHeaderI32OptionalTail | likely | Parses the shared five-field header, reads an i32 at output+0x18, then when output+0x1c is nonzero passes that value to FUN_009b0700 for additional bounded con… |
| `009b63f0` | ReadMiniGameAdminPacketHeader | likely | Initializes the BaseMiniGameAdminPacket base; for non-null input it wraps the supplied byte span in a bounded reader and calls 009b24a0 to parse a sign-extende… |
| `009b7d20` | ReadHeaderAndByteArrayRecord | likely | Body reader: parses the shared 5-field record (u16 opcode, u8 sub-opcode, three i32 ids) with FUN_009b2340 into the caller's object, then reads a count-prefixe… |
| `009b7da0` | ReadHeaderAndThreeIntArrays | likely | Body reader: parses the 5-field record with FUN_009b2340, then reads three count-prefixed i32 arrays into out+0x18, out+0x28 and out+0x38 through FUN_009b0660;… |
| `009b7e40` | ReadHeaderAndLengthBlob | likely | Body reader: parses the 5-field record with FUN_009b2340 and then a 32-bit length plus that many bytes through FUN_008bfc00, which stores (pointer,length) into… |
| `009b7ec0` | ReadNineFieldRecord | likely | Body reader: parses a 9-field record through FUN_009b48b0 - the shared 5-field record followed by a byte flag at out+0x18 and three i32 values at out+0x1c, out… |
| `009b7f30` | ReadHeaderAndTwoInts | likely | Body reader: parses the 5-field record into the caller's object and then two i32 words at out+0x18 and out+0x1c through FUN_009b5ec0; returns 1 only when the r… |
| `009b7fa0` | ReadHeaderIntAndString | likely | Body reader: parses the 5-field record, an i32 at out+0x18 and then, only when out+0x1c is non-zero, a further value at out+0x1c through the member reader FUN_… |
| `009b8230` | ReadRewardStatusBody | likely | Body reader for the reward-status message: 5-field header (FUN_009b2340); one reward-bundle entry through FUN_008e7930 into packet+0x18; two i32 words into +0x… |
| `009b8390` | ReadGroupElementArray | likely | Reads an i32 count, resizes the array object passed as param_2 (grow with FUN_009b4bb0, destroy-trailing with FUN_009b2100) and parses exactly count elements t… |
| `009b8420` | WriteFourFieldRecord | likely | Serialises a four-field record into the caller's writer object: the low i16 of record+4 (2 bytes), the byte at record+8, the i32 at record+0xc and the i32 at r… |
| `009b9700` | ReadHeaderAndRewardBundleEntry | likely | Body reader: parses the shared 5-field record (FUN_009b2340) and then one reward-bundle entry through FUN_008e7930 (whose own error string is 'Failed to Create… |
| `009b9860` | MiniGameGroupRecord_Read | likely | Reads the group record into the caller's 76-byte output structure: four i32 words, a length-prefixed blob through FUN_00894b10 into out+0x14, one i32 at out+0x… |
| `009ba240` | ReadObjectiveUnhideBody | likely | Reads the objective-unhide body into the caller's object: FUN_009b2550 (shared objective header/prefix) followed by FUN_008fd770 (a reward-bundle-style entry);… |
| `009ba6c0` | WriteHeaderAndLengthPrefixedString | likely | Serializes the shared record header from this+4 (u16), this+8 (u8), and i32 fields at +0x0c, +0x14 and +0x10 through FUN_009138e0; then writes the i32 length a… |
| `009bac60` | ReadHeaderAndScoreRows | likely | Body reader: parses the shared 5-field record (FUN_009b2340) and then a count-prefixed score-row array into the object at out+0x18 through FUN_009b82d0; return… |
| `009bae20` | SendAdminRecordPacket 🌐 | likely | Builds and sends a small record packet: creates a local writer (FUN_008d4a40 with a 0x10000000 cap), serialises the four-field record through FUN_009b8420, mar… |
| `009bb570` | SendPayloadVia0090b3d0 🌐 | likely | Send wrapper: creates a local writer (FUN_008d4a40 with a 0x10000000 cap), serialises the payload through FUN_0090b3d0, marks the packet object as a tunnelled … |
| `009bb680` | SendPayloadVia009138e0A 🌐 | likely | Send wrapper identical in shape to 009bb570 but with the payload builder FUN_009138e0: writer creation (FUN_008d4a40), payload serialisation, tunnelled-packet … |
| `009bb790` | SendPayloadVia009138e0B 🌐 | likely | Send wrapper identical in shape to 009bb570 but with the payload builder FUN_009138e0: writer creation (FUN_008d4a40), payload serialisation, tunnelled-packet … |
| `009bb8a0` | SendPayloadVia0090ac70 🌐 | likely | Send wrapper identical in shape to 009bb570 but with the payload builder FUN_0090ac70: writer creation (FUN_008d4a40), payload serialisation, tunnelled-packet … |
| `009bb9f0` | SendPayloadVia009b8420AndString 🌐 | likely | Send wrapper: creates a local writer (FUN_008d4a40), serialises the four-field admin record through FUN_009b8420, appends a string member from the caller objec… |
| `009bc060` | ReadHeaderAndGroupRecord | likely | Body reader: parses the shared 5-field record (FUN_009b2340) and then the 76-byte group record (FUN_009b9860) into the caller's object; returns 1 only when the… |
| `009bc230` | SendStringBodyPacket 🌐 | likely | Send wrapper for a string-bearing packet: creates a local writer through FUN_008d4a40, writes the shared record header from this+4 (u16), this+8 (u8), and i32 … |
| `009bcc00` | SendMiniGameAdmin28 🌐 | likely | Builds and sends the family-95 BaseMiniGameAdminPacket request with sub-opcode 0x1c (28): takes the current active state id (*(this+0x18)+0xc) or 0, constructs… |
| `009bcd40` | SendMiniGameAdmin34B 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x22 (34): current state id (or 0) and param_2 go to FUN_009a97c0(0x22,id,param_2) and the packet … |
| `009bcde0` | SendMiniGameAdmin30 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x1e (30): current state id (or 0) and param_2 go to FUN_009a97c0(0x1e,id,param_2) and the packet … |
| `009bce80` | SendMiniGameAdmin36 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x24 (36): current state id (or 0) and param_2 go to FUN_009a97c0(0x24,id,param_2) and the packet … |
| `009bcf20` | SendMiniGameAdmin31 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x1f (31): current state id (or 0) and the constant -1 go to FUN_009a97c0(0x1f,id,-1) and the pack… |
| `009bcfc0` | SendMiniGameAdmin32 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x20 (32): current state id (or 0) and the constant -1 go to FUN_009a97c0(0x20,id,-1) and the pack… |
| `009bd060` | SendMiniGameAdmin58 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x3a (58): current state id (or 0) and the constant -1 go to FUN_009a97c0(0x3a,id,-1) and the pack… |
| `009bd100` | SendMiniGameAdmin33 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x21 (33): current state id (or 0) and the constant -1 go to FUN_009a97c0(0x21,id,-1) and the pack… |
| `009bd1a0` | SendMiniGameAdmin64 🌐 | likely | Builds and sends the family-95 admin request with sub-opcode 0x40 (64): current state id (or 0) and param_2 go to FUN_009a97c0(0x40,id,param_2) and the packet … |
| `009bd590` | MiniGame_RequestCreateMiniGame 🌐 | likely | Request builder exported to Lua as MiniGame:RequestCreateMiniGame: logs the id and, depending on param_3, constructs the family-39 sub-opcode 3 (0x3) packet th… |
| `009bdd70` | ReadMiniGameInfoRecord | likely | Reads scalar and boolean fields into the composite output record, reads three string fields and the embedded count-prefixed reward-entry array through 009bc380… |
| `009bfd30` | ReadMiniGameRewardUpdate | likely | Reads the shared mini-game header and reward entry. Returns 1 only when the reader error byte is clear and either param_4 is nonzero or the signed difference (… |
| `00c48890` | StreamReadDwordTo18 | likely | Reads one 32-bit value out of the message stream param_1 (cursor at +8, end at +0xc) into param_2+0x18. When the cursor plus 4 would pass the end it zeroes the… |
| `00c48b60` | ReadKeyFromExactBuffer | likely | Unpacks a single 32-bit key field with 00c48890 out of the buffer param_2 (length param_3) into param_1+0x18 and reports success: it returns 1 when the read ra… |
| `00ca5620` | ReadPlayerStatePacketFields | likely | Reads a bounded input cursor into six consecutive payload fields, advancing by eight bytes for the first pair, then four bytes for each remaining dword or byte. |
| `00ca5720` | ReadWorldForcesPacketFields | likely | Reads a bounded cursor into a vector pair, multiple scalar fields, a nested value, boolean, and trailing scalar while marking malformed or non-finite values. |
| `00ca5a60` | ReadPlaySoundPacketFields | likely | Reads a scalar, boolean, nested value, and two-word field pair from a bounded input cursor, setting the cursor error flag on truncation. |
| `00ca5cb0` | ReadResetProxiedShipPacketFields | likely | Reads a bounded input cursor into an identifier pair, four nested 16-byte values, and a final scalar, recording truncation or non-finite input. |
| `00ca6040` | ApplyUpdatePlayerStatePacket | likely | Parses an UpdatePlayerState payload, finds a linked entity by its two identifiers, updates its state flags and counters, notifies related objects, and copies f… |
| `00ca6310` | ApplyUpdateWorldForcesPacket | likely | Parses an UpdateWorldForces payload, finds the target linked entity, copies decoded fields into its nested state, updates three world components, and destroys … |
| `00cd51b0` | BuildPlayerStateAndSend | likely | Fastcall (ECX = this). Constructs a BattleMagesPacketUpdatePlayerState (00cd4d90) in a stack buffer, copies the dwords at this+0xc, +0x8 and +0x10 into three l… |

### `net.packet.mount`

**PacketMountRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3730` | PacketMountRequest::PacketMountRequest 🌐 | confirmed | Calls the mount family constructor 0089f9c0(1) (which stores BasePacket id 0xa8, 168, and writes sub-opcode 1), installs PacketMountRequest::vftable, stores pa… |

**PacketMountUpgradeRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bff390` | PacketMountUpgradeRequest::Ctor 🌐 | confirmed | Calls the mount-family packet base constructor with sub-opcode 14, installs PacketMountUpgradeRequest::vftable, and initializes two fields to -1. |
| `00bff4a0` | PacketMountUpgradeRequest::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and calls _free(this) only when flag bit 0 is set; otherwise returns this. |

### `net.packet.namechange`

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a21c00` | StartTwoPartNameCheck 🌐 | confirmed | Validates a proposed first/last name pair by copying both strings into temporaries and converting them to wide strings. It enforces length 3..14 only on the fi… |
| `00a1e6f0` | SendNameChangeStatusPacket 🌐 | likely | Builds NameChange::ChangeNameRequestStatusPacket from three dword arguments and, when the gateway pointer at *(this+0x3bc)+0x280 is non-null, sends the stack p… |
| `00a1eaf0` | SendChangeNameRequestShapedPacket 🌐 | likely | Serialises the ChangeNameRequest-shaped packet passed in the argument (header via 00a1ae40, type dword at +0xc, 8-byte handle at +0x10, NameData at +0x18, no t… |
| `00a1ff40` | StartPetNameCheck 🌐 | likely | Same client-side pre-validation as the house check but for pet names: converts the proposed name to a wide string with 00701290, enforces length and character … |
| `00a21a50` | HandleGuildNameCheckResultKind3 🌐 | likely | Handles the result code param_5 passed from caller 00b1ee10 while dispatching a parsed GuildPaidRenameCheckReplyPacket. It maps 3->2, 2->4, 4->-4, and 0/1->-9,… |

### `net.packet.objective`

**ObjectiveClientCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cff7d0` | ObjectiveClientCompletePacket::ObjectiveClientCompletePacket 🌐 | confirmed | Calls the objective packet base constructor with subopcode 7, installs the derived packet vtable, and stores the supplied objective identifier at object offset… |
| `00cff890` | ObjectiveClientCompletePacket::ScalarDeletingDtor | confirmed | Restores the BasePacket vtable and frees the packet object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |

**ObjectiveUIEventPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009afc10` | ObjectiveUIEventPacket::Ctor | confirmed | Calls objective base constructor with value 10, installs packet vtable, sets field +0x0c to -1, and initializes a string member. |
| `009b0ac0` | ObjectiveUIEventPacket_ScalarDeletingDtor | confirmed | Calls ObjectiveUIEventPacket destructor and conditionally frees object when deletion flag bit zero is set. |
| `009afc80` | ObjectiveUIEventPacket::Dtor | likely | Restores IString and BasePacket vtables, releases the owned string buffer, and returns. |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009be130` | DispatchObjectivePacket | confirmed | Parses the objective-family header and dispatches sub-opcodes 1-6 and 8-12: it reads the matching packet body, calls the associated local objective/UI/movement… |
| `00cffba0` | SendObjectiveCompletion 🌐 | confirmed | Constructs an ObjectiveClientCompletePacket with the stored objective identifier, sends it through the gateway tunnel when the owner has a gateway pointer, the… |

### `net.packet.player_title`

**BasePlayerTitlePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b49140` | BasePlayerTitlePacket::CtorDefault | confirmed | Default constructor: stores BasePacket::vftable (018011ec), writes the 32-bit opcode 0x98 (152) at +4, stores BasePlayerTitlePacket::vftable (01821254), and ze… |
| `00b49210` | BasePlayerTitlePacket::Dtor | confirmed | Destructor body: rewrites the object's vtable from BasePlayerTitlePacket::vftable (01821254) to BasePacket::vftable (018011ec); no members are released and no … |

**PlayerTitleAddPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b492d0` | PlayerTitleAddPacket::Ctor | confirmed | Constructs the base with sub-opcode 1 through FUN_00b491a0, stores PlayerTitleAddPacket::vftable (0182125c) and zeroes the four payload words at +0xc, +0x10, +… |
| `00b49340` | PlayerTitleAddPacket::Dtor | confirmed | Destructor body: restores the vtable chain PlayerTitleAddPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b493a0` | PlayerTitleAddPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set; otherwise returns this. |

**PlayerTitleRefreshRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b49680` | PlayerTitleRefreshRequestPacket::Ctor | confirmed | Constructs the base with sub-opcode 5 through FUN_00b491a0 and stores PlayerTitleRefreshRequestPacket::vftable (01821274); no further payload is initialised. |
| `00b496e0` | PlayerTitleRefreshRequestPacket::Dtor | confirmed | Destructor body: restores the vtable chain PlayerTitleRefreshRequestPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are releas… |
| `00b49740` | PlayerTitleRefreshRequestPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |

**PlayerTitleRemovePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b49410` | PlayerTitleRemovePacket::Ctor | confirmed | Constructs the base with sub-opcode 2 through FUN_00b491a0 and stores PlayerTitleRemovePacket::vftable (01821264); no payload fields are initialised beyond the… |
| `00b49470` | PlayerTitleRemovePacket::Dtor | confirmed | Destructor body: restores the vtable chain PlayerTitleRemovePacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b494d0` | PlayerTitleRemovePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |

**PlayerTitleRequestSelectPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b49540` | PlayerTitleRequestSelectPacket::Ctor | confirmed | Constructs the base with sub-opcode 4 through FUN_00b491a0, stores PlayerTitleRequestSelectPacket::vftable (0182126c) and writes its argument into the payload … |
| `00b495b0` | PlayerTitleRequestSelectPacket::Dtor | confirmed | Destructor body: restores the vtable chain PlayerTitleRequestSelectPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are release… |
| `00b49610` | PlayerTitleRequestSelectPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |

**PlayerTitleUpdateAllPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b497b0` | PlayerTitleUpdateAllPacket::Ctor | confirmed | Constructs the base with sub-opcode 3 through FUN_00b491a0 and stores PlayerTitleUpdateAllPacket::vftable (0182127c); no further payload is initialised. |
| `00b49810` | PlayerTitleUpdateAllPacket::Dtor | confirmed | Destructor body: restores the vtable chain PlayerTitleUpdateAllPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b49870` | PlayerTitleUpdateAllPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b4a3a0` | HandlePlayerTitlePacket 🌐 | confirmed | Handles opcode 152 (BasePlayerTitlePacket) from the network dispatcher: constructs the base packet with FUN_00b49140, reads the shared 16-bit + 32-bit header w… |
| `00b4a880` | SendPlayerTitleRequestSelect 🌐 | confirmed | Constructs a PlayerTitleRequestSelectPacket (FUN_00b49540 with the title id from param_2), then, when the gateway pointer [*(this+0)+0x280] is non-null, sends … |
| `00b4a910` | SendPlayerTitleRefreshRequest 🌐 | confirmed | Constructs a PlayerTitleRefreshRequestPacket (FUN_00b49680 with sub-opcode 5) in a 12-byte local and, when the gateway pointer [*(param_1)+0x280] is non-null, … |

### `net.packet.player_update`

**BasePlayerUpdatePacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089fbb0` | BasePlayerUpdatePacket::BasePlayerUpdatePacket 🌐 | confirmed | Family constructor: installs BasePacket::vftable, stores 0x23 (opcode 35) into the packet id slot param_1[1], installs BasePlayerUpdatePacket::vftable, then st… |
| `0089fc20` | BasePlayerUpdatePacket::~BasePlayerUpdatePacket | confirmed | Destructor body: restores this->vtable to BasePlayerUpdatePacket::vftable and then to BasePacket::vftable (base destructor inlined). Touches no data fields and… |
| `008eb030` | BasePlayerUpdatePacketDtorWithAnimationList | confirmed | Destroys embedded animation list, then restores BasePlayerUpdatePacket and BasePacket vtables. |
| `0089fb50` | BasePlayerUpdatePacket::CtorWithZeroSubOpcode 🌐 | likely | Family constructor with a fixed zero sub-opcode: installs BasePacket::vftable, writes 0x23 (35) into param_1[1], installs BasePlayerUpdatePacket::vftable and c… |
| `0089fc70` | BasePlayerUpdatePacket::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: restores BasePlayerUpdatePacket then BasePacket vtables inside an SEH frame and, when the flag byte param_2 has bit 0 set, cal… |

**HudMessagePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1900` | HudMessagePacket::~HudMessagePacket | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x40 and also runs from EH funclet 0… |
| `008b5aa0` | HudMessagePacket::HudMessagePacket 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x40), installs HudMessagePacket::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 an… |
| `008b5b30` | HudMessagePacket::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayClient** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090de90` | PlayClient::HandlePlayerUpdatePosition | confirmed | Parses a position update, resolves its GUID, ignores self and actors with mismatched area identifiers, then applies position by movement type and updates attac… |
| `0090e140` | PlayClient::HandlePlayerUpdatePositionOnPlatform | confirmed | Parses platform-position fields, resolves an actor, combines the received transform with its platform transform, updates the actor's platform state and motion,… |
| `0090e470` | PlayClient::HandlePlayerUpdateJump | confirmed | Constructs a jump update object, parses its actor identifier and motion fields, rejects missing or self actors, resets attached-platform fields, then applies t… |

**PlayerMemberStatusUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1950` | PlayerMemberStatusUpdatePacket::~PlayerMemberStatusUpdatePa… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x42 and also runs from EH funclet 0… |
| `008b5ba0` | PlayerMemberStatusUpdatePacket::PlayerMemberStatusUpdatePac… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x42), installs PlayerMemberStatusUpdatePacket::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into … |
| `008b5c10` | PlayerMemberStatusUpdatePacket::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdateLaunchProjectilePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d0b10` | PlayerUpdateLaunchProjectilePacket::PlayerUpdateLaunchProje… 🌐 | confirmed | Constructor of the LaunchProjectile player-update packet: calls the family constructor FUN_0089fbb0 with sub-opcode 0x3e (62), installs PlayerUpdateLaunchProje… |
| `008d0b80` | PlayerUpdateLaunchProjectilePacket::~PlayerUpdateLaunchProj… | likely | Destructor body of the LaunchProjectile player-update packet: runs the accepted ProjectileParameters destructor FUN_008d0240 on the embedded payload record, th… |

**PlayerUpdatePacketAddEffectTagCompositeEffect** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1380` | PlayerUpdatePacketAddEffectTagCompositeEffect::~PlayerUpdat… | confirmed | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x29 and also runs from EH funclet 0… |
| `008c4d50` | PlayerUpdatePacketAddEffectTagCompositeEffect::Constructor 🌐 | confirmed | PlayClient case 0x29 constructs this packet. Base constructor sets family opcode 0x23/subcode 0x29; this body installs its vtable and initializes identifier an… |
| `008c4df0` | PlayerUpdatePacketAddEffectTagCompositeEffect::ScalarDeleti… | confirmed | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |

**PlayerUpdatePacketAddNotifications** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d06d0` | PlayerUpdatePacketAddNotifications::PlayerUpdatePacketAddNo… 🌐 | confirmed | Constructor of the AddNotifications player-update packet: runs the family constructor FUN_0089fbb0 with subtype 10, installs PlayerUpdatePacketAddNotifications… |
| `008d0740` | PlayerUpdatePacketAddNotifications::~PlayerUpdatePacketAddN… | confirmed | Destructor of the AddNotifications packet: zeroes the array count at +0x14, rewrites the SoeUtil::Array<NotificationInfo,0,1> vtable at +0xc, frees the array b… |

**PlayerUpdatePacketAddNpc** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091cac0` | PlayerUpdatePacketAddNpc::Ctor 🌐 | confirmed | Constructor of the AddNpc packet: runs BasePlayerUpdatePacket::BasePlayerUpdatePacket (FUN_0089fbb0, opcode 0x23 stored by the base), installs PlayerUpdatePack… |
| `00921a10` | PlayerUpdatePacketAddNpc_ReadBody 🌐 | confirmed | Parses the AddNpc body from the bounded reader param_2 into the record at param_1: first reads two dwords into +0x10/+0x14, then dwords, bytes and floats in or… |
| `00922960` | PlayerUpdatePacketAddNpc_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 0091cdd0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `0091cdd0` | PlayerUpdatePacketAddNpc::~PlayerUpdatePacketAddNpc 🌐 | likely | Destructor of the AddNpc packet: releases the SoeUtil::IString fields at +0x230/+0x220/+0x210, tears down the CharacterVariables subobject at +0x188 (vtable re… |

**PlayerUpdatePacketAddPc** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00922980` | PlayerUpdatePacketAddPc_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 0091d1c0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `0091d1c0` | PlayerUpdatePacketAddPc::~PlayerUpdatePacketAddPc 🌐 | likely | Destructor of the AddPc packet: drains the EffectTag list at +0x180 (FUN_008cace0), tears down the member at +0x170 (FUN_008eaf50), releases the six IString fi… |

**PlayerUpdatePacketCameraUpdate** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0160` | PlayerUpdatePacketCameraUpdate::PlayerUpdatePacketCameraUpd… | likely | Constructor of an embedded member object: installs BasePacket::vftable, writes 0x7e (126) into the id slot param_1[1] and installs PlayerUpdatePacketCameraUpda… |
| `008a01c0` | PlayerUpdatePacketCameraUpdate::~PlayerUpdatePacketCameraUp… | likely | Destructor body: restores PlayerUpdatePacketCameraUpdate::vftable then BasePacket::vftable. Called only from EH unwind funclets (01592e2c, 01592f14). |
| `008a0210` | PlayerUpdatePacketCameraUpdate::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores PlayerUpdatePacketCameraUpdate and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |

**PlayerUpdatePacketClearMovementRail** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a15f0` | PlayerUpdatePacketClearMovementRail::~PlayerUpdatePacketCle… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x36 and also runs from EH funclet 0… |
| `008b54f0` | PlayerUpdatePacketClearMovementRail::PlayerUpdatePacketClea… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x36), installs PlayerUpdatePacketClearMovementRail::vftable and stores param_2 and param_3 into +0x10 and +0x1… |
| `008b5560` | PlayerUpdatePacketClearMovementRail::ScalarDeletingDestruct… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketCustomizationChange** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091d5f0` | PlayerUpdatePacketCustomizationChange::Ctor 🌐 | confirmed | Constructor: runs FUN_0089fbb0(0x27) (BasePlayerUpdatePacket with sub-opcode 39), installs PlayerUpdatePacketCustomizationChange::vftable, sets up the SoeUtil:… |
| `0091d660` | PlayerUpdatePacketCustomizationChange::~PlayerUpdatePacketC… 🌐 | confirmed | Destructor: drains the SoeUtil::List<PlayerCustomizationData> at +0x20 with FUN_008fbe20 and restores BasePlayerUpdatePacket::vftable then BasePacket::vftable. |
| `0091d6f0` | PlayerUpdatePacketCustomizationChange::ScalarDeletingDtor 🌐 | confirmed | Scalar deleting destructor: calls the packet destructor 0091d660 and frees this when bit 0 of the flag argument is set. |

**PlayerUpdatePacketCustomizationData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091d540` | PlayerUpdatePacketCustomizationData::~PlayerUpdatePacketCus… 🌐 | confirmed | Destructor body: drains the SoeUtil::List<PlayerCustomizationData> at +0x10 with FUN_008fbe20 and restores PlayerUpdatePacketCustomizationData::vftable, BasePl… |
| `0091d5d0` | PlayerUpdatePacketCustomizationData::ScalarDeletingDtor 🌐 | confirmed | Scalar deleting destructor: calls the packet destructor 0091d540 and frees this when bit 0 of the flag argument is set. |

**PlayerUpdatePacketCustomizeNpc** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d0330` | PlayerUpdatePacketCustomizeNpc::PlayerUpdatePacketCustomize… | confirmed | Constructor of the CustomizeNpc player-update packet: runs the family constructor FUN_0089fbb0 with the stack immediate 0x2e (46) as the sub-opcode, installs P… |

**PlayerUpdatePacketDestroyed** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c4f30` | PlayerUpdatePacketDestroyed::Constructor 🌐 | confirmed | PlayClient case 0x38 constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x38; this body installs its vtable, copies defaults into +0x10/… |
| `008c4fb0` | PlayerUpdatePacketDestroyed::ScalarDeletingDestructor | confirmed | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
| `008a16f0` | PlayerUpdatePacketDestroyed::~PlayerUpdatePacketDestroyed | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x38 and also runs from EH funclet 0… |

**PlayerUpdatePacketEffectTagCompositeEffectsEnable** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1420` | PlayerUpdatePacketEffectTagCompositeEffectsEnable 🌐 | confirmed | Sub-opcode-43 constructor: calls FUN_0089fbb0(0x2b) (opcode 35, sub-opcode 43), installs its own vftable and zeroes one byte at param_1+0xc (param_1[3] as a by… |
| `008a1480` | PlayerUpdatePacketEffectTagCompositeEffectsEnable::Dtor | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2b and also runs from EH funclet 0… |
| `008b52a0` | PlayerUpdatePacketEffectTagCompositeEffectsEnable::ScalarDe… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketEquipItemChange** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d05d0` | PlayerUpdatePacketEquipItemChange::~PlayerUpdatePacketEquip… | confirmed | Destructor of the EquipItemChange packet: rewrites PlayerUpdatePacketEquipItemChange::vftable, runs the SoeUtil::IStringFixed<char,128> destructor body FUN_007… |
| `008d0640` | PlayerUpdatePacketEquipItemChange::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the EquipItemChange destructor body FUN_008d05d0 and frees the object with _free when bit 0 of the flag argument is set. |

**PlayerUpdatePacketEquippedItemsChange** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091d3b0` | PlayerUpdatePacketEquippedItemsChange::Ctor 🌐 | confirmed | Constructor: runs FUN_0089fbb0(7) (BasePlayerUpdatePacket with sub-opcode 7) and installs PlayerUpdatePacketEquippedItemsChange::vftable plus the SoeUtil::List… |
| `0091d420` | PlayerUpdatePacketEquippedItemsChange::~PlayerUpdatePacketE… 🌐 | confirmed | Destructor body: drains the SoeUtil::List<CharacterAttachmentData> at +0x1c through FUN_0070e670 and restores PlayerUpdatePacketEquippedItemsChange::vftable, B… |
| `0091d4b0` | PlayerUpdatePacketEquippedItemsChange::ScalarDeletingDtor 🌐 | confirmed | Scalar deleting destructor: calls the packet destructor 0091d420 and frees this when bit 0 of the flag argument is set. |

**PlayerUpdatePacketExpectedSpeed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0280` | PlayerUpdatePacketExpectedSpeed::~PlayerUpdatePacketExpecte… | confirmed | Destructor body: restores PlayerUpdatePacketExpectedSpeed::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a02e0` | PlayerUpdatePacketExpectedSpeed::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the ExpectedSpeed, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bi… |

**PlayerUpdatePacketFlourish** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3aa0` | PlayerUpdatePacketFlourish::PlayerUpdatePacketFlourish | confirmed | Stores BasePacket::vftable, writes packet id 0xb5 (181) into +4, installs PlayerUpdatePacketFlourish::vftable and copies the global pair DAT_01be4850/DAT_01be4… |

**PlayerUpdatePacketFreeze** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0f00` | PlayerUpdatePacketFreeze::PlayerUpdatePacketFreeze 🌐 | confirmed | Sub-opcode-32 constructor: calls the family constructor FUN_0089fbb0(0x20), which writes opcode 35 and sub-opcode 32 into the header slots, then installs Playe… |
| `008a0f60` | PlayerUpdatePacketFreeze::~PlayerUpdatePacketFreeze | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x20 and also runs from EH funclet 0… |
| `008b4c60` | PlayerUpdatePacketFreeze::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketHeadInflationScale** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0e60` | PlayerUpdatePacketHeadInflationScale::~PlayerUpdatePacketHe… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1e and EH funclet 01591464. |
| `008b4a80` | PlayerUpdatePacketHeadInflationScale::PlayerUpdatePacketHea… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x1e), installs PlayerUpdatePacketHeadInflationScale::vftable and stores param_2, param_3 and param_4 into +0x1… |
| `008b4b00` | PlayerUpdatePacketHeadInflationScale::ScalarDeletingDestruc… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketHitPointAndShieldsModification** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4e10` | PlayerUpdatePacketHitPointAndShieldsModification::Ctor 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x3a) and installs PlayerUpdatePacketHitPointAndShieldsModification::vftable; no field is initialised. |
| `008b4e70` | PlayerUpdatePacketHitPointAndShieldsModification::ScalarDel… | confirmed | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |
| `008a10b0` | PlayerUpdatePacketHitPointAndShieldsModification::Dtor | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x3a and also runs from EH funclet 0… |

**PlayerUpdatePacketHitPointModification** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1060` | PlayerUpdatePacketHitPointModification::~PlayerUpdatePacket… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x23 and also runs from EH funclet 0… |
| `008b4d40` | PlayerUpdatePacketHitPointModification::PlayerUpdatePacketH… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x23) and installs PlayerUpdatePacketHitPointModification::vftable; no field is initialised. |
| `008b4da0` | PlayerUpdatePacketHitPointModification::ScalarDeletingDestr… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketItemDefinitionRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1100` | PlayerUpdatePacketItemDefinitionRequest::PlayerUpdatePacket… 🌐 | confirmed | Sub-opcode-34 constructor taking an id: calls FUN_0089fbb0(0x22) (opcode 35, sub-opcode 34), zeroes param_1[3] and param_1[4], installs PlayerUpdatePacketItemD… |
| `008a1170` | PlayerUpdatePacketItemDefinitionRequest::~PlayerUpdatePacke… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x22 and is also called by the reque… |
| `008b4ee0` | PlayerUpdatePacketItemDefinitionRequest::ScalarDeletingDest… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketItemDefinitions** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1210` | PlayerUpdatePacketItemDefinitions::PlayerUpdatePacketItemDe… 🌐 | confirmed | Sub-opcode-37 constructor: calls FUN_0089fbb0(0x25) (opcode 35, sub-opcode 37), installs PlayerUpdatePacketItemDefinitions::vftable and zeroes param_1[3] and p… |
| `008a1270` | PlayerUpdatePacketItemDefinitions::~PlayerUpdatePacketItemD… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x25 and also runs from EH funclet 0… |
| `008b5030` | PlayerUpdatePacketItemDefinitions::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketJump** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0090` | PlayerUpdatePacketJump::~PlayerUpdatePacketJump | confirmed | Destructor body: restores PlayerUpdatePacketJump::vftable then BasePacket::vftable. |
| `008a00f0` | PlayerUpdatePacketJump::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores PlayerUpdatePacketJump and BasePacket vtables in an SEH frame and calls _free(this) when param_2 bit 0 is set. |

**PlayerUpdatePacketKnockback** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a08b0` | PlayerUpdatePacketKnockback::~PlayerUpdatePacketKnockback | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 4 and EH funclet 01591590. |
| `008b4190` | PlayerUpdatePacketKnockback::PlayerUpdatePacketKnockback 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(4), installs PlayerUpdatePacketKnockback::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/… |
| `008b4210` | PlayerUpdatePacketKnockback::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketLootEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d0a00` | PlayerUpdatePacketLootEvent::PlayerUpdatePacketLootEvent 🌐 | confirmed | Constructor of the LootEvent player-update packet: calls the family constructor FUN_0089fbb0 with sub-opcode 0x1d (29), installs PlayerUpdatePacketLootEvent::v… |
| `008d0a80` | PlayerUpdatePacketLootEvent::~PlayerUpdatePacketLootEvent | likely | Destructor body of the LootEvent player-update packet: rewrites the embedded SoeUtil::IString<char> member at +0x30 (param_1[0xc]) to its vtable, LOCK-decremen… |

**PlayerUpdatePacketMoveOnRail** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1570` | PlayerUpdatePacketMoveOnRail::~PlayerUpdatePacketMoveOnRail | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x35 and also runs from EH funclet 0… |
| `008b53e0` | PlayerUpdatePacketMoveOnRail::PlayerUpdatePacketMoveOnRail 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x35), installs PlayerUpdatePacketMoveOnRail::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0… |
| `008b5460` | PlayerUpdatePacketMoveOnRail::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketMoveOnRelativeRail** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b5670` | PlayerUpdatePacketMoveOnRelativeRail::ScalarDeletingDestruc… | confirmed | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |
| `008a1640` | PlayerUpdatePacketMoveOnRelativeRail::~PlayerUpdatePacketMo… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x37 and also runs from EH funclet 0… |
| `008b55f0` | PlayerUpdatePacketMoveOnRelativeRail::PlayerUpdatePacketMov… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x37), installs PlayerUpdatePacketMoveOnRelativeRail::vftable, stores the global pair DAT_01be4850/DAT_01be4854… |

**PlayerUpdatePacketNpcRelevance** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d08b0` | PlayerUpdatePacketNpcRelevance::PlayerUpdatePacketNpcReleva… 🌐 | confirmed | Constructor of the NpcRelevance player-update packet: runs the family constructor FUN_0089fbb0 with subtype 0xc (12), installs PlayerUpdatePacketNpcRelevance::… |
| `008d0920` | PlayerUpdatePacketNpcRelevance::~PlayerUpdatePacketNpcRelev… | confirmed | Destructor body of the NpcRelevance player-update packet: rewrites the embedded SoeUtil::Array<PlayerUpdatePacketNpcRelevance::Entry> vtable at +0xc (param_1[3… |

**PlayerUpdatePacketPlayCompositeEffect** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c4b30` | PlayerUpdatePacketPlayCompositeEffect::Constructor 🌐 | confirmed | PlayClient case 0x10 constructs this packet. Base constructor sets family opcode 0x23/subcode 0x10; this body installs its vtable, copies defaults into +0x10..… |
| `008c4bd0` | PlayerUpdatePacketPlayCompositeEffect::ScalarDeletingDestru… | confirmed | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
| `008a0af0` | PlayerUpdatePacketPlayCompositeEffect::~PlayerUpdatePacketP… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x10 and EH funclet 015915c2. |

**PlayerUpdatePacketPlayerTitle** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a12c0` | PlayerUpdatePacketPlayerTitle::~PlayerUpdatePacketPlayerTit… | confirmed | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x28 and also runs from EH funclet 0… |
| `008b50a0` | PlayerUpdatePacketPlayerTitle::PlayerUpdatePacketPlayerTitle 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x28), installs PlayerUpdatePacketPlayerTitle::vftable and zeroes +0x18, +0x1c, +0x20 and +0x24. |
| `008b5110` | PlayerUpdatePacketPlayerTitle::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketPopup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b5ca0` | PlayerUpdatePacketPopup::PlayerUpdatePacketPopup 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x46), installs PlayerUpdatePacketPopup::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+… |
| `008b5d20` | PlayerUpdatePacketPopup::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketPreferredLanguages** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a11c0` | PlayerUpdatePacketPreferredLanguages::~PlayerUpdatePacketPr… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x26 and also runs from EH funclet 0… |
| `008b4f50` | PlayerUpdatePacketPreferredLanguages::PlayerUpdatePacketPre… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x26), installs PlayerUpdatePacketPreferredLanguages::vftable, stores the global pair DAT_01be4850/DAT_01be4854… |
| `008b4fc0` | PlayerUpdatePacketPreferredLanguages::ScalarDeletingDestruc… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketQueueAnimation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0860` | PlayerUpdatePacketQueueAnimation::~PlayerUpdatePacketQueueA… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x16 and EH funclet 01591585. |
| `008b4090` | PlayerUpdatePacketQueueAnimation::PlayerUpdatePacketQueueAn… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x16), installs PlayerUpdatePacketQueueAnimation::vftable, zeroes +0x10..+0x1c, +0x24, +0x28 and the byte at +0… |
| `008b4120` | PlayerUpdatePacketQueueAnimation::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketRemoveEffectTagCompositeEffect** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a13d0` | PlayerUpdatePacketRemoveEffectTagCompositeEffect::Dtor | confirmed | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2a and also runs from EH funclet 0… |
| `008b51c0` | PlayerUpdatePacketRemoveEffectTagCompositeEffect::Ctor 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x2a), installs PlayerUpdatePacketRemoveEffectTagCompositeEffect::vftable, stores the global pair DAT_01be4850/… |
| `008b5230` | PlayerUpdatePacketRemoveEffectTagCompositeEffect::ScalarDel… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketRemoveNotifications** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d07c0` | PlayerUpdatePacketRemoveNotifications::PlayerUpdatePacketRe… 🌐 | confirmed | Constructor of the RemoveNotifications player-update packet: runs the family constructor FUN_0089fbb0 with subtype 0xb (11), installs PlayerUpdatePacketRemoveN… |
| `008d0830` | PlayerUpdatePacketRemoveNotifications::~PlayerUpdatePacketR… | confirmed | Destructor of the RemoveNotifications packet: zeroes the array count at +0x14, rewrites the SoeUtil::Array<PlayerUpdatePacketRemoveNotifications::Entry,0,1> vt… |

**PlayerUpdatePacketRemoveNpcCustomization** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d0400` | PlayerUpdatePacketRemoveNpcCustomization::PlayerUpdatePacke… | confirmed | Constructor of the RemoveNpcCustomization player-update packet: runs the family constructor FUN_0089fbb0 with the stack immediate 0x30 (48) as the sub-opcode, … |

**PlayerUpdatePacketRemovePlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089fce0` | PlayerUpdatePacketRemovePlayer::~PlayerUpdatePacketRemovePl… | confirmed | Destructor body of the sub-opcode-3 packet: restores PlayerUpdatePacketRemovePlayer::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. N… |
| `0089fd40` | PlayerUpdatePacketRemovePlayer::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores PlayerUpdatePacketRemovePlayer, BasePlayerUpdatePacket and BasePacket vtables in an SEH frame, and calls _free(this) when … |

**PlayerUpdatePacketRemovePlayerGracefully** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089fdb0` | PlayerUpdatePacketRemovePlayerGracefully::~PlayerUpdatePack… | confirmed | Destructor body of the sub-opcode-1 packet: restores PlayerUpdatePacketRemovePlayerGracefully::vftable, then PlayerUpdatePacketRemovePlayer::vftable, then Base… |
| `008c4ad0` | PlayerUpdatePacketRemovePlayerGracefully::Constructor 🌐 | confirmed | Calls 0x008b38a0 with this in ECX and stack argument 1, installs PlayerUpdatePacketRemovePlayerGracefully vtable, zeros dword +0x24, and returns this. Its call… |
| `0089fe10` | PlayerUpdatePacketRemovePlayerGracefully::ScalarDeletingDes… | likely | Thin deleting destructor: calls FUN_0089fdb0 on this and then, when param_2 bit 0 is set, calls _free(this). |

**PlayerUpdatePacketRemoveTemporaryAppearance** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a09d0` | PlayerUpdatePacketRemoveTemporaryAppearance::~PlayerUpdateP… | confirmed | Destructor body: restores PlayerUpdatePacketRemoveTemporaryAppearance::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b4310` | PlayerUpdatePacketRemoveTemporaryAppearance::PlayerUpdatePa… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0xf) and installs PlayerUpdatePacketRemoveTemporaryAppearance::vftable; no field is initialised. |
| `008a0a30` | PlayerUpdatePacketRemoveTemporaryAppearance::ScalarDeleting… | likely | Scalar deleting destructor: restores the RemoveTemporaryAppearance, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) whe… |

**PlayerUpdatePacketRenamePlayer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d09a0` | PlayerUpdatePacketRenamePlayer::~PlayerUpdatePacketRenamePl… | likely | Destructor body of a BasePlayerUpdatePacket-derived packet: destroys the member object at this+0x18 through FUN_0070a790, then rewrites the object vtable to Ba… |

**PlayerUpdatePacketReplaceBaseModel** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c5020` | PlayerUpdatePacketReplaceBaseModel::Constructor 🌐 | confirmed | PlayClient case 0x31 constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x31; this body installs its vtable and zeros dword +0x1c. |
| `008c5080` | PlayerUpdatePacketReplaceBaseModel::ScalarDeletingDestructor | confirmed | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
| `008a1850` | PlayerUpdatePacketReplaceBaseModel::~PlayerUpdatePacketRepl… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x31 and also runs from EH funclet 0… |

**PlayerUpdatePacketRequestStripEffect** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0fb0` | PlayerUpdatePacketRequestStripEffect::PlayerUpdatePacketReq… 🌐 | confirmed | Sub-opcode-33 constructor: calls FUN_0089fbb0(0x21) so the packet header carries opcode 35 and sub-opcode 33, then installs PlayerUpdatePacketRequestStripEffec… |
| `008a1010` | PlayerUpdatePacketRequestStripEffect::~PlayerUpdatePacketRe… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; its only caller is EH unwind funclet 01591880. |
| `008b4cd0` | PlayerUpdatePacketRequestStripEffect::ScalarDeletingDestruc… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketScriptedAnimation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0810` | PlayerUpdatePacketScriptedAnimation::~PlayerUpdatePacketScr… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x18 and EH funclet 015916b4. |
| `008b3fc0` | PlayerUpdatePacketScriptedAnimation::PlayerUpdatePacketScri… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x18) and installs PlayerUpdatePacketScriptedAnimation::vftable; nothing else is written. |
| `008b4020` | PlayerUpdatePacketScriptedAnimation::ScalarDeletingDestruct… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSeekTarget** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1750` | PlayerUpdatePacketSeekTarget::~PlayerUpdatePacketSeekTarget | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x3b and also runs from EH funclet 0… |
| `008b57e0` | PlayerUpdatePacketSeekTarget::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSeekTargetUpdate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1800` | PlayerUpdatePacketSeekTargetUpdate::~PlayerUpdatePacketSeek… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x3c and also runs from EH funclet 0… |
| `008b5910` | PlayerUpdatePacketSeekTargetUpdate::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSetAnimation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3ee0` | PlayerUpdatePacketSetAnimation::PlayerUpdatePacketSetAnimat… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(8) and installs PlayerUpdatePacketSetAnimation::vftable; nothing else is written. |
| `008a07c0` | PlayerUpdatePacketSetAnimation::~PlayerUpdatePacketSetAnima… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; no fields, no deallocation. Called from the opcode-35 dispatcher case 8 and… |
| `008b3f40` | PlayerUpdatePacketSetAnimation::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSetCollidable** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b59c0` | PlayerUpdatePacketSetCollidable::PlayerUpdatePacketSetColli… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x32) and installs PlayerUpdatePacketSetCollidable::vftable; no field is initialised. |
| `008a18a0` | PlayerUpdatePacketSetCollidable::~PlayerUpdatePacketSetColl… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x32 and also runs from EH funclet 0… |
| `008b5a20` | PlayerUpdatePacketSetCollidable::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSetDisposition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4980` | PlayerUpdatePacketSetDisposition::PlayerUpdatePacketSetDisp… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x1c), installs PlayerUpdatePacketSetDisposition::vftable and stores param_2, param_3 and param_4 into +0x10, +… |
| `008b4a10` | PlayerUpdatePacketSetDisposition::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSetLookAt** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4440` | PlayerUpdatePacketSetLookAt::PlayerUpdatePacketSetLookAt 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x11) and installs PlayerUpdatePacketSetLookAt::vftable; no field is initialised. |
| `008a0b40` | PlayerUpdatePacketSetLookAt::~PlayerUpdatePacketSetLookAt | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x11 and EH funclet 015915cd. |
| `008b44a0` | PlayerUpdatePacketSetLookAt::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSetSpawnAnimation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b5310` | PlayerUpdatePacketSetSpawnAnimation::PlayerUpdatePacketSetS… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x2d) and installs PlayerUpdatePacketSetSpawnAnimation::vftable; no field is initialised. |
| `008a14d0` | PlayerUpdatePacketSetSpawnAnimation::~PlayerUpdatePacketSet… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2d and also runs from EH funclet 0… |
| `008b5370` | PlayerUpdatePacketSetSpawnAnimation::ScalarDeletingDestruct… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketSetSpawnerActivationEffect** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c4e60` | PlayerUpdatePacketSetSpawnerActivationEffect::Constructor 🌐 | confirmed | PlayClient case 0x2f constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x2f; this body installs its vtable and zeros dword +0x18. |
| `008c4ec0` | PlayerUpdatePacketSetSpawnerActivationEffect::ScalarDeletin… | confirmed | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
| `008a1520` | PlayerUpdatePacketSetSpawnerActivationEffect::~PlayerUpdate… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2f and also runs from EH funclet 0… |

**PlayerUpdatePacketSetSynchronizedAnimations** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008eafc0` | PlayerUpdatePacketSetSynchronizedAnimationsCtor 🌐 | confirmed | Constructs base player-update packet with family 35 and sub-opcode 63, installs derived vtable and initializes animation list. |
| `008fde40` | PlayerUpdatePacketSetSynchronizedAnimations::ScalarDeleting… | confirmed | thiscall with one byte argument (delete flags, ret 4). Calls 008eb030 on this (the destructor body, unlabelled), then if bit 0 of the flag is set calls the _fr… |

**PlayerUpdatePacketSlotCompositeEffectOverride** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0eb0` | PlayerUpdatePacketSlotCompositeEffectOverride::~PlayerUpdat… | confirmed | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1f and EH funclet 015914c7. |
| `008b4b70` | PlayerUpdatePacketSlotCompositeEffectOverride::PlayerUpdate… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x1f), installs PlayerUpdatePacketSlotCompositeEffectOverride::vftable and stores param_2..param_5 into +0x10..… |
| `008b4bf0` | PlayerUpdatePacketSlotCompositeEffectOverride::ScalarDeleti… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketThoughtBubble** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0db0` | PlayerUpdatePacketThoughtBubble::~PlayerUpdatePacketThought… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1b and EH funclet 015916bf. |
| `008b4870` | PlayerUpdatePacketThoughtBubble::PlayerUpdatePacketThoughtB… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x1b), installs PlayerUpdatePacketThoughtBubble::vftable, stores param_2..param_5 into +0x10..+0x1c and the byt… |
| `008b4910` | PlayerUpdatePacketThoughtBubble::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketUpdateActiveWieldType** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a06f0` | PlayerUpdatePacketUpdateActiveWieldType::~PlayerUpdatePacke… | confirmed | Destructor body: restores PlayerUpdatePacketUpdateActiveWieldType::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b3e80` | PlayerUpdatePacketUpdateActiveWieldType::PlayerUpdatePacket… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x3d) and installs PlayerUpdatePacketUpdateActiveWieldType::vftable; nothing else is written. |
| `008a0750` | PlayerUpdatePacketUpdateActiveWieldType::ScalarDeletingDest… | likely | Scalar deleting destructor: restores the wield-type, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0… |

**PlayerUpdatePacketUpdateCharacterState** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0b90` | PlayerUpdatePacketUpdateCharacterState::~PlayerUpdatePacket… | confirmed | Destructor body: restores PlayerUpdatePacketUpdateCharacterState::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b4510` | PlayerUpdatePacketUpdateCharacterState::PlayerUpdatePacketU… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x14) and installs PlayerUpdatePacketUpdateCharacterState::vftable; no field is initialised. |
| `008a0bf0` | PlayerUpdatePacketUpdateCharacterState::ScalarDeletingDestr… | likely | Scalar deleting destructor: restores the character-state, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 … |

**PlayerUpdatePacketUpdateHitpoints** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a03b0` | PlayerUpdatePacketUpdateHitpoints::~PlayerUpdatePacketUpdat… | confirmed | Destructor body: restores PlayerUpdatePacketUpdateHitpoints::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b3d00` | PlayerUpdatePacketUpdateHitpoints::PlayerUpdatePacketUpdate… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(5) and installs PlayerUpdatePacketUpdateHitpoints::vftable; nothing else is written. |
| `008a0410` | PlayerUpdatePacketUpdateHitpoints::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the hit-point, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 … |

**PlayerUpdatePacketUpdateIdleAnim** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4770` | PlayerUpdatePacketUpdateIdleAnim::PlayerUpdatePacketUpdateI… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x1a), installs PlayerUpdatePacketUpdateIdleAnim::vftable and stores param_2, param_3 and param_4 into +0x10, +… |
| `008a0d30` | PlayerUpdatePacketUpdateIdleAnim::~PlayerUpdatePacketUpdate… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1a and EH funclet 015916a9. |
| `008b4800` | PlayerUpdatePacketUpdateIdleAnim::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketUpdateLivesRemaining** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0620` | PlayerUpdatePacketUpdateLivesRemaining::~PlayerUpdatePacket… | confirmed | Destructor body: restores PlayerUpdatePacketUpdateLivesRemaining::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b3e20` | PlayerUpdatePacketUpdateLivesRemaining::PlayerUpdatePacketU… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x12) and installs PlayerUpdatePacketUpdateLivesRemaining::vftable; nothing else is written. |
| `008a0680` | PlayerUpdatePacketUpdateLivesRemaining::ScalarDeletingDestr… | likely | Scalar deleting destructor: restores the lives-remaining, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 … |

**PlayerUpdatePacketUpdateMana** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0480` | PlayerUpdatePacketUpdateMana::~PlayerUpdatePacketUpdateMana | confirmed | Destructor body: restores PlayerUpdatePacketUpdateMana::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b3d60` | PlayerUpdatePacketUpdateMana::PlayerUpdatePacketUpdateMana 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(9) and installs PlayerUpdatePacketUpdateMana::vftable; nothing else is written. |
| `008a04e0` | PlayerUpdatePacketUpdateMana::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the mana, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is se… |

**PlayerUpdatePacketUpdateOwner** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3c70` | PlayerUpdatePacketUpdateOwner::PlayerUpdatePacketUpdateOwner 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x33), installs PlayerUpdatePacketUpdateOwner::vftable, writes Target::vftable into +0x18, zeroes +0x1c and the… |
| `008a0350` | PlayerUpdatePacketUpdateOwner::~PlayerUpdatePacketUpdateOwn… | likely | Destructor body that also carries an SEH frame and a stack-cookie check: restores BasePlayerUpdatePacket::vftable and BasePacket::vftable, calls FUN_0101b370 w… |

**PlayerUpdatePacketUpdatePosition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089fe30` | PlayerUpdatePacketUpdatePosition::~PlayerUpdatePacketUpdate… | confirmed | Destructor body: restores PlayerUpdatePacketUpdatePosition::vftable then BasePacket::vftable; no fields touched, no deallocation. |
| `0089ff00` | PlayerUpdatePacketUpdatePosition::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores PlayerUpdatePacketUpdatePosition and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is se… |

**PlayerUpdatePacketUpdatePositionOnPlatform** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ff70` | PlayerUpdatePacketUpdatePositionOnPlatform::~PlayerUpdatePa… | confirmed | Destructor body: restores PlayerUpdatePacketUpdatePositionOnPlatform::vftable, then PlayerUpdatePacketUpdatePosition::vftable, then BasePacket::vftable. |
| `008b3a30` | PlayerUpdatePacketUpdatePositionOnPlatform::PlayerUpdatePac… 🌐 | confirmed | Runs the position-packet constructor 008b3910, installs PlayerUpdatePacketUpdatePositionOnPlatform::vftable over it, stores the global pair DAT_01be4850/DAT_01… |
| `0089ffd0` | PlayerUpdatePacketUpdatePositionOnPlatform::ScalarDeletingD… | likely | Scalar deleting destructor: restores the OnPlatform, UpdatePosition and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |

**PlayerUpdatePacketUpdateRunAnim** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4670` | PlayerUpdatePacketUpdateRunAnim::PlayerUpdatePacketUpdateRu… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x19), installs PlayerUpdatePacketUpdateRunAnim::vftable and stores param_2, param_3 and param_4 into +0x10, +0… |
| `008a0cd0` | PlayerUpdatePacketUpdateRunAnim::~PlayerUpdatePacketUpdateR… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x19 and EH funclet 0159169e. |
| `008b4700` | PlayerUpdatePacketUpdateRunAnim::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketUpdateScale** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0900` | PlayerUpdatePacketUpdateScale::~PlayerUpdatePacketUpdateSca… | confirmed | Destructor body: restores PlayerUpdatePacketUpdateScale::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b42b0` | PlayerUpdatePacketUpdateScale::PlayerUpdatePacketUpdateScale 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0xd) and installs PlayerUpdatePacketUpdateScale::vftable; no field is initialised. |
| `008a0960` | PlayerUpdatePacketUpdateScale::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the scale, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is s… |

**PlayerUpdatePacketUpdateShields** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a0550` | PlayerUpdatePacketUpdateShields::~PlayerUpdatePacketUpdateS… | confirmed | Destructor body: restores PlayerUpdatePacketUpdateShields::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008b3dc0` | PlayerUpdatePacketUpdateShields::PlayerUpdatePacketUpdateSh… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x39) and installs PlayerUpdatePacketUpdateShields::vftable; nothing else is written. |
| `008a05b0` | PlayerUpdatePacketUpdateShields::ScalarDeletingDestructor | likely | Scalar deleting destructor: restores the shields, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is… |

**PlayerUpdatePacketUpdateTemporaryAppearance** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4370` | PlayerUpdatePacketUpdateTemporaryAppearance::PlayerUpdatePa… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0xe) and installs PlayerUpdatePacketUpdateTemporaryAppearance::vftable; no field is initialised. |
| `008a0aa0` | PlayerUpdatePacketUpdateTemporaryAppearance::~PlayerUpdateP… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0xe, from 0096c300 and from three EH funclets. |
| `008b43d0` | PlayerUpdatePacketUpdateTemporaryAppearance::ScalarDeleting… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdatePacketUpdateTintAlias** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d04d0` | PlayerUpdatePacketUpdateTintAlias::PlayerUpdatePacketUpdate… 🌐 | confirmed | Constructor of the UpdateTintAlias player-update packet: runs the family constructor FUN_0089fbb0 with subtype 0x34 (52), installs PlayerUpdatePacketUpdateTint… |

**PlayerUpdatePacketUpdateWalkAnim** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b4570` | PlayerUpdatePacketUpdateWalkAnim::PlayerUpdatePacketUpdateW… 🌐 | confirmed | Calls the opcode-35 family constructor 0089fbb0(0x15), installs PlayerUpdatePacketUpdateWalkAnim::vftable and stores param_2, param_3 and param_4 into +0x10, +… |
| `008a0c70` | PlayerUpdatePacketUpdateWalkAnim::~PlayerUpdatePacketUpdate… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x15 and EH funclet 01591693. |
| `008b4600` | PlayerUpdatePacketUpdateWalkAnim::ScalarDeletingDestructor | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdateProfileNameplateImageIdPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1a00` | PlayerUpdateProfileNameplateImageIdPacket::~PlayerUpdatePro… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; both branches of dispatcher case 0x47 call it before the shared base-destru… |
| `008b5d90` | PlayerUpdateProfileNameplateImageIdPacket::PlayerUpdateProf… 🌐 | likely | Calls the opcode-35 family constructor 0089fbb0(0x47), installs PlayerUpdateProfileNameplateImageIdPacket::vftable, zeroes +0xc and stores the global pair DAT_… |
| `008b5e00` | PlayerUpdateProfileNameplateImageIdPacket::ScalarDeletingDe… | likely | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct … |

**PlayerUpdateTriggerEffectPackagePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c4c40` | PlayerUpdateTriggerEffectPackagePacket::Constructor 🌐 | confirmed | PlayClient case 0x24 constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x24; this body installs its vtable, copies defaults into +0x10.… |
| `008c4ce0` | PlayerUpdateTriggerEffectPackagePacket::ScalarDeletingDestr… | confirmed | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
| `008a1310` | PlayerUpdateTriggerEffectPackagePacket::~PlayerUpdateTrigge… | likely | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x24 and also runs from EH funclet 0… |

**(no class)** (51)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00934740` | SendStripEffectRequest 🌐 | confirmed | Constructs a PlayerUpdatePacketRequestStripEffect, copies param_2 to a local field, and sends it only when the gateway pointer at this+0x280 is non-null. |
| `0089ef60` | Wrapper_0089ef60 | likely | Fifteen-byte forwarding wrapper: its raw bytes are mov eax,[esp+4]; push 0; push eax; call 00782550; ret 4, so it calls FUN_00782550 with the caller's single a… |
| `008d0470` | PlayerUpdatePacketDtor_0777340_b | likely | Second emission of the player-update packet destructor: runs FUN_00777340 (releases the packet's embedded SoeUtil::IString at +0x28), then rewrites BasePlayerU… |
| `008d0540` | PlayerUpdatePacketDtor_WithIString18 | likely | Destructor of a BasePlayerUpdatePacket-derived packet that carries one string: releases the embedded SoeUtil::IString at +0x18 (capacity +0x24) through the loc… |
| `008e5940` | ReadNpcMovementOpcode125 | likely | thiscall: ECX = output record, one stack argument = bounded reader. Reads an int16 (sign-extended) to +4; an 8-byte value (two dwords) to +8; three float32 to … |
| `008e5ba0` | ReadPlayerUpdateJumpBody | likely | thiscall: ECX = record, one stack argument = bounded reader. Reads int16 (sign-extended) to +4; an 8-byte value to +8 and +0xc; float32 values to +0x10, +0x14,… |
| `008f75b0` | ReadBodyVia_008d6890 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7620` | ReadBodyVia_008d6900 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7690` | ReadBodyVia_008d6980 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7700` | ReadBodyVia_008d69f0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7770` | ReadBodyVia_008d6a60 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f77e0` | ReadBodyVia_008d6b40 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7850` | ReadBodyVia_008d6bc0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f78c0` | ReadBodyVia_008d6c50 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7930` | ReadBodyVia_008d6d80 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f79a0` | ReadBodyVia_008d6f10 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7a10` | ReadBodyVia_008d6fd0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7a80` | ReadBodyVia_008d7090 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7af0` | ReadBodyVia_008d7120 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7b60` | ReadBodyVia_008d71b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7bd0` | ReadBodyVia_008d7230 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7c40` | ReadBodyVia_008d72a0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7cb0` | ReadBodyVia_008d7340 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7d20` | ReadBodyVia_008d73b0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7d90` | ReadBodyVia_008d7550 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7e00` | ReadBodyVia_008d75d0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7e70` | ReadBodyVia_008d7650 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7ee0` | ReadBodyVia_008d76c0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7f50` | ReadBodyVia_008d7730 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f7fc0` | ReadBodyVia_008d77a0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8030` | ReadBodyVia_008d7810 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f80a0` | ReadBodyVia_008d7880 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8110` | ReadBodyPairVia_008e3710 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8180` | ReadBodyVia_008d7980 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f81f0` | ReadHeaderBlobVia_008d6830 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8270` | ReadBodyVia_008d7a60 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f82e0` | ReadBodyVia_008d7ad0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8350` | ReadBodyVia_008d7bc0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f83c0` | ReadBodyPairVia_008e3cf0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8430` | ReadBodyVia_008d7c70 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f84a0` | ReadBodyVia_008d7d00 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8510` | ReadBodyVia_008d7d70 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8580` | ReadBodyPairVia_008e3d60 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f85f0` | ReadBodyVia_008d7e40 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8660` | ReadBodyVia_008d7f00 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f86d0` | ReadBodyVia_008d7f80 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f8740` | ReadBodyVia_008d80d0 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `008f87b0` | ReadBodyVia_008d8140 | likely | cdecl(destination, start, length, allowTrailing), plain ret: returns 0 when destination or start is null; otherwise builds the bounded reader {start, length, c… |
| `0091af50` | ReadI16x2_I32x2_List_I32 🌐 | likely | Reads a record from the binary reader at param_2 (cursor at +8, end at +0xc, error flag at +0x10) into param_1: two i16 into +4/+8 via FUN_008d6830, two i32 in… |
| `0091aff0` | ReadI16x2_I32x2_Bool_List 🌐 | likely | Reads a record from the reader at param_2 into param_1: two i16 into +4/+8 via FUN_008d6830, two i32 into +0x10/+0x14, one bool byte into +0x18, then a count-p… |
| `00934a20` | BuildPlayerPositionUpdate 🌐 | likely | Checks elapsed-time and position thresholds, builds a normal or on-platform position packet, applies the epsilon comparison, sends changed data, then updates c… |

### `net.packet.point_of_interest`

**PacketPointOfInterestDefinitionReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b97ad0` | PacketPointOfInterestDefinitionReply::PacketPointOfInterest… 🌐 | confirmed | Constructor: stores BasePacket::vftable, writes the opcode 0x39 (57) into the packet id slot at +4, stores PacketPointOfInterestDefinitionReply::vftable and ze… |
| `00b97f50` | PacketPointOfInterestDefinitionReply::ScalarDeletingDestruc… | confirmed | MSVC scalar deleting destructor: stores BasePacket::vftable at this+0 inside an SEH frame and calls _free(this) when bit 0 of param_2 is set. |

**PacketPointOfInterestDefinitionRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b97a20` | PacketPointOfInterestDefinitionRequest::PacketPointOfIntere… 🌐 | confirmed | Constructor: stores BasePacket::vftable, writes the opcode 0x38 (56) into the packet id slot at +4 and then stores PacketPointOfInterestDefinitionRequest::vfta… |
| `00b97ef0` | PacketPointOfInterestDefinitionRequest::ScalarDeletingDestr… | confirmed | MSVC scalar deleting destructor: stores BasePacket::vftable at this+0 inside an SEH frame and calls _free(this) when bit 0 of param_2 is set; returns this. |

**PacketZoneTeleportRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b97b80` | PacketZoneTeleportRequest::PacketZoneTeleportRequest 🌐 | confirmed | Constructor: stores BasePacket::vftable, writes the opcode 0x5a (90) into the packet id slot at +4, stores PacketZoneTeleportRequest::vftable and stores the si… |
| `00b97fb0` | PacketZoneTeleportRequest::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor: stores BasePacket::vftable at this+0 inside an SEH frame and calls _free(this) when bit 0 of param_2 is set. |
| `00b98ad0` | PacketZoneTeleportRequest_Serialize 🌐 | likely | Serializes the op-90 teleport request into a writer: writes the low 16 bits of the packet id at this+4 as a u16 and the dword at this+8 as four bytes, each wri… |

### `net.packet.promotions`

**PromotionalBundleDataPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a05a50` | PromotionalBundleDataPacket::PromotionalBundleDataPacket | confirmed | Constructor: installs PromotionalBundleDataPacket::vftable at offset 0, calls FUN_00a057e0(2) which runs the BasePacket + BasePromotionalPacket base constructo… |
| `00a05ab0` | PromotionalBundleDataPacket::~PromotionalBundleDataPacket | confirmed | Destructor body: rewrites the object vtable to PromotionalBundleDataPacket::vftable, then to BasePromotionalPacket::vftable, then to BasePacket::vftable; no me… |
| `00a05b10` | PromotionalBundleDataPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the vtable-reset destructor chain (PromotionalBundleDataPacket, BasePromotionalPacket, BasePacket) and then calls _free(this) … |

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a08d70` | ParseKeyCodeRedemptionNotification | confirmed | Parses the subtype-1 promotional payload: constructs the notification packet with 00a05910 on this+8, wraps the (ptr,len) buffer in a stream, reads the leading… |
| `00a07850` | Stream_Read2Shorts | likely | Reads two 16-bit values from the stream object at param_2 (cursor +8, end +0xc, error flag +0x10) into param_1+4 and param_1+8, sign-extending them, and zeroes… |
| `00a079e0` | Stream_Read3IntsPlusBlob | likely | Reads three 32-bit values into param_1[0..2] and then a length-prefixed byte blob: reads a count dword, bounds-checks it against the stream end and assigns the… |
| `00a07ab0` | Element50_Construct | likely | Constructs one 0x50-byte array element from three dwords and a string source: stores the three dwords and constructs the trailing StringFixed<48> at +0xc with … |
| `00a07b20` | Element50_InitDefault | likely | Default-initialises one 0x50-byte array element: stores 0, -1, -1 in the first three dwords and default-constructs the StringFixed<48> member at +0xc through 0… |
| `00a07dc0` | Element50_CopyConstruct | likely | Copy-constructs one 0x50-byte array element: copies the three leading dwords and constructs the string member at +0xc from the source's member with 00a07790. |
| `00a07eb0` | Stream_ReadArrayOf20 | likely | Reads a counted array of 0x14-byte elements from the stream at param_1 into the array object at param_2: reads the count, resizes the array up (00a07210) or do… |
| `00a081b0` | Array50_GrowAndCopy | likely | Grows the 0x50-byte element array: allocates a new buffer through vtable slot 1, copy-constructs each old element with 00a07dc0, destroys the old string member… |
| `00a08380` | Array50_Resize | likely | Resizes the 0x50-byte element array: grows the buffer with 00a081b0 when needed, stores the new count at +8 and default-constructs each new element with 00a07b… |
| `00a08420` | Array50_Append | likely | Appends one element to the 0x50-byte record array: ensures capacity with 00a081b0, increments the count at +8 and constructs the new element from the four argu… |
| `00a08940` | Stream_ReadArrayOf50 | likely | Reads a counted array of 0x50-byte elements from the stream into the array object: reads the count, resizes up with 00a08380 or down with 00a08120, then reads … |
| `00a08aa0` | Stream_ReadTwoItemArrays | likely | Reads a length-prefixed blob into an SoeUtil string with FUN_00894b10 (setting the stream error flag on a bad length) and then reads two arrays into the same r… |

### `net.packet.quest`

**BaseQuestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86340` | BaseQuestPacketCtor 🌐 | confirmed | Base constructor of the quest packet family: stores BasePacket's vtable, writes the family class id 0x31 (49) into +4, stores the BaseQuestPacket vtable and wr… |
| `00a87ba0` | BaseQuestPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of BaseQuestPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is se… |
| `00a863b0` | BaseQuestPacketDtorBody | likely | Minimal destructor body used by the quest packet family: rewrites the object's vtable to ClientGateway::BasePacket and returns; 49 EH unwind funclets and one q… |

**ProgressiveQuestRequestClaimPrizePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c14670` | ProgressiveQuestRequestClaimPrizePacket::ProgressiveQuestRe… 🌐 | confirmed | Constructor taking two payload words: calls the progressive-quest base constructor FUN_00bdc8a0 with sub-opcode 4, stores ProgressiveQuestRequestClaimPrizePack… |
| `00c14620` | ProgressiveQuestRequestClaimPrizePacket::~ProgressiveQuestR… | likely | Destructor body that stores only BasePacket::vftable at *this and returns. |

**ProgressiveQuestRequestClaimSlotPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c14560` | ProgressiveQuestRequestClaimSlotPacket::~ProgressiveQuestRe… | likely | Destructor body that stores only BasePacket::vftable at *this and returns. |

**ProgressiveQuestRequestStartQuestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c144f0` | ProgressiveQuestRequestStartQuestPacket::ProgressiveQuestRe… 🌐 | confirmed | Constructor taking two payload words: calls the progressive-quest base constructor FUN_00bdc8a0 with sub-opcode 2, stores ProgressiveQuestRequestStartQuestPack… |
| `00c144a0` | ProgressiveQuestRequestStartQuestPacket::~ProgressiveQuestR… | likely | Destructor body that stores only BasePacket::vftable at *this and returns (the packet's two payload words at +0xc/+0x10 are scalars). |

**QuestEndReplyPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a864c0` | QuestEndReplyPacketCtor 🌐 | confirmed | Constructor of QuestEndReplyPacket: runs the BaseQuestPacket constructor with sub-opcode 0xe (family class id 49), stores the QuestEndReplyPacket vtable and sa… |
| `00a87c60` | QuestEndReplyPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of QuestEndReplyPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 i… |

**QuestReplyPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87c00` | QuestReplyPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of QuestReplyPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is s… |
| `00a86400` | QuestReplyPacketCtor 🌐 | likely | Constructor of QuestReplyPacket: runs the BaseQuestPacket constructor with sub-opcode 2 (family class id 49), stores the QuestReplyPacket vtable, saves param_2… |

**QuestStartBreadcrumbPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87cc0` | QuestStartBreadcrumbPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of QuestStartBreadcrumbPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of pa… |

### `net.packet.quickchat`

**BaseQuickChatPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b8ad10` | BaseQuickChatPacket::CtorWithSubtype 🌐 | confirmed | Installs BasePacket and BaseQuickChatPacket vtables, stores opcode 0x43, and copies the supplied subtype into field 2. |
| `00b8ad80` | BaseQuickChatPacket::Dtor | confirmed | Reinstalls the BaseQuickChatPacket vtable and then the BasePacket vtable at object offset zero. |
| `00b8add0` | BaseQuickChatPacket::ScalarDeletingDtor | confirmed | Restores quick-chat and base packet vtables, then frees the object when flag bit zero is set. |
| `00b8aeb0` | BaseQuickChatPacket::DtorBase | confirmed | Reinstalls BaseQuickChatPacket and BasePacket vtables at the object's first word. |
| `00b93ad0` | BaseQuickChatPacket::ReadBaseFields 🌐 | confirmed | Reads two 16-bit header values, then reads one dword and two more dwords into offsets 0x0c, 0x10, and 0x14; exhausted reads zero fields and set the cursor erro… |
| `00b966e0` | BaseQuickChatPacket_Unpack 🌐 | confirmed | Handler for a received quick-chat packet: overlays BaseQuickChatPacket on the stack object (00b8acb0), reads the two header words from the wire buffer (00b9064… |

**QuickChatSendChatPacketBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b90880` | QuickChatSendChatPacketBase::Ctor | confirmed | Calls the base quick-chat packet constructor, installs QuickChatSendChatPacketBase::vftable, initializes embedded fixed strings, and returns this. |
| `00b908e0` | QuickChatSendChatPacketBase::CtorWithSubtype | confirmed | Calls the base quick-chat constructor with the supplied subtype, installs QuickChatSendChatPacketBase::vftable, initializes embedded fixed strings, and returns… |
| `00b90950` | QuickChatSendChatPacketBase::DeletingDtor | confirmed | Calls quick-chat base cleanup, frees this only when flag bit 0 is set, and returns this otherwise. |

**QuickChatSendChatToChannelPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b90a10` | QuickChatSendChatToChannelPacket::Ctor | confirmed | Calls the common constructor with subtype 3, installs QuickChatSendChatToChannelPacket::vftable, and zeros four subtype fields. |
| `00b90a80` | QuickChatSendChatToChannelPacket::DeletingDtor | confirmed | Calls quick-chat base cleanup, frees this only when flag bit 0 is set, and returns this otherwise. |

**QuickChatSendTellPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b90970` | QuickChatSendTellPacket::Ctor | confirmed | Calls the common constructor with subtype 2, installs QuickChatSendTellPacket::vftable, and initializes its embedded string vtable, empty-storage pointer, and … |
| `00b909f0` | QuickChatSendTellPacket::DeletingDtor | confirmed | Calls tell cleanup, frees this only when flag bit 0 is set, and returns this otherwise. |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b90750` | ReadCountedHashRecords 🌐 | likely | Reads a record count and each record key from the bounded cursor, allocates a 0x3c node, fills it through the reader helper, then links it at the list tail and… |
| `00b96450` | QuickChatPacketBase_ReadChatPayload | likely | Builds a wire reader over (param_2, param_3), reads the two header words with FUN_00b90640, reads a name block into the object at param_1+0xc through FUN_00b90… |
| `00b96f80` | QuickChatPacket_SerializeChatFields | likely | Serializes the chat-shaped quick-chat packet: runs the base packet serializer 00b96d40, then appends the u16 at this+0xa0, the dword at this+0xa4 and the field… |
| `00b97210` | QuickChat_SendTellPacket 🌐 | likely | Sends one packet object through the gateway: allocates an 8KB buffer (FUN_008d4a40), serializes the base fields with 00b96d40 plus the members at +0xb0 (FUN_00… |

### `net.packet.rating`

**BaseRatingPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5ca0` | BaseRatingPacket::BaseRatingPacket | confirmed | Stores BasePacket::vftable, sets the family opcode field to 0xae (174), stores BaseRatingPacket::vftable and writes the constructor argument into object+0x8. |
| `00aa5d10` | BaseRatingPacket::~BaseRatingPacket | confirmed | Stores BaseRatingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5d60` | BaseRatingPacket::ScalarDeletingDtor | confirmed | Stores BaseRatingPacket::vftable and BasePacket::vftable, then calls _free on the object when bit 0 of param_2 is set. |
| `00abab40` | BaseRatingPacket::Dtor | confirmed | Destructor body of a BaseRatingPacket-derived object: calls FUN_00abaad0 on the embedded IString-valued HashListMap, stamps the IString vftable at +0xc, releas… |

**RatingPacketAddCandidate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aadbb0` | RatingPacketAddCandidate::ctor 🌐 | confirmed | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 6 (base opcode 0xae = 174), stamps RatingPacketAddCandidate::vftable at … |
| `00aadc90` | RatingPacketAddCandidate::~RatingPacketAddCandidate | likely | Destructor: releases the seven SoeUtil::IString<char> members at +0xc, +0x1c, +0x2c, +0x3c, +0x4c, +0x5c and +0x6c (each: vtable reset, refcount decrement, rel… |

**RatingPacketAddVote** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aadf80` | RatingPacketAddVote::ctor 🌐 | confirmed | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 8 (base opcode 0xae = 174), stamps RatingPacketAddVote::vftable at +0 an… |
| `00aae020` | RatingPacketAddVote::~RatingPacketAddVote | likely | Destructor: releases the four SoeUtil::IString<char> members at +0xc, +0x1c, +0x2c and +0x3c (vtable reset, refcount decrement, release through vtable slot +8)… |

**RatingPacketCandidateInfoRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aae280` | RatingPacketCandidateInfoRequest::ctor 🌐 | confirmed | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 0x10 (16, base opcode 0xae = 174), stamps RatingPacketCandidateInfoReque… |
| `00aae320` | RatingPacketCandidateInfoRequest::~RatingPacketCandidateInf… | likely | Destructor: releases the two SoeUtil::IString<char> members at +0xc and +0x1c (vtable reset, refcount decrement, release through vtable slot +8) and resets the… |

**RatingPacketRemoveCandidate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aade40` | RatingPacketRemoveCandidate::ctor 🌐 | confirmed | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 7 (base opcode 0xae = 174), stamps RatingPacketRemoveCandidate::vftable … |
| `00aadec0` | RatingPacketRemoveCandidate::~RatingPacketRemoveCandidate | likely | Destructor: releases the two SoeUtil::IString<char> members at +0xc and +0x1c (vtable reset, refcount decrement, release through vtable slot +8) and resets the… |

**RatingPacketRequestFeatured** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aadab0` | RatingPacketRequestFeatured::ctor 🌐 | confirmed | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 0x14 (20, base opcode 0xae = 174), stamps RatingPacketRequestFeatured::v… |
| `00aadb20` | RatingPacketRequestFeatured::~RatingPacketRequestFeatured | likely | Destructor: releases the SoeUtil::IString<char> member at +0xc (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtabl… |

**RatingPacketSearchRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aae140` | RatingPacketSearchRequest::ctor 🌐 | confirmed | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 0xc (12, base opcode 0xae = 174), stamps RatingPacketSearchRequest::vfta… |
| `00aae1c0` | RatingPacketSearchRequest::~RatingPacketSearchRequest | likely | Destructor: releases the two SoeUtil::IString<char> members at +0x18 and +0x28 (vtable reset, refcount decrement, release through vtable slot +8) and resets th… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac62e0` | SendRatingRequestFeatured | likely | Builds and sends RatingPacketRequestFeatured (ctor FUN_00aadab0): resolves the rating-category string from the hashed key 'RatingSystem.HousingId' in the score… |
| `00ac6a00` | SendRatingAddVote | likely | Submits a housing rating vote: gated on the selected row object *(*(this+0x41a0)+0x260) being non-null, it constructs RatingPacketAddVote with FUN_00aadf80, se… |
| `00ac7df0` | SendRatingClientDataRequest | likely | Serialises and enqueues the rating-system packet owned by its argument through the writer FUN_00abf6f0 (that writer emits the family/subtype header via FUN_00a… |
| `00ac9340` | RatingLookupHousingIdAndSend 🌐 | likely | Resolves the interned key "RatingSystem.HousingId" in the client string-hash table using mask 0x7ff, falling back to "Housing". It assigns the resolved value t… |
| `00c857b0` | BuildRatingEntryKeys | likely | Formats each entry's two values into a string, hashes that string, and inserts the hash and string into the provided map. |

### `net.packet.reference_data`

**ClientProfileData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3a2b0` | ClientProfileData_ReadFromReader 🌐 | likely | Reads one record from the bounded reader into the node in ECX: eleven 32-bit fields into +0x00..+0x20 and +0x28/+0x2c, two single-byte booleans into +0x24 and … |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00928d00` | PlayClient::DispatchReferenceData 🌐 | confirmed | Parses a reference-data header, dispatches subtypes 1/2/3 to item-class, item-category, and profile-data handling, and updates related client state. |

**ReferenceDataPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a44f0` | ReferenceDataPacket::ReferenceDataPacket 🌐 | confirmed | Reference-data family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 44 (0x2c) at +4, installs ReferenceDataPacket::vftable an… |
| `008a4550` | ReferenceDataPacket::ReferenceDataPacketSub 🌐 | confirmed | Reference-data family constructor taking the sub-opcode: installs BasePacket::vftable, stores opcode 44 (0x2c) at +4, installs ReferenceDataPacket::vftable and… |
| `008a45c0` | ReferenceDataPacket::~ReferenceDataPacket | confirmed | Destructor body: writes ReferenceDataPacket::vftable then BasePacket::vftable into the object vtable slot and returns; 12 callers including the 00928d00 dispat… |
| `008a4610` | ReferenceDataPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: in an SEH frame restores ReferenceDataPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |

**ReferenceDataPacketClientProfileData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a39bb0` | ReferenceDataPacketClientProfileData::ReferenceDataPacketCl… 🌐 | confirmed | Constructor of the reference-data sub-opcode 3 packet: calls the family constructor FUN_008a4550 with sub-opcode 3, installs ReferenceDataPacketClientProfileDa… |
| `00a39d90` | ReferenceDataPacketClientProfileData::ScalarDeletingDestruc… | confirmed | Scalar deleting destructor: under an SEH frame installs ReferenceDataPacket::vftable then BasePacket::vftable at +0, and calls _free(this) when bit 0 of the fl… |
| `00a3a900` | ReferenceDataPacketClientProfileData_Parse 🌐 | confirmed | Receive path for reference-data sub-opcode 3: constructs a ReferenceDataPacketClientProfileData in its own SEH frame over the supplied table pointer, builds a … |
| `00a39c20` | ReferenceDataPacketClientProfileData::~ReferenceDataPacketC… | likely | Destructor body: writes ReferenceDataPacket::vftable at +0 and then BasePacket::vftable at +0, i.e. it unwinds the packet object back to its base classes; no o… |

**ReferenceDataPacketItemCategoryDefinitions** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4740` | ReferenceDataPacketItemCategoryDefinitions::ReferenceDataPa… 🌐 | confirmed | Item-category-definitions packet constructor taking a payload pointer: calls FUN_008a4550(2) so the packet is opcode 44 with sub-opcode 2, installs ReferenceDa… |
| `008b7d20` | ReferenceDataPacketItemCategoryDefinitions::ScalarDeletingD… | confirmed | Restores ReferenceDataPacket::vftable then BasePacket::vftable, and frees the object when the deleting flag bit is set. |
| `008a47b0` | ReferenceDataPacketItemCategoryDefinitions::~ReferenceDataP… | likely | Destructor body: writes ReferenceDataPacket::vftable then BasePacket::vftable into the object vtable slot and returns; the 00928d00 dispatcher and EH funclet 0… |

**ReferenceDataPacketItemClassDefinitions** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4680` | ReferenceDataPacketItemClassDefinitions::ReferenceDataPacke… 🌐 | confirmed | Item-class-definitions packet constructor taking a payload pointer: calls FUN_008a4550(1) so the packet is opcode 44 with sub-opcode 1, installs ReferenceDataP… |
| `008b7cb0` | ReferenceDataPacketItemClassDefinitions::ScalarDeletingDtor | confirmed | Restores ReferenceDataPacket::vftable then BasePacket::vftable, and frees the object when the deleting flag bit is set. |
| `008a46f0` | ReferenceDataPacketItemClassDefinitions::~ReferenceDataPack… | likely | Destructor body: writes ReferenceDataPacket::vftable then BasePacket::vftable into the object vtable slot and returns; the 00928d00 dispatcher and EH funclet 0… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3a8a0` | ParseClientProfileDataBlob | likely | Parses a payload into a client profile table: builds a bounded reader over (param_1, param_1+param_2), calls 008d8d20 to read two 16-bit header values through … |

### `net.packet.repeating_activity`

**BaseRepeatingActivityPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad9620` | BaseRepeatingActivityPacket::BaseRepeatingActivityPacket | confirmed | Base constructor for the repeating-activity packet family: installs BasePacket::vftable, writes 0x8f (143) into the 2-byte packet id slot at +4, installs BaseR… |
| `00ad9eb0` | BaseRepeatingActivityPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for BaseRepeatingActivityPacket (slot 0 of its vtable): in an SEH frame it resets the vtable to BasePacket::vftable, then calls _fr… |

**RepeatingActivityAddCountPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada280` | RepeatingActivityAddCountPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityAddCountPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the fla… |

**RepeatingActivityAddCountResultPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada2e0` | RepeatingActivityAddCountResultPacket::RepeatingActivityAdd… | confirmed | Runs admin base constructor 00ada120 with activity type 7, installs RepeatingActivityAddCountResultPacket::vftable and stores a success byte at +0x20; the disp… |
| `00ada370` | RepeatingActivityAddCountResultPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityAddCountResultPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when t… |
| `00ad9b70` | RepeatingActivityAddCountResultPacket::~RepeatingActivityAd… | likely | Resets the vtable to BasePacket::vftable (67-byte body); dispatcher 00adbf00 calls it on the result packet it built via 00ada2e0 and one EH unwind funclet also… |
| `00adb890` | RepeatingActivityAddCountResultPacket_Unserialize | likely | Builds a local stream over the buffer/length arguments, parses the admin header plus trailing bool with 00adb5d0 and returns 1 only when no error was flagged a… |

**RepeatingActivityAddPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada9b0` | RepeatingActivityAddPacket::RepeatingActivityAddPacket | confirmed | Runs state-packet constructor 00ad9740, installs RepeatingActivityAddPacket::vftable, then default-constructs two IString members at +0x20 and +0x30 (vtable pl… |
| `00adaa50` | RepeatingActivityAddPacket::~RepeatingActivityAddPacket | confirmed | Releases the two reference-counted IString members at +0x30 and +0x20 (atomic decrement of the refcount in front of the buffer, virtual release when it hits ze… |
| `00adad40` | RepeatingActivityAddPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: calls the non-deleting destructor 00adaa50 and then _free(this) when bit 0 of the flag argument is set; it is vtable slot 0 of Rep… |
| `00adb740` | RepeatingActivityAddPacket_Unserialize | likely | Builds a local stream over the buffer/length arguments, parses the add body with 00adafa0 and returns 1 only when no error was flagged and (unless partial inpu… |

**RepeatingActivityBaseAdminPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada1a0` | RepeatingActivityBaseAdminPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityBaseAdminPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the fl… |

**RepeatingActivityClearRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad9fd0` | RepeatingActivityClearRequest::RepeatingActivityClearRequest 🌐 | confirmed | Runs base constructor 00ad9620 with activity type 4, installs RepeatingActivityClearRequest::vftable and stores three arguments at +0x10, +0x14 and +0x18 (the … |
| `00ada040` | RepeatingActivityClearRequest::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityClearRequest: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag … |
| `00adc160` | RepeatingActivityClearRequest_Serialize 🌐 | likely | Writes the repeating-activity header, copies min(remaining output capacity, 4) bytes from the dword at object +0x18, advances the cursor by the copied length, … |

**RepeatingActivityConsecutiveMsgPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad98d0` | RepeatingActivityConsecutiveMsgPacket::RepeatingActivityCon… | confirmed | Runs base constructor 00ad9620 with activity type 3, installs RepeatingActivityConsecutiveMsgPacket::vftable, stores the activity id at +0xc and zeroes six dwo… |
| `00ad9f70` | RepeatingActivityConsecutiveMsgPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityConsecutiveMsgPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when t… |
| `00ad9940` | RepeatingActivityConsecutiveMsgPacket::~RepeatingActivityCo… | likely | Resets the object's vtable to BasePacket::vftable (67-byte body); dispatcher 00adbf00 calls it on the consecutive-message packet it built and one EH unwind fun… |
| `00adb7b0` | RepeatingActivityConsecutiveMsgPacket_Unserialize | likely | Builds a local stream over the buffer/length arguments, parses the six dwords with 00adb070 and returns 1 only when no error was flagged and (unless partial in… |

**RepeatingActivityCountRemainingInfoPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada610` | RepeatingActivityCountRemainingInfoPacket::ScalarDeletingDt… | confirmed | Virtual deleting destructor for RepeatingActivityCountRemainingInfoPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object wh… |
| `00ad9c60` | RepeatingActivityCountRemainingInfoPacket::~RepeatingActivi… | likely | Resets the vtable to BasePacket::vftable (67-byte body); dispatcher 00adbf00 calls it on the info packet it built via 00ada590 and one EH unwind funclet also c… |
| `00adb820` | RepeatingActivityCountRemainingInfoPacket_Unserialize | likely | Builds a local stream over the buffer/length arguments, parses the admin header (00adb160: id, two dwords, 64-bit Guid) and returns 1 only when no error was fl… |

**RepeatingActivityGetCountRemainingPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada530` | RepeatingActivityGetCountRemainingPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityGetCountRemainingPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object whe… |

**RepeatingActivitySetConsecutivePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad99e0` | RepeatingActivitySetConsecutivePacket::RepeatingActivitySet… 🌐 | confirmed | Runs base constructor 00ad9620 with activity type 5, installs RepeatingActivitySetConsecutivePacket::vftable, then stores the two arguments at +0xc and +0x10. |
| `00ada0a0` | RepeatingActivitySetConsecutivePacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivitySetConsecutivePacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when t… |
| `00adc1d0` | RepeatingActivitySetConsecutivePacket_Serialize 🌐 | likely | Writes the repeating-activity header with 00adbe60, then appends the two 4-byte fields at +0xc and +0x10. |

**RepeatingActivitySetCountPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada450` | RepeatingActivitySetCountPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivitySetCountPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the fla… |

**RepeatingActivityStatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad9f10` | RepeatingActivityStatePacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor for RepeatingActivityStatePacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag a… |
| `00ad97b0` | RepeatingActivityStatePacket::~RepeatingActivityStatePacket | likely | Resets the object's vtable to BasePacket::vftable (67-byte body, no member cleanup); it is the cleanup the dispatcher 00adbf00 runs on the state packet it buil… |
| `00adb6d0` | RepeatingActivityStatePacket_Unserialize | likely | Builds a local stream over the buffer/length arguments, parses the common state fields with 00adaed0 and returns 1 only when no error was flagged and (unless p… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adbf00` | RepeatingActivity_DispatchMessage | confirmed | Constructs a temporary base packet and stream, reads the 16-bit field at +4 and the following message-type byte at +8 with 00adae70, then dispatches valid bodi… |
| `00adc3c0` | RepeatingActivity_SendClearRequest 🌐 | confirmed | Serializes the clear request into an 8 KB stack buffer via 00adc160, wraps the bytes in a ClientGateway::BasePacket and hands it to the gateway send helper FUN… |
| `00adc4d0` | RepeatingActivity_SendSetConsecutive 🌐 | confirmed | Serializes the set-consecutive request into a stack buffer via 00adc1d0, wraps it in a ClientGateway::BasePacket and sends it through FUN_00925640 with the tun… |
| `00adc5e0` | RepeatingActivity_SendAddCount 🌐 | confirmed | Serializes the addcount request into a stack buffer via 00adc270, wraps it in a ClientGateway::BasePacket and sends it through FUN_00925640. |
| `00adc6f0` | RepeatingActivity_SendSetCount 🌐 | confirmed | Serializes the setcount request into a stack buffer via 00adc270, wraps it in a ClientGateway::BasePacket and sends it through FUN_00925640. |
| `00adc800` | RepeatingActivity_SendGetCountRemaining 🌐 | confirmed | Serializes the count-remaining query into a stack buffer via 00adc270, wraps it in a ClientGateway::BasePacket and sends it through FUN_00925640. |
| `00adbe60` | RepeatingActivity_WriteHeader 🌐 | likely | Appends the packet header to the output buffer: copies two bytes of the packet's opcode field at +4 and one byte of the type field at +8 through the bounds-che… |

### `net.packet.report`

**BaseReportPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5650` | BaseReportPacket::BaseReportPacket 🌐 | confirmed | Report family constructor: installs BasePacket::vftable, stores opcode 68 (0x44) at +4, installs BaseReportPacket::vftable and stores the sub-opcode argument a… |
| `008a56c0` | BaseReportPacket::~BaseReportPacket | confirmed | Destructor body: restores BaseReportPacket::vftable then BasePacket::vftable; four EH funclets (01584c10, 01586130, 015c3c90, 015c3cd0) call it. |
| `008a5710` | BaseReportPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: in an SEH frame restores BaseReportPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
| `008d1e10` | BaseReportPacket::~BaseReportPacket | confirmed | Destructor body of the report-packet base: first runs the CRT vector-destructor iterator over the ten inline 0x30-byte records starting at +0x20 (element destr… |

**ReportPacketReportHouse** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aae4f0` | ReportPacketReportHouse::ctor 🌐 | confirmed | Constructor: runs the BaseReportPacket base constructor FUN_008a5650 with inner opcode 3 (base opcode 0x44 = 68), stamps ReportPacketReportHouse::vftable at +0… |
| `00ab0a40` | ReportPacketReportHouseScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00aae580` | ReportPacketReportHouse::~ReportPacketReportHouse | likely | Destructor: releases the two SoeUtil::IString<char> members at +0x20 and +0x30 (vtable reset, refcount decrement, release through vtable slot +8) and resets th… |
| `00ac19a0` | SendReportPacketReportHouse | likely | Transport wrapper for the house-report packet: opens the scratch scope with FUN_008d4a40(0x10000000), runs the family header writer FUN_0090b190, writes the 64… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d1cf0` | Ctor_TwoStringsAt8And18 | likely | Initializer of a 0x30-byte record: installs two embedded SoeUtil::IString<char> members, one at +8 (buffer +0xc, length +0x10, capacity +0x14) and one at +0x18… |
| `008d1d60` | Dtor_TwoStringsAt8And18 | likely | Destructor of the record initialized by 008d1cf0: releases the embedded SoeUtil::IString<char> at +0x18 (buffer +0x1c, capacity +0x24) and then the one at +8 (… |
| `00ac71a0` | SendReportHouse | likely | Report sender for the selected row: takes the row object at *(*(this+0x41a0)+0x260) and, when non-null, constructs ReportPacketReportHouse with FUN_00aae4f0, c… |

### `net.packet.reward`

**RewardBasePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b88ad0` | RewardBasePacket_Ctor 🌐 | confirmed | Constructor of the reward packet base with a fixed zero subtype: installs BasePacket::vftable, stores the base opcode 0x32 at +4, installs RewardBasePacket::vf… |
| `00b88b40` | RewardBasePacket_CtorWithSubtype 🌐 | confirmed | Reward packet base constructor taking the subtype: installs BasePacket::vftable, stores base opcode 0x32 at +4, installs RewardBasePacket::vftable, stores para… |
| `00b88ff0` | RewardBasePacket::ScalarDeletingDtor | confirmed | Vtable slot 0 of RewardBasePacket: runs the base destructor 00b88bc0 and frees the object through the _free thunk when bit 0 of param_2 is set. |
| `00b89190` | RewardBasePacket_ReadHeader 🌐 | confirmed | Base body reader of the reward family: over the bounded reader object (cursor this+8, end this+0xc, error byte this+0x10) it reads a little-endian 16-bit value… |

**RewardBundlePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b88c40` | RewardBundlePacket::RewardBundlePacket 🌐 | confirmed | Constructor of the subtype-1 reward bundle packet: runs the base constructor with subtype 1 (00b88b40), installs RewardBundlePacket::vftable, stores the bundle… |
| `00b88cb0` | RewardBundlePacket_SetByte20 | confirmed | Ten-byte setter at vtable slot 1 of RewardBundlePacket: writes the byte argument to this+0x20 and returns. |
| `00b89010` | RewardBundlePacket::ScalarDeletingDtor | confirmed | Vtable slot 0 of RewardBundlePacket: runs the shared base destructor 00b88bc0 under an SEH frame and frees the packet through _free when bit 0 of param_2 is se… |
| `00b8a300` | RewardBundlePacket_Read 🌐 | likely | Entry point that reads a subtype-1 bundle packet: constructs the bounded reader over (param_2, param_2+param_3), calls the base header reader 00b89190, passes … |

**RewardNonBundledItem** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b88d10` | RewardNonBundledItem::RewardNonBundledItem 🌐 | confirmed | Constructor of the subtype-2 non-bundled item packet: runs the base constructor with subtype 2 (00b88b40), installs RewardNonBundledItem::vftable, stores the a… |
| `00b89070` | RewardNonBundledItem::ScalarDeletingDtor | confirmed | Vtable slot 0 of RewardNonBundledItem: runs the class destructor body 00b88db0 and frees the packet through _free when bit 0 of param_2 is set. |
| `00b88db0` | RewardNonBundledItem::~RewardNonBundledItem | likely | Destructor body of the non-bundled item packet: reinstalls Loggable::vftable at this+0x24 (the secondary base subobject), then destroys the reward packet base … |
| `00b891f0` | RewardNonBundledItem_ReadBody 🌐 | likely | Body reader for the subtype-2 item packet: calls the base header reader 00b89190, then reads four 32-bit words into the fields at +0x1c, +0x20 (plus an embedde… |
| `00b89590` | RewardNonBundledItem_Read 🌐 | likely | Entry point that reads a subtype-2 item packet body from a raw buffer: sets up the bounded reader (cursor = param_2, end = param_2+param_3, error byte local), … |

**RewardPacketNonMemberRewardDeferred** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b88e10` | RewardPacketNonMemberRewardDeferred::RewardPacketNonMemberR… 🌐 | confirmed | Constructor of the subtype-6 deferred non-member reward packet: runs the base constructor with subtype 6 (00b88b40), installs RewardPacketNonMemberRewardDeferr… |
| `00b89090` | RewardPacketNonMemberRewardDeferred::ScalarDeletingDtor | confirmed | Vtable slot 0 of the subtype-6 deferred packet: runs the base destructor 00b88bc0 under an SEH frame and frees the packet through _free when bit 0 of param_2 i… |
| `00b892b0` | RewardPacketNonMemberRewardDeferred_ReadBody 🌐 | confirmed | Body reader for the subtype-6 deferred packet: calls the base header reader 00b89190 and then reads three 32-bit words into the fields at +0x1c, +0x20 and +0x2… |
| `00b89600` | RewardPacketNonMemberRewardDeferred_Read 🌐 | likely | Entry point that reads a subtype-6 deferred packet body from a raw buffer: constructs the bounded reader over (param_2, param_2+param_3), calls the body reader… |

**RewardPacketNonMemberRewardGranted** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b893a0` | RewardPacketNonMemberRewardGranted::RewardPacketNonMemberRe… 🌐 | confirmed | Constructor of the subtype-7 granted-bank packet: runs the base constructor with subtype 7 (00b88b40), installs RewardPacketNonMemberRewardGranted::vftable, ze… |
| `00b894a0` | RewardPacketNonMemberRewardGranted_ReadBody 🌐 | confirmed | Body reader for the subtype-7 granted packet: calls the base header reader 00b89190, reads one 32-bit value into +0x1c (granted coins) with bounds handling, th… |
| `00b89500` | RewardPacketNonMemberRewardGranted::ScalarDeletingDtor | confirmed | Vtable slot 0 of the granted-bank packet: runs the destructor body 00b89410 and frees the packet through _free when bit 0 of param_2 is set. |
| `00b89410` | RewardPacketNonMemberRewardGranted_Destroy | likely | Destructor body of the granted-bank packet: reinstalls the list vtable at +0x20, clears the embedded deferred-stars list through 00b88f80 and then destroys the… |
| `00b89c00` | RewardPacketNonMemberRewardGranted_Read 🌐 | likely | Entry point that reads a subtype-7 granted packet body from a raw buffer: constructs the bounded reader over (param_2, param_2+param_3), calls the body reader … |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b88a70` | RewardDeferredStars_ReadRow 🌐 | confirmed | Row reader over a bounded packet reader object (cursor this+8, end this+0xc, error byte this+0x10): reads two consecutive 32-bit words into param_1[0] and para… |
| `00b89340` | RewardDeferredStars_ReadList 🌐 | confirmed | List reader of the subtype-7 body: clears the list (00b88f80), then reads a 32-bit count from the bounded reader and, while the count remains and no error is s… |
| `00b88e70` | RewardNonMemberDeferred_DestroyBase | likely | Cleanup helper that destroys the reward packet base of the caller's temporary through 00b88bc0 under an SEH frame. |

### `net.packet.shipcombat`

**BaseShipCombatPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4ec00` | BaseShipCombatPacket::BaseShipCombatPacket_Sub | confirmed | Shared base constructor of every concrete ship-combat packet: stores BasePacket::vftable, writes opcode 0x5e (94) into field +4, stores BaseShipCombatPacket::v… |
| `00a4ec70` | BaseShipCombatPacket::~BaseShipCombatPacket | confirmed | Destructor body of the ship-combat packet base: resets the object's vtable to BaseShipCombatPacket then to BasePacket; no members are destroyed. Called from 11… |
| `00a4ecc0` | BaseShipCombatPacket::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor occupying vtable slot 0: resets the vtable to BaseShipCombatPacket then BasePacket and frees the packet through the _free thunk… |
| `00a57620` | BaseShipCombatPacket_DtorWithFixed64 | likely | Destroys two embedded fixed-64 string members under SEH, then resets the object through BaseShipCombatPacket and BasePacket vtables. |
| `00a577a0` | BaseShipCombatPacket_Dtor | likely | Runs FUN_00703350 for a base member inside an SEH frame, then restores BaseShipCombatPacket and BasePacket vtables. |
| `00a57810` | BaseShipCombatPacket_Dtor | likely | Destroys an embedded fixed string through FUN_00a55550, then resets the object to BaseShipCombatPacket and BasePacket vtables. |

**ShipCombatPacketAIDebugData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a527e0` | ShipCombatPacketAIDebugData::ShipCombatPacketAIDebugData | confirmed | Constructor: base-initialises with sub-opcode 0x24 (36) and stores ShipCombatPacketAIDebugData::vftable. |
| `00a52860` | ShipCombatPacketAIDebugData::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f4c0` | ShipCombatPacketAIDebugData::~ShipCombatPacketAIDebugData | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a527e0 (sub-opcode 0x24) through caller 00a659… |

**ShipCombatPacketAnnouncerMessage** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a526f0` | ShipCombatPacketAnnouncerMessage::ShipCombatPacketAnnouncer… | confirmed | Constructor: base-initialises with sub-opcode 0x22 (34) and stores ShipCombatPacketAnnouncerMessage::vftable. |
| `00a52770` | ShipCombatPacketAnnouncerMessage::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f460` | ShipCombatPacketAnnouncerMessage::~ShipCombatPacketAnnounce… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a526f0 (sub-opcode 0x22) through caller 00a622… |

**ShipCombatPacketCreateProxiedShip** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a678f0` | ShipCombatPacketCreateProxiedShip::ShipCombatPacketCreatePr… 🌐 | confirmed | Constructor: runs the shared ship-combat base constructor FUN_00a4ec00(9) (which writes BasePacket::vftable, the family opcode 0x5e at +4, BaseShipCombatPacket… |
| `00a679a0` | ShipCombatPacketCreateProxiedShip::~ShipCombatPacketCreateP… | confirmed | Destructor body: rewrites its own vtable into the object, then destroys the payload members in reverse construction order - FUN_00a55550 twice (the two SoeUtil… |

**ShipCombatPacketDespawnProjectile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a54fe0` | ShipCombatPacketDespawnProjectile::Ctor 🌐 | confirmed | Calls the shared ship-combat packet base constructor with 0x1b, installs the DespawnProjectile vtable, clears two fields, and copies default four-dword state v… |
| `00a55060` | ShipCombatPacketDespawnProjectile::ScalarDeletingDtor | confirmed | Rewrites the object's vtable to BaseShipCombatPacket then BasePacket; if flag bit 0 is set, frees the object, and returns its address otherwise. |

**ShipCombatPacketDestroyProxiedPickup** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a52400` | ShipCombatPacketDestroyProxiedPickup::ShipCombatPacketDestr… | confirmed | Constructor: base-initialises with sub-opcode 0x1d (29), stores ShipCombatPacketDestroyProxiedPickup::vftable, seeds +0xc from the static default at 01be6e80, … |
| `00a52490` | ShipCombatPacketDestroyProxiedPickup::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f370` | ShipCombatPacketDestroyProxiedPickup::~ShipCombatPacketDest… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52400 (sub-opcode 0x1d) through caller 00a5fa… |

**ShipCombatPacketDestroyProxiedShip** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a521b0` | ShipCombatPacketDestroyProxiedShip::ShipCombatPacketDestroy… | confirmed | Constructor: base-initialises with sub-opcode 10 and stores ShipCombatPacketDestroyProxiedShip::vftable. |
| `00a52230` | ShipCombatPacketDestroyProxiedShip::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f180` | ShipCombatPacketDestroyProxiedShip::~ShipCombatPacketDestro… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a521b0 (sub-opcode 10) through caller 00a5f120… |

**ShipCombatPacketDestroyProxiedStructure** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a52520` | ShipCombatPacketDestroyProxiedStructure::ShipCombatPacketDe… | confirmed | Constructor: base-initialises with sub-opcode 0x20 (32), stores ShipCombatPacketDestroyProxiedStructure::vftable and seeds the field +0xc from the static defau… |
| `00a52590` | ShipCombatPacketDestroyProxiedStructure::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f3c0` | ShipCombatPacketDestroyProxiedStructure::~ShipCombatPacketD… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52520 (sub-opcode 0x20) through caller 00a5fe… |

**ShipCombatPacketMarkTreasurePickups** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a57880` | ShipCombatPacketMarkTreasurePickups::Ctor 🌐 | confirmed | Calls the base constructor with opcode 0x1e, installs the MarkTreasurePickups vtable, and initializes a Vector2 array member with empty storage. |

**ShipCombatPacketPlayerAwards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a576b0` | ShipCombatPacketPlayerAwards::Ctor 🌐 | confirmed | Calls the common packet constructor with opcode 0x18, installs PlayerAwards packet vtable, initializes an embedded ShipCombatAwardData array and zeros its stor… |

**ShipCombatPacketRefreshUI** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4eeb0` | ShipCombatPacketRefreshUI::ShipCombatPacketRefreshUI | confirmed | Constructor: base-initialises with sub-opcode 4 and stores ShipCombatPacketRefreshUI::vftable. |
| `00a51e30` | ShipCombatPacketRefreshUI::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4ef20` | ShipCombatPacketRefreshUI::~ShipCombatPacketRefreshUI | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4eeb0 through caller 00a655a0. |

**ShipCombatPacketRegisterShipPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4ef70` | ShipCombatPacketRegisterShipPlayer::ShipCombatPacketRegiste… | confirmed | Constructor: base-initialises with sub-opcode 5 and stores ShipCombatPacketRegisterShipPlayer::vftable. |
| `00a51ea0` | ShipCombatPacketRegisterShipPlayer::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4efd0` | ShipCombatPacketRegisterShipPlayer::~ShipCombatPacketRegist… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4ef70 through caller 00a69380. |

**ShipCombatPacketRemoveShipPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a51f10` | ShipCombatPacketRemoveShipPlayer::ShipCombatPacketRemoveShi… | confirmed | Constructor: base-initialises with sub-opcode 6, stores ShipCombatPacketRemoveShipPlayer::vftable and stores the two caller arguments into the fields +0x10 and… |
| `00a51fa0` | ShipCombatPacketRemoveShipPlayer::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f020` | ShipCombatPacketRemoveShipPlayer::~ShipCombatPacketRemoveSh… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired through shared caller 00a6cbd0 with the constructor 00a51f10 (sub-opc… |

**ShipCombatPacketSetPlayerCount** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4f070` | ShipCombatPacketSetPlayerCount::ShipCombatPacketSetPlayerCo… | confirmed | Constructor: base-initialises with sub-opcode 7, stores ShipCombatPacketSetPlayerCount::vftable and zeroes the fields +0xc and +0x10 (the player-count message … |
| `00a52010` | ShipCombatPacketSetPlayerCount::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f0e0` | ShipCombatPacketSetPlayerCount::~ShipCombatPacketSetPlayerC… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4f070 through caller 00a5f030. |

**ShipCombatPacketUpdateGameState** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4ed30` | ShipCombatPacketUpdateGameState::ShipCombatPacketUpdateGame… | confirmed | Constructor: base-initialises with sub-opcode 2, stores ShipCombatPacketUpdateGameState::vftable and zeroes the trailing field +0xc. |
| `00a51d50` | ShipCombatPacketUpdateGameState::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and frees the packet through the _free thunk when th… |
| `00a4edb0` | ShipCombatPacketUpdateGameState::~ShipCombatPacketUpdateGam… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor at 00a4ed30 through the shared caller 00a5ed80 a… |

**ShipCombatPacketUpdatePlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a52080` | ShipCombatPacketUpdatePlayer::ShipCombatPacketUpdatePlayer | confirmed | Constructor: base-initialises with sub-opcode 8, stores ShipCombatPacketUpdatePlayer::vftable and initialises the two fields +0x10/+0x14 from the static data a… |
| `00a52110` | ShipCombatPacketUpdatePlayer::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f130` | ShipCombatPacketUpdatePlayer::~ShipCombatPacketUpdatePlayer | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52080 (sub-opcode 8) through caller 00a67110. |

**ShipCombatPacketUpdatePlayerGameStates** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4ee00` | ShipCombatPacketUpdatePlayerGameStates::ShipCombatPacketUpd… | confirmed | Constructor: base-initialises with sub-opcode 3 and stores ShipCombatPacketUpdatePlayerGameStates::vftable. |
| `00a51dc0` | ShipCombatPacketUpdatePlayerGameStates::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4ee60` | ShipCombatPacketUpdatePlayerGameStates::~ShipCombatPacketUp… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4ee00 through caller 00a69480. |

**ShipCombatPacketUpdateProxiedStructure** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a52600` | ShipCombatPacketUpdateProxiedStructure::ShipCombatPacketUpd… | confirmed | Constructor: base-initialises with sub-opcode 0x21 (33), stores ShipCombatPacketUpdateProxiedStructure::vftable, seeds +0xc from the static default at 01be6e18… |
| `00a52680` | ShipCombatPacketUpdateProxiedStructure::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f410` | ShipCombatPacketUpdateProxiedStructure::~ShipCombatPacketUp… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52600 (sub-opcode 0x21) through caller 00a5ff… |

**ShipCombatPacketUpdateShipCannons** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a522c0` | ShipCombatPacketUpdateShipCannons::ShipCombatPacketUpdateSh… | confirmed | Constructor: base-initialises with sub-opcode 0x13 (19), stores ShipCombatPacketUpdateShipCannons::vftable, initialises +0x10/+0x14 from static defaults and ze… |
| `00a52360` | ShipCombatPacketUpdateShipCannons::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
| `00a4f2a0` | ShipCombatPacketUpdateShipCannons::~ShipCombatPacketUpdateS… | likely | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a522c0 (sub-opcode 0x13) through caller 00a5ee… |

**ShipCombatPacketZoneConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4f1d0` | ShipCombatPacketZoneConfig::ShipCombatPacketZoneConfig | confirmed | Constructor: base-initialises with sub-opcode 0xc (12), stores ShipCombatPacketZoneConfig::vftable, then runs the zone-configuration helper FUN_01067ac0 on the… |
| `00a522a0` | ShipCombatPacketZoneConfig::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): runs the ZoneConfig destructor body 00a4f240 (which tears down the config member and resets the base vtables) and f… |
| `00a4f240` | ShipCombatPacketZoneConfig::~ShipCombatPacketZoneConfig | likely | Destructor body: runs the zone-config member teardown FUN_01067f20 and then resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the c… |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6d270` | ShipCombat_DispatchInbound | confirmed | Inbound ship-combat dispatcher. Returns 0 when the gate byte at this+0x10 is clear. Otherwise it reads the sub-opcode from the bounded payload with 00a5a0a0 an… |
| `00a4f320` | ShipCombatPacketDtor_00a4f320 | likely | Destructor body of a ship combat packet class: resets the vtable to BaseShipCombatPacket then BasePacket. Nothing in this shard constructs the class; it is des… |
| `00a67ff0` | GatewaySend_ShipCombatHeaderAndU32 🌐 | likely | Same tunneled-send skeleton as 00a67ee0 but with the richer serialiser FUN_00a65140, which after the inner packet header (fields +4 and +8) also writes the u32… |
| `00a68100` | GatewaySend_AdminCommandShipCombat 🌐 | likely | Tunneled send for the admin-command packet passed as param_1: serialises it with FUN_00a651a0 - which first writes the command payload through FUN_0090ac70, th… |
| `00a6ccdf` | ShipCombat_ApplyRemoveShipPlayer | likely | Apply step of the RemoveShipPlayer message without parsing, with the object taken from ESI: looks up the entry with 00a55390, calls 00a55410, and when an entry… |
| `00a6cdf0` | ShipCombat_HandleZoneConfig | likely | Inbound handler for sub-opcode 12: constructs a temporary ShipCombatPacketZoneConfig (00a4f1d0), reads the bounded record with 00a5a0a0 and 00a69cf0, and when … |

### `net.packet.soccer`

**BaseSoccerPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b73110` | BaseSoccerPacket::~BaseSoccerPacket | confirmed | Base-object destructor body of the family-76 packet hierarchy: stores BaseSoccerPacket::vftable, marks the object dead for EH bookkeeping (local_4 = 0xffffffff… |
| `00b73160` | BaseSoccerPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the object's vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, calls _f… |
| `00b73950` | BaseSoccerPacket::~BaseSoccerPacket | confirmed | Stores BaseSoccerPacket::vftable and then BasePacket::vftable into the object. |
| `00b77fc0` | BaseSoccerPacket::ReadInnerHeader | confirmed | Reads two 16-bit words from the byte reader (cursor at reader+8, end at reader+0xc). The first word is sign-extended (movsx) into this+4 and the second into th… |

**SoccerPacketAcquireBall** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75010` | SoccerPacketAcquireBall::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketAcquirePickUp** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74ee0` | SoccerPacketAcquirePickUp::Ctor 🌐 | confirmed | Constructor of the family-76 sub 0xa (10) packet SoccerPacketAcquirePickUp: calls the shared base ctor FUN_00b730a0(10) and installs SoccerPacketAcquirePickUp:… |
| `00b74f40` | SoccerPacketAcquirePickUp::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketBallBounce** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b73840` | SoccerPacketBallBounce::Ctor 🌐 | confirmed | Constructor of the family-76 sub 0x19 (25) packet SoccerPacketBallBounce: calls the shared base ctor FUN_00b730a0(0x19) and installs SoccerPacketBallBounce::vf… |
| `00b751c0` | SoccerPacketBallBounce::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketFreeBall** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75080` | SoccerPacketFreeBall::Ctor 🌐 | confirmed | Constructor of the family-76 sub 0xc (12) packet SoccerPacketFreeBall: calls the shared base ctor FUN_00b730a0(0xc) and installs SoccerPacketFreeBall::vftable. |
| `00b750e0` | SoccerPacketFreeBall::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketGoalMade** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b749f0` | SoccerPacketGoalMade::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketHighArcBall** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74da0` | SoccerPacketHighArcBall::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketImpulseBall** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75150` | SoccerPacketImpulseBall::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |
| `00b79580` | SoccerPacketImpulseBall::ReadBody | confirmed | Calls 00b77fc0 to read the inner header, then reads a u32 into this+0xc, a float4 via 008e2410 into this+0x10, u32 values into this+0x20 and +0x24 (each NaN-fl… |
| `00b7b840` | SoccerPacketImpulseBall::ReadBodyRange | confirmed | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b73790` | SoccerPacketImpulseBall::Ctor 🌐 | likely | Constructor of the family-76 sub 0xd (13) packet SoccerPacketImpulseBall: calls the shared base ctor FUN_00b730a0(0xd) (family 0x4C at +4, sub 13 at +8) and in… |

**SoccerPacketPlayerVariableStats** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74c60` | SoccerPacketPlayerVariableStats::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketRegisterPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74b90` | SoccerPacketRegisterPlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |
| `00b74b30` | SoccerPacketRegisterPlayer::Ctor 🌐 | likely | Constructor of the family-76 sub 3 packet SoccerPacketRegisterPlayer: calls the shared base ctor FUN_00b730a0(3) and installs SoccerPacketRegisterPlayer::vftab… |

**SoccerPacketSendAIInfo** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b77ca0` | SoccerPacketSendAIInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the class destructor body FUN_00b76d50 (member cleanup plus vtable resets) and, when the flag byte's low bit is set, frees th… |
| `00b76d50` | SoccerPacketSendAIInfo::~SoccerPacketSendAIInfo | likely | Non-deleting destructor of the sub-0x18 SendAIInfo packet: calls the member cleanup helper FUN_00401fd0, then resets the vtable to BaseSoccerPacket::vftable an… |

**SoccerPacketSetClientConfig** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b77cc0` | SoccerPacketSetClientConfig::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the class destructor body FUN_00b76e40 (releases the embedded SoeUtil::IString refcount then resets base vtables) and, when t… |
| `00b76e40` | SoccerPacketSetClientConfig::~SoccerPacketSetClientConfig | likely | Non-deleting destructor of the sub-7 SetClientConfig packet: resets the embedded SoeUtil::IString vtable at +0x8c, and when its refcount (word before the buffe… |

**SoccerPacketSetPlayerTeam** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b753d0` | SoccerPacketSetPlayerTeam::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketSetSoccerPlayerOneTimeStats** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74e70` | SoccerPacketSetSoccerPlayerOneTimeStats::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketSpawnPickUp** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75fe0` | SoccerPacketSpawnPickUp::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketSpecialAnimTime** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74ac0` | SoccerPacketSpecialAnimTime::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketStatRollInfo** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74640` | SoccerPacketStatRollInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketTeamColours** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b77d50` | SoccerPacketTeamColours::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the class destructor body FUN_00b76ed0 (two member cleanup calls plus vtable resets) and, when the flag byte's low bit is set… |
| `00b76ed0` | SoccerPacketTeamColours::~SoccerPacketTeamColours | likely | Non-deleting destructor of the sub-8 TeamColours packet: runs the two member cleanup helpers FUN_004022b0 (the second one triggered by the EH state), then rese… |

**SoccerPacketUpdateGameState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b752a0` | SoccerPacketUpdateGameState::Ctor 🌐 | confirmed | Constructor of the family-76 sub 5 packet SoccerPacketUpdateGameState: calls the shared base ctor FUN_00b730a0(5) and installs SoccerPacketUpdateGameState::vft… |
| `00b75300` | SoccerPacketUpdateGameState::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketUpdateGameTime** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75230` | SoccerPacketUpdateGameTime::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketUpdatePlayerControls** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74920` | SoccerPacketUpdatePlayerControls::Ctor 🌐 | confirmed | Constructor of the family-76 sub 0x10 (16) packet SoccerPacketUpdatePlayerControls: calls the shared base ctor FUN_00b730a0(0x10) and installs SoccerPacketUpda… |
| `00b74980` | SoccerPacketUpdatePlayerControls::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |
| `00b793c0` | SoccerPacketUpdatePlayerControls::ReadBody | confirmed | Calls 00b77fc0 to read the inner header, then reads a u64 (player guid) into this+0x10 and this+0x14, two u32 values into this+0x18 and this+0x1c (each NaN-fla… |
| `00b7b760` | SoccerPacketUpdatePlayerControls::ReadBodyRange | confirmed | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |

**SoccerPacketUpdatePlayerPos** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74850` | SoccerPacketUpdatePlayerPos::Ctor 🌐 | confirmed | Constructor of the family-76 sub 1 packet SoccerPacketUpdatePlayerPos: calls the shared base ctor FUN_00b730a0(1) and installs SoccerPacketUpdatePlayerPos::vft… |
| `00b748b0` | SoccerPacketUpdatePlayerPos::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketUpdatePlayerState** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b746b0` | SoccerPacketUpdatePlayerState::Ctor 🌐 | confirmed | Constructor of the family-76 sub 0xe (14) packet SoccerPacketUpdatePlayerState: calls the shared base ctor FUN_00b730a0(0xe) and installs SoccerPacketUpdatePla… |
| `00b74710` | SoccerPacketUpdatePlayerState::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |
| `00b794b0` | SoccerPacketUpdatePlayerState::ReadBody | confirmed | Calls 00b77fc0 to read the inner header, then reads a u64 (player guid) into this+0x10 and this+0x14, u8 values into this+0x18 (state) and this+0x19 (aux), a u… |
| `00b7b7d0` | SoccerPacketUpdatePlayerState::ReadBodyRange | confirmed | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |

**SoccerPacketUpdatePlayerStats** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b747e0` | SoccerPacketUpdatePlayerStats::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**SoccerPacketUpdateSoccerBall** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b74cd0` | SoccerPacketUpdateSoccerBall::Ctor 🌐 | confirmed | Constructor of the family-76 sub 4 packet SoccerPacketUpdateSoccerBall: calls the shared base ctor FUN_00b730a0(4) and installs SoccerPacketUpdateSoccerBall::v… |
| `00b74d30` | SoccerPacketUpdateSoccerBall::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object … |

**(no class)** (65)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b731d0` | SoccerPacketDtor_00b731d0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1 (the vtable-reset chain of the family-76 packet hierarc… |
| `00b73220` | SoccerPacketDtor_00b73220 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1; byte-identical to 00b731d0 except for the EH handler i… |
| `00b73270` | SoccerPacketDtor_00b73270 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b732c0` | SoccerPacketDtor_00b732c0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73310` | SoccerPacketDtor_00b73310 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b733c0` | SoccerPacketDtor_00b733c0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73410` | SoccerPacketDtor_00b73410 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73460` | SoccerPacketDtor_00b73460 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b734b0` | SoccerPacketDtor_00b734b0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b735b0` | SoccerPacketDtor_00b735b0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73600` | SoccerPacketDtor_00b73600 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73650` | SoccerPacketDtor_00b73650 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73740` | SoccerPacketDtor_00b73740 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b737f0` | SoccerPacketDtor_00b737f0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b738a0` | SoccerPacketDtor_00b738a0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b739a0` | SoccerPacketDtor_00b739a0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b739f0` | SoccerPacketDtor_00b739f0 | confirmed | Base-object destructor copy: stores BaseSoccerPacket::vftable and then BasePacket::vftable into param_1. |
| `00b73500` | SoccerPacketUpdateSoccerBall::~SoccerPacketUpdateSoccerBall | likely | Runs the shared SoccerPacket/BasePacket destructor body on the temporary for family-76 sub 0x4: stores BaseSoccerPacket::vftable, then BasePacket::vftable. |
| `00b736a0` | SoccerPacketAcquirePickUp::~SoccerPacketAcquirePickUp | likely | Runs the shared SoccerPacket/BasePacket destructor body on the temporary for family-76 sub 0xa: stores BaseSoccerPacket::vftable, then BasePacket::vftable. |
| `00b736f0` | SoccerPacketAcquireBall::~SoccerPacketAcquireBall | likely | Runs the shared SoccerPacket/BasePacket destructor body on the temporary for family-76 sub 0xb: stores BaseSoccerPacket::vftable, then BasePacket::vftable. |
| `00b78020` | ReadBody_00b78020 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, then two single bytes into this+0x18 and this+0x19 as bool values. A sh… |
| `00b780c0` | ReadBody_00b780c0 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, then five u32 values into this+0x18, +0x1c, +0x20, +0x24 and +0x28 with… |
| `00b781c0` | ReadBody_00b781c0 | likely | Calls 00b77fc0 to read the inner header, then reads a u8 into this+0xc, a u32 into this+0x10, a u8 into this+0x14, u32 values into this+0x18, +0x1c, +0x20 and … |
| `00b78340` | ReadBody_00b78340 | likely | Calls 00b77fc0 to read the inner header, then reads a u8 into this+0xc, a u32 into this+0x10 (sets reader flag byte reader+0x11 when NaN), and a u64 into this+… |
| `00b783f0` | ReadBody_00b783f0 | likely | Calls 00b77fc0 to read the inner header, then reads a u8 into this+0xc, a u64 into this+0x10 and this+0x14, and a u64 into this+0x18 and this+0x1c. A short rea… |
| `00b78490` | ReadBody_00b78490 | likely | Calls 00b77fc0 to read the inner header. Then reads two counted byte spans: each takes an int32 length, which must be non-negative and fit the remaining bytes,… |
| `00b78600` | ReadBody_00b78600 | likely | Calls 00b77fc0 to read the inner header, then reads a float4 via 008e2410 into this+0x10; sixteen u32 values into this+0x20 through +0x5c (each flags reader+0x… |
| `00b78b00` | ReadBody_00b78b00 | likely | Calls 00b77fc0 to read the inner header, then reads a float4 via 008e2410 into this+0x10, a u8 into this+0x20, and a u32 into this+0x24 (sets reader flag byte … |
| `00b78b90` | ReadBody_00b78b90 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, then a u32 into this+0x18 (sets reader flag byte reader+0x11 when NaN).… |
| `00b78c10` | ReadBody_00b78c10 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14; fourteen u32 values into this+0x18 through +0x4c (each flags reader+0x1… |
| `00b790f0` | ReadBody_00b790f0 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, u32 values into this+0x18, +0x1c, +0x20, +0x24 and +0x28, a u32 into +0… |
| `00b792b0` | ReadBody_00b792b0 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, float4 values via 008e2410 into this+0x20, +0x30 and +0x40, two u32 val… |
| `00b796a0` | ReadBody_00b796a0 | likely | Calls 00b77fc0 to read the inner header, then reads a u32 into this+0xc and a u32 into this+0x10. A short read zeroes the field, sets the reader error byte and… |
| `00b79710` | ReadBody_00b79710 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, a u32 into this+0x20, a u32 into this+0x24, a u64 into this+0x18 and th… |
| `00b79830` | ReadBody_00b79830 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, then a counted byte span: an int32 length that must be non-negative and… |
| `00b798c0` | ReadBody_00b798c0 | likely | Calls 00b77fc0 to read the inner header, then reads u32 values into this+0xc, this+0x10 and this+0x14. A short read zeroes the field, sets the reader error byt… |
| `00b79950` | ReadBody_00b79950 | likely | Calls 00b77fc0 to read the inner header, then reads a u8 into this+0xc, a float4 via 008e2410 into this+0x10, and a u64 into this+0x20 and this+0x24. A short r… |
| `00b799d0` | ReadBody_00b799d0 | likely | Calls 00b77fc0 to read the inner header, then reads a u32 into this+0xc and a u32 into this+0x10; each sets the reader flag byte reader+0x11 when the float bit… |
| `00b79a60` | ReadBody_00b79a60 | likely | Calls 00b77fc0 to read the inner header, then reads fourteen u32 values into this+0xc through +0x40 (step 4), a u64 into this+0x48 and this+0x4c, and u32 value… |
| `00b79cc0` | ReadBody_00b79cc0 | likely | Calls 00b77fc0 to read the inner header, then reads a u64 into this+0x10 and this+0x14, then u32 values into this+0x18 and this+0x1c. A short read zeroes the f… |
| `00b79d50` | Ctor_00b79d50 | likely | Constructs a 0x3c-byte record: zeroes dwords +0x0, +0x4, +0x10 through +0x20, +0x2c and +0x30; stores the globals DAT_01be99c0 and DAT_01be99c4 at +0x8 and +0x… |
| `00b79e00` | CopyCtor_00b79e00 | likely | Copy constructor of the 0x3c record: copies dwords +0x0 through +0x20 from the source, installs the SoeUtil::IString<char> vtable at +0x24, sets +0x28 to the s… |
| `00b79ed0` | CopyAssign_00b79ed0 | likely | Copy-assigns the record (object offsets up to +0xec) from the source argument: dwords +0x4, +0x8, +0x10 through +0x88, +0x9c, +0xa0, and +0xb0 through +0xec ar… |
| `00b7acb0` | CopyConstructAndReturnThis | likely | Calls 00b79e00 (copy constructor of the 0x3c record) on this with the stack argument as the source, then returns this with ret 4. |
| `00b7ae90` | ReadBody_00b7ae90 | likely | Calls 00b77fc0 to read the inner header, then reads float4 values via 008e2410 into this+0x10, +0x20, +0x30, +0x40 and +0x50; u8 values into this+0x60 and +0x6… |
| `00b7b190` | ReadBodyRange_00b7b190 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b290` | ReadBodyRange_00b7b290 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b300` | ReadBodyRange_00b7b300 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b370` | ReadBodyRange_00b7b370 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b3e0` | ReadBodyRange_00b7b3e0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b450` | ReadBodyRange_00b7b450 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b4c0` | ReadBodyRange_00b7b4c0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b530` | ReadBodyRange_00b7b530 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b5a0` | ReadBodyRange_00b7b5a0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b610` | ReadBodyRange_00b7b610 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b680` | ReadBodyRange_00b7b680 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b6f0` | ReadBodyRange_00b7b6f0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b8b0` | ReadBodyRange_00b7b8b0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b920` | ReadBodyRange_00b7b920 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7b990` | ReadBodyRange_00b7b990 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7ba00` | ReadBodyRange_00b7ba00 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7ba70` | ReadBodyRange_00b7ba70 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7bae0` | ReadBodyRange_00b7bae0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7bb50` | ReadBodyRange_00b7bb50 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |
| `00b7bbc0` | ReadBodyRange_00b7bbc0 | likely | Cdecl-style: stack arguments are the packet object, buffer start, length and allowTrailing flag. Returns 0 at once when the object or the buffer pointer is nul… |

### `net.packet.social_share`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009028b0` | BaseClient::DispatchSocialShareSubOp 🌐 | likely | Virtual, thiscall (ecx = this, saved in ebx). Constructs a local social-share packet object through FUN_008a5040, reads an i16 pair from the buffer (param_1, l… |

**BaseSocialSharePacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5040` | BaseSocialSharePacket::BaseSocialSharePacket 🌐 | confirmed | Social-share family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 205 (0xcd) at +4, installs BaseSocialSharePacket::vftable a… |
| `008a50a0` | BaseSocialSharePacket::~BaseSocialSharePacket | confirmed | Destructor body: restores BaseSocialSharePacket::vftable then BasePacket::vftable; 14 EH funclets (0158a320 and the 015caxxx block) call it. |
| `008a50f0` | BaseSocialSharePacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: in an SEH frame restores BaseSocialSharePacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
| `00ade960` | BaseSocialSharePacket::BaseSocialSharePacket | confirmed | Base constructor for the social-share family: installs BasePacket::vftable, writes 0xcd (205) into the 2-byte packet id slot at +4, installs BaseSocialSharePac… |
| `00adea30` | BaseSocialSharePacket::~BaseSocialSharePacket | confirmed | Destructor body that restores BaseSocialSharePacket::vftable and then BasePacket::vftable; a single EH unwind funclet calls this copy. |
| `00adeae0` | BaseSocialSharePacket::~BaseSocialSharePacket | confirmed | Destructor body that restores BaseSocialSharePacket::vftable and then BasePacket::vftable; a single EH unwind funclet calls this second copy. |

**SocialSharePacketGetProviderStatusRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ade9d0` | SocialSharePacketGetProviderStatusRequest::SocialSharePacke… | confirmed | Calls base constructor 00ade960 with sub-opcode 1 and installs SocialSharePacketGetProviderStatusRequest::vftable. |
| `00aded30` | SocialSharePacketGetProviderStatusRequest::ScalarDeletingDt… | confirmed | Virtual deleting destructor: restores BaseSocialSharePacket::vftable then BasePacket::vftable inside an SEH frame and frees the object when the flag argument h… |

**SocialSharePacketGetProviderStatusResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adf140` | SocialSharePacketGetProviderStatusResponse::SocialSharePack… 🌐 | confirmed | Packet constructor: calls the family base constructor 00ade960 with sub-opcode 2, installs SocialSharePacketGetProviderStatusResponse::vftable at +0, and initi… |
| `00adf1c0` | SocialSharePacketGetProviderStatusResponse::~SocialSharePac… | confirmed | Destructor body: releases the second embedded IString at +0x1c (shared buffer at +0x20 when length +0x28 is positive), then the first IString at +0xc (buffer +… |
| `00adf490` | SocialSharePacketGetProviderStatusResponse::ScalarDeletingD… | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 00adf1c0 on this, then, when bit 0 of the flag argument is set, calls _free(this), and fi… |

**SocialSharePacketPublishRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adf300` | SocialSharePacketPublishRequest::~SocialSharePacketPublishR… | confirmed | Destructor body: releases the embedded IString at +0x24 (shared buffer at +0x28 when length +0x30 is positive), then the IString at +0x14 (buffer +0x18, length… |
| `00adf4b0` | SocialSharePacketPublishRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 00adf300 on this, then, when bit 0 of the flag argument is set, calls _free(this), and fi… |

**SocialSharePacketPublishResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adea80` | SocialSharePacketPublishResponse::SocialSharePacketPublishR… | confirmed | Calls base constructor 00ade960 with sub-opcode 4 and installs SocialSharePacketPublishResponse::vftable. |
| `00adeda0` | SocialSharePacketPublishResponse::ScalarDeletingDtor | confirmed | Virtual deleting destructor: restores BaseSocialSharePacket::vftable then BasePacket::vftable inside an SEH frame and frees the object when the flag argument h… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adfa90` | ReadSocialSharePublishResponseFields | likely | Prepares the input cursor, reads two 32-bit values with bounds checks, writes them at destination offsets +0xc/+0x10, and marks the cursor failed when data is … |
| `00ae02c0` | WriteProviderPair 🌐 | likely | Builds a 2-byte value from the first input field, writes up to two bytes to an output buffer, then writes a 2-byte value from the second input field and advanc… |
| `00ae06f0` | WriteProviderStatusResponse | likely | Writes the common provider pair, then two 32-bit fields from source offsets +0xc and +0x10, advancing the output cursor by the bytes actually accepted. |

### `net.packet.ui`

**BaseTaskPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86710` | BaseTaskPacketCtor 🌐 | confirmed | Base constructor of the task packet family (a BaseUiPacket subclass): runs the BaseUiPacket constructor 00a86640(param_2) with the caller's sub-opcode, stores … |
| `00a87d80` | BaseTaskPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of BaseTaskPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set… |

**BaseUiPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86640` | BaseUiPacketCtor 🌐 | confirmed | Base constructor of the BaseUiPacket (class 47) family: stores BasePacket's vtable, writes the family class id 0x2f (47) into +4, stores the BaseUiPacket vtabl… |
| `00a87d20` | BaseUiPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of BaseUiPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; … |

**CinematicStartLookAtPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86ec0` | CinematicStartLookAtPacketDtorBody | likely | Destroys the embedded Target at this+0x14, then the Target at this+0x0c, through 0101b370, and restores ClientGateway::BasePacket in the object vtable. |

**ExecuteScriptPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a8b710` | ExecuteScriptPacket::Construct | confirmed | Constructs an ExecuteScriptPacket: calls the BaseUiPacket constructor 00a86640 with sub-opcode 7, stores ExecuteScriptPacket::vftable at *this, an empty IStrin… |

**LoadingScreenPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a8b820` | LoadingScreenPacket::Construct | confirmed | Constructs a LoadingScreenPacket: calls the BaseUiPacket constructor 00a86640 with sub-opcode 9, stores LoadingScreenPacket::vftable at *this, initialises an e… |
| `00a8b8c0` | LoadingScreenPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of LoadingScreenPacket: calls 00a89940 (PacketIStringPairDtorBody), then calls _free(this) through thunk 00d82463 when bit 0 of para… |

**ObjectiveTargetUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a88080` | ObjectiveTargetUpdatePacket::Construct | confirmed | Constructs an ObjectiveTargetUpdatePacket: calls the BaseUiPacket constructor 00a86640 with sub-opcode 0xe, stores ObjectiveTargetUpdatePacket::vftable at *thi… |
| `00a88120` | ObjectiveTargetUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of ObjectiveTargetUpdatePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of p… |

**SelectTaskRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86b30` | SelectTaskRequestCtor 🌐 | confirmed | Constructor of SelectTaskRequest: runs the BaseUiPacket constructor with sub-opcode 6 (class id 47), stores the SelectTaskRequest vtable and saves the three ca… |
| `00a87f60` | SelectTaskRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor of SelectTaskRequest: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is … |

**StartTimerPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a88020` | StartTimerPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of StartTimerPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is s… |

**TaskAddPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87de0` | TaskAddPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of TaskAddPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set;… |
| `00a867e0` | TaskAddPacketCtor 🌐 | likely | Constructor of TaskAddPacket: runs the task base constructor with sub-opcode 1 and second argument 0xffffffff, stores the TaskAddPacket vtable and zeroes the p… |

**TaskCompletePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87ea0` | TaskCompletePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of TaskCompletePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is… |

**TaskFailPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86a80` | TaskFailPacketCtor 🌐 | confirmed | Constructor of TaskFailPacket: runs the task base constructor with sub-opcode 4 and the caller's id argument, then stores the TaskFailPacket vtable. |
| `00a87f00` | TaskFailPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of TaskFailPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set… |

**TaskUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86900` | TaskUpdatePacketCtor 🌐 | confirmed | Constructor of TaskUpdatePacket: runs the task base constructor with sub-opcode 2, argument 0xffffffff, stores the TaskUpdatePacket vtable and initialises the … |
| `00a87e40` | TaskUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of TaskUpdatePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is s… |

**UiMessagePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86d70` | UiMessagePacketCtor 🌐 | confirmed | Constructor of UiMessagePacket: runs the BaseUiPacket constructor with sub-opcode 0xf (class id 47), stores the UiMessagePacket vtable and saves param_2 into +… |
| `00a881a0` | UiMessagePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor of UiMessagePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is se… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a91bf0` | BaseUiPacketDispatch 🌐 | confirmed | Reads a subtype with 00a8b240, invokes its case reader, and dispatches parsed values to packet or UI callbacks. |

### `net.packet.vehicle`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00944840` | BaseClient::ParseVehicleFamilyPacket | confirmed | Virtual slot 76 of BaseClient (shared with PlayClient). Always constructs a BaseVehicleLoadoutPacket with sub-opcode argument 0 (008a4c40), reads a 16-bit then… |

**BaseVehicleLoadoutPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4c40` | BaseVehicleLoadoutPacket::BaseVehicleLoadoutPacket 🌐 | confirmed | Vehicle packet-family constructor: installs BasePacket::vftable, stores opcode 137 (0x89) at +4, installs BaseVehicleLoadoutPacket::vftable and stores the sub-… |
| `008b7f00` | BaseVehicleLoadoutPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4cb0` | BaseVehicleLoadoutPacket::~BaseVehicleLoadoutPacket | likely | Destructor body: writes BasePacket::vftable and returns; nine EH funclets (0157dfe0 and neighbours) call it. |

**BaseVehiclePartPacket** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d2310` | BaseVehiclePartPacket::Ctor | confirmed | Installs BasePacket, writes packet opcode 0x8b, switches to the BaseVehiclePartPacket vtable, and clears the subtype field. |
| `009d2370` | BaseVehiclePartPacket::CtorWithSubtype | confirmed | Installs BasePacket, writes opcode 0x8b, installs the BaseVehiclePartPacket vtable, and stores the supplied subtype. |
| `009d23e0` | BaseVehiclePartPacket::Dtor | confirmed | Restores the BaseVehiclePartPacket vtable and then the BasePacket vtable. |
| `009d2430` | BaseVehiclePartPacket::ScalarDeletingDtor | confirmed | Restores derived and base packet vtables, then frees the object when the deleting flag is set. |
| `009d3e10` | BaseVehiclePartPacket::Dtor | confirmed | Releases the owned IString buffer when present, then restores BaseVehiclePartPacket and BasePacket vtables. |
| `00b86ae0` | BaseVehiclePartPacket::~BaseVehiclePartPacket | likely | Base destructor body used during unwinding around the vehicle-part packet constructors: resets this object to BaseVehiclePartPacket::vftable and then to BasePa… |
| `00b86b90` | BaseVehiclePartPacket_DestroyBase | likely | Second base-destructor body of the vehicle-part packet hierarchy: resets the object to BaseVehiclePartPacket::vftable and then to BasePacket::vftable. |
| `00b86c40` | BaseVehiclePartPacket_DestroyBase2 | likely | Base-destructor body for the vehicle-part packet hierarchy: resets the object to BaseVehiclePartPacket::vftable and then BasePacket::vftable. |
| `00b86cf0` | BaseVehiclePartPacket_DestroyBase3 | likely | Base-destructor body for the vehicle-part packet hierarchy: resets the object to BaseVehiclePartPacket::vftable and then BasePacket::vftable. |

**BaseVehicleRacePacket** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99270` | BaseVehicleRacePacket::BaseVehicleRacePacket_Default 🌐 | confirmed | Family constructor without a subtype word: stores BasePacket::vftable, writes the opcode 0x21 (33) into the packet id slot at +4, stores BaseVehicleRacePacket:… |
| `00b992d0` | BaseVehicleRacePacket::BaseVehicleRacePacket_WithSubtype 🌐 | confirmed | Family constructor: stores BasePacket::vftable, writes the opcode 0x21 (33) into the packet id slot at +4, stores BaseVehicleRacePacket::vftable and stores the… |
| `00b99340` | BaseVehicleRacePacket::~BaseVehicleRacePacket | confirmed | Destructor body of the vehicle race packet base: stores BaseVehicleRacePacket::vftable at this+0 and then BasePacket::vftable; touches no fields and frees noth… |
| `00b99390` | BaseVehicleRacePacket::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor: stores BaseVehicleRacePacket::vftable then BasePacket::vftable inside an SEH frame and calls _free(this) when bit 0 of param_2… |
| `00b9d230` | BaseVehicleRacePacket::CopyHeaderFrom | confirmed | Copy-initialises a packet header in place: stores BasePacket::vftable into *this, copies param_2[1] into this[1]; stores BaseVehicleRacePacket::vftable into *t… |
| `00b996b0` | BaseVehicleRacePacket_DtorEmission_A | likely | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99700` | BaseVehicleRacePacket_DtorEmission_B | likely | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99940` | BaseVehicleRacePacket_DtorEmission_C | likely | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99a40` | BaseVehicleRacePacket_DtorEmission_D | likely | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99bf0` | BaseVehicleRacePacket_DtorEmission_E | likely | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9a300` | BaseVehicleRacePacket_DtorEmission_F | likely | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**MountBasePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089f9c0` | MountBasePacket::MountBasePacket 🌐 | confirmed | Family constructor taking the sub-opcode: installs BasePacket::vftable, writes 0xa8 (168) into the packet id slot param_1[1], installs MountBasePacket::vftable… |

**VehiclePartPacketRequestRemovePartFromSlot** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86f80` | VehiclePartPacketRequestRemovePartFromSlot::ScalarDeletingD… | confirmed | Vtable slot 0 of the remove-part request packet: resets the base vtables and frees the object through _free when bit 0 of param_2 is set. |

**VehiclePartPacketRequestSelectVehicle** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86be0` | VehiclePartPacketRequestSelectVehicle::VehiclePartPacketReq… 🌐 | confirmed | Constructor of the select-vehicle request packet: runs the base packet constructor FUN_009d2370 with sub-opcode constant 5, installs the class vtable and zeroe… |
| `00b86f10` | VehiclePartPacketRequestSelectVehicle::ScalarDeletingDtor | confirmed | Vtable slot 0 of the select-vehicle request packet: resets the base vtables and frees the object through _free when bit 0 of param_2 is set. |

**VehiclePartPacketRequestVehicleConfig** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86a80` | VehiclePartPacketRequestVehicleConfig::VehiclePartPacketReq… 🌐 | confirmed | Constructor of the vehicle-config request packet: runs the base packet constructor FUN_009d2370 with sub-opcode constant 1 and installs VehiclePartPacketReques… |
| `00b86e30` | VehiclePartPacketRequestVehicleConfig::ScalarDeletingDtor | confirmed | Vtable slot 0 of the vehicle-config request packet: resets the object to BaseVehiclePartPacket::vftable then BasePacket::vftable and frees the packet through t… |

**VehiclePartPacketRequestVehicleEquipmentArrayUpdate** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86ea0` | VehiclePartPacketRequestVehicleEquipmentArrayUpdate::Scalar… | confirmed | Vtable slot 0 of the equipment-array update request packet: resets the base vtables (BaseVehiclePartPacket then BasePacket) and frees the object through _free … |

**VehiclePartPacketVehicleConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d6770` | VehiclePartPacketVehicleConfig::Ctor | confirmed | Fastcall constructor: runs BaseVehiclePartPacket::CtorWithSubtype with sub-opcode 2 (009d2370), stores the VehiclePartPacketVehicleConfig vftable at *this, con… |
| `009d67e0` | VehiclePartPacketVehicleConfig::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls 009d62b0 (the VehicleConfig packet destructor body) and calls _free(this) when bit 0 of the stack flag is set… |
| `009d62b0` | VehiclePartPacketVehicleConfig::DtorBody | likely | Fastcall destructor body: destroys the Array<VehicleEquipmentEntry,16,0> member through 00907d60 and the VehicleConfig member through 01056260, then stores the… |

**VehiclePartPacketVehicleUpdateDecal** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d3da0` | VehiclePartPacketVehicleUpdateDecal::Ctor | confirmed | Initializes the base vehicle-part packet with subtype 4, installs the derived vtable, initializes an IString field, and clears two trailing dwords. |
| `009d4a90` | VehiclePartPacketVehicleUpdateDecal::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the base destructor body 009d3e10 (BaseVehiclePartPacket::Dtor) and calls _free(this) when bit 0 of the stack flag is set; re… |

**VehicleRacePacketBoostAwardRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c7d0` | VehicleRacePacketBoostAwardRequest::VehicleRacePacketBoostA… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x16) (inner opcode 33 subtype 22) and stores VehicleRacePacketB… |
| `00b9c830` | VehicleRacePacketBoostAwardRequest::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |

**VehicleRacePacketBoostAwardResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99c40` | VehicleRacePacketBoostAwardResponse::VehicleRacePacketBoost… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x17 (23) and then stores VehicleRacePacketBoostAwardResponse::vftable; returns th… |
| `00b99ca0` | VehicleRacePacketBoostAwardResponse::~VehicleRacePacketBoos… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketCreateProxiedVehicleRace** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c910` | VehicleRacePacketCreateProxiedVehicleRace::VehicleRacePacke… 🌐 | confirmed | Constructs the 33/11 create-proxied-vehicle packet: calls the family base constructor FUN_00b992d0(0xb), stores VehicleRacePacketCreateProxiedVehicleRace::vfta… |
| `00b9c9f0` | VehicleRacePacketCreateProxiedVehicleRace::ScalarDeletingDt… | confirmed | MSVC scalar deleting destructor: calls FUN_00b99cf0(this) (the out-of-line base destructor that resets the base vtables) and, when the low byte of param_2 is s… |
| `00ba7370` | VehicleRacePacketCreateProxiedVehicleRace::ReadPayload 🌐 | confirmed | Reads the vehicle-race header, a variable-length vehicle definition, an 8-byte owner id, four 12-byte vectors into fields at this+0xe1c through this+0xe40, the… |
| `00b99cf0` | VehicleRacePacketCreateProxiedVehicleRace::~VehicleRacePack… | likely | Destructor of the create-proxied-vehicle packet: runs FUN_01056260 (which destroys a VehicleConfig and its friction-material hash map), then stores BaseVehicle… |

**VehicleRacePacketCreateProxiedVehicleRaceExplosion** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9ee60` | VehicleRacePacketCreateProxiedVehicleRaceExplosion::Ctor 🌐 | confirmed | Constructs the 33/27 create-proxied-explosion packet: calls the family base constructor FUN_00b992d0(0x1b), stores VehicleRacePacketCreateProxiedVehicleRaceExp… |
| `00b9f320` | VehicleRacePacketCreateProxiedVehicleRaceExplosion::ScalarD… | confirmed | MSVC scalar deleting destructor: calls 00b9eed0(this) to destroy the class (including its embedded IString) and, when the low byte of param_2 is set, calls _fr… |
| `00b9eed0` | VehicleRacePacketCreateProxiedVehicleRaceExplosion::Dtor | likely | Destroys the 33/27 explosion packet in place: resets the embedded SoeUtil::IString<char> at +0x40 (releasing its refcounted buffer through virtual slot 2 when … |

**VehicleRacePacketCreateProxiedVehicleRaceMine** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9ccf0` | VehicleRacePacketCreateProxiedVehicleRaceMine::VehicleRaceP… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x1d) (inner opcode 33 subtype 29) and stores VehicleRacePacketC… |
| `00b9cd50` | VehicleRacePacketCreateProxiedVehicleRaceMine::ScalarDeleti… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99f40` | VehicleRacePacketCreateProxiedVehicleRaceMine::~VehicleRace… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketCreateProxiedVehicleRaceMissile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9df00` | VehicleRacePacketCreateProxiedVehicleRaceMissile::Ctor 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x18) (inner opcode 33 subtype 24) and stores VehicleRacePacketC… |
| `00b9df60` | VehicleRacePacketCreateProxiedVehicleRaceMissile::ScalarDel… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99df0` | Destruct | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketCreateProxiedVehicleRacePickup** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9de30` | VehicleRacePacketCreateProxiedVehicleRacePickup::VehicleRac… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xf) (inner opcode 33 subtype 15) and stores VehicleRacePacketCr… |
| `00b9de90` | VehicleRacePacketCreateProxiedVehicleRacePickup::ScalarDele… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99840` | Destruct | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00ba0070` | ParseCreatePickupPacket | likely | Parses a 33/15 create-proxied-pickup payload: header via 00b9fab0, then u32 at +0xc, u32 at +0x10, a 16-byte float4 via FUN_008e2410 into +0x20, a float at +0x… |

**VehicleRacePacketDestroyProxiedVehicleRace** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9ca10` | VehicleRacePacketDestroyProxiedVehicleRace::VehicleRacePack… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xc) (inner opcode 33 subtype 12) and stores VehicleRacePacketDe… |
| `00b9ca70` | VehicleRacePacketDestroyProxiedVehicleRace::ScalarDeletingD… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99d50` | VehicleRacePacketDestroyProxiedVehicleRace::~VehicleRacePac… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketDestroyProxiedVehicleRaceExplosion** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9a090` | Construct | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x1c (28) and then stores VehicleRacePacketDestroyProxiedVehicleRaceExplosion::vft… |
| `00b9cf00` | VehicleRacePacketDestroyProxiedVehicleRaceExplosion::Scalar… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b9a0f0` | Destruct | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketDestroyProxiedVehicleRaceMine** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99f90` | VehicleRacePacketDestroyProxiedVehicleRaceMine::VehicleRace… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x1e (30) and then stores VehicleRacePacketDestroyProxiedVehicleRaceMine::vftable;… |
| `00b9cdc0` | VehicleRacePacketDestroyProxiedVehicleRaceMine::ScalarDelet… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99ff0` | VehicleRacePacketDestroyProxiedVehicleRaceMine::~VehicleRac… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketDestroyProxiedVehicleRaceMissile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99e40` | Construct | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x19 (25) and then stores VehicleRacePacketDestroyProxiedVehicleRaceMissile::vftab… |
| `00b99ea0` | Destruct | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketDestroyProxiedVehicleRacePickup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99890` | Construct | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x10 (16) and then stores VehicleRacePacketDestroyProxiedVehicleRacePickup::vftabl… |
| `00b998f0` | Destruct | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketGateHit** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c130` | VehicleRacePacketGateHit::VehicleRacePacketGateHit 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(7) (inner opcode 33 subtype 7) and stores VehicleRacePacketGateH… |

**VehicleRacePacketItemExpired** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99b40` | VehicleRacePacketItemExpired::VehicleRacePacketItemExpired | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x15 (21) and then stores VehicleRacePacketItemExpired::vftable; returns this. |
| `00b99ba0` | VehicleRacePacketItemExpired::~VehicleRacePacketItemExpired | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketKickProxiedVehicleRace** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9cae0` | VehicleRacePacketKickProxiedVehicleRace::VehicleRacePacketK… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xd) (inner opcode 33 subtype 13) and stores VehicleRacePacketKi… |
| `00b99da0` | VehicleRacePacketKickProxiedVehicleRace::~VehicleRacePacket… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketPickupHitRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c4e0` | VehicleRacePacketPickupHitRequest::VehicleRacePacketPickupH… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x11) (inner opcode 33 subtype 17) and stores VehicleRacePacketP… |

**VehicleRacePacketPickupHitResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99990` | VehicleRacePacketPickupHitResponse::VehicleRacePacketPickup… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x12 (18) and then stores VehicleRacePacketPickupHitResponse::vftable; returns thi… |
| `00b999f0` | VehicleRacePacketPickupHitResponse::~VehicleRacePacketPicku… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketRegisterVehicleRacePlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c2d0` | VehicleRacePacketRegisterVehicleRacePlayer::VehicleRacePack… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(9) (inner opcode 33 subtype 9) and stores VehicleRacePacketRegis… |
| `00b997a0` | VehicleRacePacketRegisterVehicleRacePlayer::~VehicleRacePac… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketResetNpcsRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9a2a0` | VehicleRacePacketResetNpcsRequest::VehicleRacePacketResetNp… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x20 (32) and then stores VehicleRacePacketResetNpcsRequest::vftable; returns this. |
| `00b9d1a0` | VehicleRacePacketResetNpcsRequest::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |

**VehicleRacePacketResetProxiedVehicleRace** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9d040` | VehicleRacePacketResetProxiedVehicleRace::VehicleRacePacket… 🌐 | confirmed | Constructs the 33/33 reset-proxied-vehicle packet: calls the family base constructor FUN_00b992d0(0x21), stores VehicleRacePacketResetProxiedVehicleRace::vftab… |
| `00b9d110` | VehicleRacePacketResetProxiedVehicleRace::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b9a190` | VehicleRacePacketResetProxiedVehicleRace::~VehicleRacePacke… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00ba0260` | ParseResetProxiedVehicleRacePacket | likely | Parses a 33/33 reset payload: header via 00b9fab0, u64 at +0x10/+0x14, then four 12-byte vectors via FUN_009d4230 into +0x18, +0x24, +0x30 and +0x3c; every rea… |

**VehicleRacePacketSetRaceConfig** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9a1e0` | VehicleRacePacketSetRaceConfig::VehicleRacePacketSetRaceCon… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x22 (34), stores VehicleRacePacketSetRaceConfig::vftable and then constructs the … |
| `00b9d180` | VehicleRacePacketSetRaceConfig::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls FUN_00b9a240(this) (the out-of-line base destructor that restores the base vtables) and, when the low byte of param_2 is… |
| `00ba0680` | ParseSetRaceConfigBody | confirmed | Thiscall parser of an 84-byte body with no header read: u32 to +0x30, u32 to +0x34, float to +0x38 (sets reader NaN byte +0x11 when NaN), four float4 via ReadF… |
| `00b9a240` | VehicleRacePacketSetRaceConfig::~VehicleRacePacketSetRaceCo… | likely | Destructor of the set-race-config packet: destroys the embedded RaceConfig member with FUN_0107cdd0, then stores BaseVehicleRacePacket::vftable and BasePacket:… |

**VehicleRacePacketUpdateNpcCount** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b995b0` | VehicleRacePacketUpdateNpcCount::VehicleRacePacketUpdateNpc… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 4 and then stores VehicleRacePacketUpdateNpcCount::vftable; returns this. |
| `00b9bdc0` | VehicleRacePacketUpdateNpcCount::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99610` | VehicleRacePacketUpdateNpcCount::~VehicleRacePacketUpdateNp… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdatePlayerCount** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99500` | VehicleRacePacketUpdatePlayerCount::VehicleRacePacketUpdate… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 3 and then stores VehicleRacePacketUpdatePlayerCount::vftable; returns this. |
| `00b99560` | VehicleRacePacketUpdatePlayerCount::~VehicleRacePacketUpdat… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleGamePlayerState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9bc80` | VehicleRacePacketUpdateVehicleGamePlayerState::VehicleRaceP… 🌐 | confirmed | Constructs an empty vehicle-race packet object in place: calls the family base constructor FUN_00b992d0(2) (inner opcode 33, subtype 2) and stores VehicleRaceP… |
| `00b994b0` | VehicleRacePacketUpdateVehicleGamePlayerState::~VehicleRace… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleGameState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99400` | VehicleRacePacketUpdateVehicleGameState::VehicleRacePacketU… | likely | Subtype constructor: calls the family base constructor 00b992d0 with subtype 1 and then stores VehicleRacePacketUpdateVehicleGameState::vftable; returns this. |
| `00b99460` | VehicleRacePacketUpdateVehicleGameState::~VehicleRacePacket… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleRaceClientData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9be30` | VehicleRacePacketUpdateVehicleRaceClientData::VehicleRacePa… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(5) (inner opcode 33 subtype 5) and stores VehicleRacePacketUpdat… |
| `00b9dfd0` | VehicleRacePacketUpdateVehicleRaceClientData::CopyCtor | confirmed | Copy-constructs a 33/5 client-data packet: calls the base header copy 00b9d230, stores the class vftable, then copies the payload dwords +0xc through +0x54, th… |
| `00b99660` | VehicleRacePacketUpdateVehicleRaceClientData::~VehicleRaceP… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleRaceMineData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9ce30` | VehicleRacePacketUpdateVehicleRaceMineData::VehicleRacePack… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x1f) (inner opcode 33 subtype 31) and stores VehicleRacePacketU… |
| `00b9ce90` | VehicleRacePacketUpdateVehicleRaceMineData::ScalarDeletingD… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b9a040` | VehicleRacePacketUpdateVehicleRaceMineData::~VehicleRacePac… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleRaceMissileData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9cc20` | VehicleRacePacketUpdateVehicleRaceMissileData::VehicleRaceP… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x1a) (inner opcode 33 subtype 26) and stores VehicleRacePacketU… |
| `00b9cc80` | VehicleRacePacketUpdateVehicleRaceMissileData::ScalarDeleti… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b99ef0` | VehicleRacePacketUpdateVehicleRaceMissileData::~VehicleRace… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleRaceProgressData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9cfd0` | VehicleRacePacketUpdateVehicleRaceProgressData::ScalarDelet… | confirmed | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero ca… |
| `00b9fc80` | ParseProgressDataPacket | confirmed | Parses a 33/14 progress payload in this read order: header via 00b9fab0, then u64 guid into +0x10/+0x14, u32 into +0x38, floats into +0x3c, +0x40, u32 into +0x… |
| `00b9a140` | VehicleRacePacketUpdateVehicleRaceProgressData::~VehicleRac… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketUpdateVehicleRaceServerData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9bfb0` | VehicleRacePacketUpdateVehicleRaceServerData::VehicleRacePa… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(6) (inner opcode 33 subtype 6) and stores VehicleRacePacketUpdat… |

**VehicleRacePacketUseItemRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c620` | VehicleRacePacketUseItemRequest::VehicleRacePacketUseItemRe… 🌐 | likely | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x13) (inner opcode 33 subtype 19) and stores VehicleRacePacketU… |

**VehicleRacePacketUseItemResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b99a90` | VehicleRacePacketUseItemResponse::VehicleRacePacketUseItemR… | confirmed | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x14 (20) and then stores VehicleRacePacketUseItemResponse::vftable; returns this. |
| `00b99af0` | VehicleRacePacketUseItemResponse::~VehicleRacePacketUseItem… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRacePacketVehicleHit** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c200` | VehicleRacePacketVehicleHit::VehicleRacePacketVehicleHit 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(8) (inner opcode 33 subtype 8) and stores VehicleRacePacketVehic… |
| `00b99750` | VehicleRacePacketVehicleHit::~VehicleRacePacketVehicleHit | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9ffb0` | ParseVehicleHitPacket | likely | Parses a 33/8 vehicle-hit payload: header via 00b9fab0, then u64 at +0x10/+0x14, u64 at +0x18/+0x1c and a 4-byte float at +0x20 (setting the reader's invalid-v… |

**VehicleRacePacketVehicleRaceCountdownStatus** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9c3a0` | VehicleRacePacketVehicleRaceCountdownStatus::VehicleRacePac… 🌐 | confirmed | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(10) (inner opcode 33 subtype 10) and stores VehicleRacePacketVeh… |
| `00b997f0` | VehicleRacePacketVehicleRaceCountdownStatus::~VehicleRacePa… | likely | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |

**VehicleRefreshConfigsPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4d10` | VehicleRefreshConfigsPacket::VehicleRefreshConfigsPacket 🌐 | confirmed | Constructor: calls FUN_008a4c40(3), installs VehicleRefreshConfigsPacket::vftable, then calls FUN_0101dcd0 with ECX pointing to the subobject at this+0xc insid… |
| `008b7f60` | VehicleRefreshConfigsPacket::ScalarDeletingDtor | confirmed | Calls VehicleRefreshConfigsPacket destructor, then frees the object when the deleting flag bit is set. |
| `008a4d70` | VehicleRefreshConfigsPacket::~VehicleRefreshConfigsPacket | likely | Destructor body calls FUN_0101d850 with ECX pointing to the subobject at this+0xc, then restores BasePacket::vftable. Callers include 008b7f60, vehicle dispatc… |

**VehicleUpdateConfigPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009154f0` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 0180504c (thiscall, one byte argument). Calls FUN_0090bca0 (destructor body; label accepted:confirmed BasePacketSubobjectCleanup) with… |

**VehicleUpdateItemPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a4dd0` | VehicleUpdateItemPacket::VehicleUpdateItemPacket 🌐 | confirmed | Constructor: calls FUN_008a4c40(2), installs VehicleUpdateItemPacket::vftable, zeroes payload dwords at +0xc and +0x10, then calls FUN_0101cd40 with ECX pointi… |
| `008b7f80` | VehicleUpdateItemPacket::ScalarDeletingDtor | confirmed | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
| `008a4e40` | VehicleUpdateItemPacket::~VehicleUpdateItemPacket | likely | Destructor body: writes BasePacket::vftable and returns; the only caller is EH unwind funclet 0159334c. |

**(no class)** (67)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9fab0` | ReadPacketHeader | confirmed | Reads the two header words of a vehicle-race packet from the reader at param_2: each u16 is bounds-checked against the reader end pointer at +0xc (on truncatio… |
| `00ba04f0` | ParseCountdownPayload | confirmed | Thiscall parser (record = this, reader = arg): calls ReadPacketHeader (00b9fab0), then reads a u64 (two dwords) into +0x10/+0x14 and a u32 into +0x18. A short … |
| `00ba1070` | ParseExact_Sub05 | confirmed | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise it bu… |
| `00ba1850` | ParseExact_Sub10 | confirmed | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba3670` | ParseExact_Sub34 | confirmed | Exact-length entry for the subtype-34 race-config packet (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise bu… |
| `00ba7460` | ValidateVehicleRaceCreateProxiedVehiclePayload 🌐 | confirmed | If the packet object and input pointer are non-null, initializes a bounded reader from the byte span, calls 00ba7370, then returns one only when the reader err… |
| `009d4420` | ReadFloat46Block | likely | Reads 16 floats into the destination (NaN flag on NaN, zero and error byte on exhaustion), then 30 more through 008d53b0 (ReaderReadFloat) into destination flo… |
| `009d64d0` | ReadCountedKeyedIStringRecords | likely | Reads a 32-bit count, then grows the 0x18-record array through 009d5a60 or truncates the excess through 008e4580, sets the count, and reads each record with 00… |
| `009d6560` | ReadCountedDwordPairFloat4Records | likely | Reads a 32-bit count, grows the 0x50-byte record array through 009d34b0 when capacity is short, sets the count, then reads each record with 009d5b10 (two dword… |
| `009d65e0` | ReadCountedIStringArray16 | likely | Reads a 32-bit count (rejects negative), grows the 16-byte IString array through 009d5ba0 or truncates it through 0071b470, sets the count, and reads each elem… |
| `009d6b20` | ReadCounted0x30Records | likely | Reads a 32-bit count, grows the 0x30 record array through 009d5c40 or truncates through 009d4de0, sets the count, and reads each record with 009d6680. |
| `009d6d70` | ReadCounted0x80Records | likely | Reads a 32-bit count, grows the 0x80 record array through 009d6bb0 or truncates through 009d6480, sets the count, and reads each record with 009d6060. |
| `009d7b50` | ReadVehicleDefinition | likely | Reads the variable-length vehicle definition into this: a length-prefixed name blob (00894b10), scalar readers (008c9d20, 008c9dc0, 008cb500, 008d53b0, 009b066… |
| `009d88b0` | ReadVehiclePartMessageBody | likely | Reads a vehicle part message body: an int16 pair through 009d4e20, the vehicle definition through 009d7b50, a counted keyed IString list through 009d64d0 into … |
| `00b87130` | VehicleEquipmentUpdate_WritePayload | likely | Payload writer for the vehicle equipment array update: first writes the two 16-bit fields via 00b87080, then appends the dword at param_1+0xc (bounded by the w… |
| `00b87190` | VehicleSelect_WritePayload | likely | Payload writer for the select-vehicle request: writes the two 16-bit fields via 00b87080 and then the two dwords at param_1+0xc and param_1+0x10 through FUN_00… |
| `00b87230` | VehicleRemovePart_WritePayload | likely | Payload writer for the remove-part request: writes the two 16-bit fields via 00b87080 and then the dword at param_1+0xc through FUN_007cc400 with cursor/end bo… |
| `00b87310` | SendVehicleConfigRequest 🌐 | likely | Builds and sends the vehicle-config request: initializes a 0x2018-byte serializer with FUN_008d4a40(0x10000000), serializes the passed packet through 00b87080,… |
| `00b87420` | SendVehicleEquipmentUpdateRequest 🌐 | likely | Same send envelope as 00b87310 but serializes through the equipment-update payload writer 00b87130: init serializer, write payload, build PacketTunneledClientP… |
| `00b87530` | SendSelectVehicleRequest 🌐 | likely | Send envelope for the select-vehicle request: serializer init, payload via 00b87190, tunneled client packet construction, length copy, send and teardown. |
| `00b87640` | SendRemovePartFromSlotRequest 🌐 | likely | Send envelope for the remove-part request: serializer init, payload via 00b87230, tunneled client packet construction, length copy, send and teardown. |
| `00ba0350` | ParsePayload_Sub27 | likely | Thiscall parser (record = this, reader = arg): calls ReadPacketHeader (00b9fab0), then reads u32 to +0xc, u32 to +0x10, a u64 (two dwords) to +0x18/+0x1c, a fl… |
| `00ba0890` | ParsePayload_Sub03 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba08d0` | ParsePayload_Sub04 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0920` | ParsePayload_Sub13 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads a u64 (two dwords) into record+0x10/+0x14. A short rea… |
| `00ba0970` | ParsePayload_Sub12 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads a u64 (two dwords) into record+0x10/+0x14. A short rea… |
| `00ba09e0` | ParsePayload_Sub16 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0a20` | ParsePayload_Sub18 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0a70` | ParsePayload_Sub20 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0ab0` | ParsePayload_Sub21 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0b00` | ParsePayload_Sub25 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0b40` | ParsePayload_Sub30 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0b90` | ParsePayload_Sub28 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0be0` | ParsePayload_Sub01 | likely | Cdecl parser (reader = arg1, record = arg2): calls ReadPacketHeader (00b9fab0) on the reader, then reads one u32 into record+0xc. A short read zeroes it, sets … |
| `00ba0c40` | ParsePayload_Sub24 | likely | Thiscall parser (record = this, reader = arg): calls ReadPacketHeader (00b9fab0), then u32 to +0xc, u32 to +0x10, four float4 via ReadFloat4 (008e2410) to +0x2… |
| `00ba0d60` | ParsePayload_Sub26 | likely | Thiscall parser (record = this, reader = arg): calls ReadPacketHeader (00b9fab0), then u32 to +0xc, u64 to +0x10/+0x14, six float4 via ReadFloat4 (008e2410) to… |
| `00ba0e50` | ParsePayload_Sub31 | likely | Thiscall parser (record = this, reader = arg): calls ReadPacketHeader (00b9fab0), then u32 to +0xc and four float4 via ReadFloat4 (008e2410) to +0x10/+0x20/+0x… |
| `00ba0ec0` | ParsePayload_Sub29 | likely | Thiscall parser (record = this, reader = arg): calls ReadPacketHeader (00b9fab0), then u32 to +0xc, u32 to +0x10, four float4 via ReadFloat4 (008e2410) to +0x2… |
| `00ba10e0` | ParseExact_Sub03 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba1150` | ParseExact_Sub04 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba11c0` | ParseExact_Sub14 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba1230` | ParseExact_Sub13 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba12a0` | ParseExact_Sub12 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba1310` | ParseExact_Sub08 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba1380` | ParseExact_Sub15 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba13f0` | ParseExact_Sub16 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba1460` | ParseExact_Sub18 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba14d0` | ParseExact_Sub23 | likely | Exact-length entry for a header-only payload (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reade… |
| `00ba1540` | ParseExact_Sub20 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba15b0` | ParseExact_Sub21 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba1620` | ParseExact_Sub33 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba1690` | ParseExact_Sub25 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba1700` | ParseExact_Sub30 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba1770` | ParseExact_Sub27 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba17e0` | ParseExact_Sub28 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba18c0` | ParseExact_Sub01 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing): returns 0 when record or data is null. Otherwise builds a reader over [data, data+length)… |
| `00ba2860` | ParseExact_Sub24 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba28d0` | ParseExact_Sub26 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba2940` | ParseExact_Sub31 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba29b0` | ParseExact_Sub29 | likely | Exact-length entry (cdecl args: record, data, length, allowTrailing; record goes in ECX for the callee): returns 0 when record or data is null. Otherwise build… |
| `00ba2a80` | WriteHeaderTwoU16 | likely | Thiscall writer (this = record, arg = writer): writes the u16 at record+4 and then the u16 at record+8 into the writer through 007cc400. Each write is clamped … |
| `00ba2b20` | WriteThreeDwords | likely | Cdecl writer (arg1 = writer, arg2 = source of three dwords): writes source[0], source[1] and source[2] in turn into the writer through 007cc400, each write cla… |
| `00ba36f0` | WriteHdrObj10Dw18 | likely | Thiscall serializer (this = record, arg = writer): writes the two-word header through 00ba2a80, calls 008fb810 on the sub-object at record+0x10, then writes th… |
| `00ba37d0` | WriteHdrObj10Obj18Dw20 | likely | Thiscall serializer (this = record, arg = writer): writes the header through 00ba2a80, calls 008fb810 on the sub-objects at record+0x10 and record+0x18, then w… |
| `00ba3840` | WriteHdrVec3x6DwordByteObj | likely | Thiscall serializer (this = record, arg = writer): writes the header through 00ba2a80, then six three-dword vectors from record+0xc, +0x18, +0x24, +0x30, +0x3c… |
| `00ba3980` | WriteHdrObj10Dw18_B | likely | Thiscall serializer (this = record, arg = writer): identical in body to 00ba36f0. Writes the two-word header through 00ba2a80, calls 008fb810 on record+0x10, t… |
| `00ba3e90` | SendTunneled_HdrObj10Dw4At20_B | likely | Send wrapper, identical to 00ba3d60 in the body: header through 00ba2a80, sub-object at record+0x10 through 008fb810, four dwords from record+0x20 through 0091… |

### `net.packet.wall_of_data`

**WallOfDataBasePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adcd60` | WallOfDataBasePacket::WallOfDataBasePacket 🌐 | confirmed | Base constructor for the wall-of-data telemetry family: installs BasePacket::vftable, writes 0xc2 (194) into the 2-byte packet id slot at +4, installs WallOfDa… |
| `00add0a0` | WallOfDataBasePacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtab… |

**WallOfDataMembershipPurchasePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adced0` | WallOfDataMembershipPurchasePacket::WallOfDataMembershipPur… 🌐 | confirmed | Base constructor call 00adcd60 with event type 5, then installs WallOfDataMembershipPurchasePacket::vftable, zeroes the byte at +0xc and the dword at +0x10. |
| `00add160` | WallOfDataMembershipPurchasePacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtab… |
| `00addbf0` | WallOfDataMembershipPurchasePacket_Serialize 🌐 | likely | Writes the wall-of-data header with 00addaa0, then appends the 1-byte flag at +0xc and the 4-byte value at +0x10. |

**WallOfDataPlayerClickMovePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adce20` | WallOfDataPlayerClickMovePacket::WallOfDataPlayerClickMoveP… 🌐 | confirmed | Base constructor call 00adcd60 with event type 3 followed by installing WallOfDataPlayerClickMovePacket::vftable. |
| `00add100` | WallOfDataPlayerClickMovePacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtab… |

**WallOfDataPlayerKeyboardPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00add3b0` | WallOfDataPlayerKeyboardPacket::~WallOfDataPlayerKeyboardPa… | confirmed | Releases the array member through FUN_008c77e0 and then resets the vtable to BasePacket::vftable; called by the scalar deleting dtor 00add520 and by an EH unwi… |
| `00add4c0` | WallOfDataPlayerKeyboardPacket::WallOfDataPlayerKeyboardPac… 🌐 | confirmed | Base constructor call 00adcd60 with event type 2, then installs WallOfDataPlayerKeyboardPacket::vftable and default-constructs its SoeUtil::Array<unsigned __in… |
| `00add520` | WallOfDataPlayerKeyboardPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: calls the array-releasing destructor 00add3b0 and then frees the object when the flag argument has bit 0 set; it is vtable slot 0 … |

**WallOfDataUIEventPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00add540` | WallOfDataUIEventPacket::~WallOfDataUIEventPacket | confirmed | Releases the three IStringFixed<char,64> members through FUN_004022b0 and then resets the vtable to BasePacket::vftable. |
| `00add700` | WallOfDataUIEventPacket::WallOfDataUIEventPacket 🌐 | confirmed | Base constructor call 00adcd60 with event type 4 (opcode 194/4, the catalogue's WallOfDataUIEventPacket), installs WallOfDataUIEventPacket::vftable and default… |
| `00add780` | WallOfDataUIEventPacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: calls the string-releasing destructor 00add540 and then frees the object when the flag argument has bit 0 set; it is vtable slot 0… |

**WallOfDataWalletBalancePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adcf80` | WallOfDataWalletBalancePacket::WallOfDataWalletBalancePacket 🌐 | confirmed | Base constructor call 00adcd60 with event type 6, then installs WallOfDataWalletBalancePacket::vftable and zeroes the two dwords at +0xc and +0x10. |
| `00add1c0` | WallOfDataWalletBalancePacket::ScalarDeletingDtor | confirmed | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtab… |
| `00addc90` | WallOfDataWalletBalancePacket_Serialize 🌐 | likely | Writes the wall-of-data header with 00addaa0, then appends the two 4-byte fields at +0xc and +0x10. |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00addeb0` | WallOfData_SendUIEvent 🌐 | confirmed | Serializes a UI-event packet: writes the header with 00addaa0, serialises the three strings at +0xc, +0x60 and +0xb4 with FUN_00911c30, wraps the bytes in a tu… |
| `00ade100` | WallOfData_SendMembershipPurchase 🌐 | confirmed | Serializes a membership-purchase packet with 00addbf0, wraps the bytes in a tunnel packet (FUN_0089ae70) and sends them through FUN_00926360. |
| `00ade210` | WallOfData_SendWalletBalance 🌐 | confirmed | Serializes a wallet-balance packet with 00addc90, wraps the bytes in a tunnel packet (FUN_0089ae70) and sends them through FUN_00926360. |
| `00ade3d0` | WallOfData_SendKeyboard 🌐 | confirmed | Serializes a keyboard packet: writes the header with 00addaa0, writes the element batch with 00addb40, wraps the bytes in a tunnel packet (FUN_0089ae70) and se… |
| `00addaa0` | WallOfData_WriteHeader 🌐 | likely | Appends the wall-of-data header to the output buffer: copies two bytes of the packet's opcode field at +4 and one byte of the event type at +8 through the boun… |
| `00addb40` | WallOfDataKeyboardPacket_WriteEntries 🌐 | likely | Writes the element count (a 4-byte prefix via FUN_00911420) and then serialises each 8-byte element of the array at +4 with FUN_008fb810. |

### `net.ssl`

**SslConnectionFilterImpl** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f91800` | SslConnectionFilterImpl::HandleError | confirmed | Takes an SSL library result code from 011923f0 and, for codes other than 0, 2, 3 and 6, formats its text through 011affe0 and logs 'SslConnectionFilterImpl::Ha… |
| `00f918f0` | SslConnectionFilterImpl::ReportSSLErrors | confirmed | Drains up to four pending error codes from the SSL error queue (011b0180), formats each through 011affe0 and logs 'SslConnectionFilterImpl::ReportSSLErrors, er… |
| `00f919e0` | SslConnectionFilterImpl::Write | confirmed | Writes up to the given length through 0118fb60 (SSL_write). A result below 1 is reported through 00f91800 with the text 'SSL_write failed' and returns -1 when … |
| `00f91b00` | SslConnectionFilterImpl::DoSSL | confirmed | Pumps the SSL state machine: drains the outgoing memory BIO (+0x2028) in 0x2000-byte chunks into the framed transport send 010e92c0 (TcpSendFramedMessage label… |
| `00f921b0` | SslConnectionFilterImpl::SendAll | likely | Loops over the input in chunks of at most 0x2000 bytes: writes each chunk through BioWrite (00f91a60), then pumps DoSSL (00f91b00); on a BioWrite error it sets… |

**SslLibrary** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8f180` | SslLibrary::GetFuncByName | confirmed | Calls GetProcAddress with the HMODULE in EAX and the procedure name in EDI, returns the FARPROC. On failure it logs at level 2 through 00f99fd0 with the text '… |
| `00f8fd40` | SslLibrary::CalcHash | confirmed | Hashes a buffer with the CryptoAPI function table in ESI: CryptAcquireContext (slot +0xc, provider type 1), CryptCreateHash with CALG 0x8003 (MD5) (slot +0x10)… |
| `00f900e0` | SslLibrary::MakeDirectory | confirmed | Appends the argument path to a global buffer at DAT_01cb40c0, adds a trailing slash, then walks the buffer and, for each directory component, calls GetFileAttr… |
| `00f90d90` | SslLibrary::ConvertStoreToDirectory | confirmed | Creates a UtilsReportHandlerAdapter on the stack and loads the Crypt32 wrapper (00f90270), makes the output directory (00f900e0), clears its regular files (00f… |
| `00f90eb0` | SslLibrary::ConvertAllStores | confirmed | Loads the Crypt32 wrapper, makes and clears the output directory, then converts each store name in the null-terminated table at 0x01b97c2c through 00f90830, st… |
| `00f905c0` | SslLibrary::ClearDirectoryFiles | likely | Appends a wildcard to the directory argument and iterates FindFirstFileA/FindNextFileA; every entry without the directory attribute 0x10 is removed with Delete… |

**SslLibrary::AdvApiDllWrapper** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8f960` | SslLibrary::AdvApiDllWrapper::Init | confirmed | Loads AdvApi32.dll and resolves AllocateAndInitializeSid, CryptAcquireContextA, CryptCreateHash, CryptDestroyHash, CryptGetHashParam, CryptHashData, CryptRelea… |

**SslLibrary::CryptoDllWrapper** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f90270` | SslLibrary::CryptoDllWrapper::Init | confirmed | Allocates a 0x34-byte module record (operator new through thunk_FUN_00d833a6, zeroed), requires AdvApi init 00f8f960 to succeed, loads Crypt32.dll with LoadLib… |
| `00f90040` | SslLibrary::CryptoDllWrapper::Release | likely | Releases the Crypt32 wrapper: frees the module-holder block at *this with its library, calls FreeLibrary on the module stored at this+8 (clearing it), and free… |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f91670` | SslConnectionHolderAssign | likely | Releases the holder's previous connection as in 00f91600 (clear +0x90, decrement refcount, release at zero), then stores arg2 as the held connection, copies th… |
| `00f91720` | SslConnectionHolderStatus | likely | Returns 1 when the holder is empty; otherwise returns the mapped status of the held connection through 00f91520. |
| `00f91dd0` | SslConnectionStatusText | likely | Returns entry arg of the pointer table at 0x01b97c78 (the first entry is cErrNone). |
| `00f91de0` | SslConnectionHolderDeleteObject | likely | If the holder's object pointer at this is nonzero, calls the object's vtable slot 0 with flag 1 (scalar deleting destructor); the holder pointer is not cleared… |
| `00f91df0` | SslConnectionHolderSetWord18 | likely | Stores arg2 as the dword at +0x18 of the held connection object. |
| `00f91e10` | SslConnectionHolderGetStatusWord | likely | Returns the dword at +0x204c of the object held by the pointer at this (the status word written by SetStatus and DoConnect). |

### `net.tcp`

**ClientServerCore::BaseTcpApi** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d8010` | ClientServerCore::BaseTcpApi::EstablishConnection | confirmed | Stamps the 64-bit tick from 007027f0 at this+0x88/+0x8c and stores this at +0xc, allocates 0xf8 bytes through 00d833a6 and constructs the TCP driver with 010eb… |
| `010d8170` | ClientServerCore::BaseTcpApi::DisconnectRequest | confirmed | Virtual, slot [4] of BaseTcpApi. Logs 'BaseTcpApi disconnect request' through 00704dd0 using the category at +0x1b0, clears the byte at +0x1a8, and when the dr… |
| `010d8260` | ClientServerCore::BaseTcpApi::WaitForConnect | confirmed | Virtual, slot [9] of BaseTcpApi and the ExternalLoginTcpApi sibling. Logs 'BaseTcpApi waiting for connect, timeout=%d' through 00704dd0 with the category at +0… |
| `010d8350` | ClientServerCore::BaseTcpApi::AutoReconnectTick | confirmed | Virtual, slot [5] of BaseTcpApi. When the driver at +0x78 is present it calls 010eb0f0 and returns. Otherwise, when the byte at +0x1a8 is set and the count at … |
| `010d83e0` | ClientServerCore::BaseTcpApi::OnDisconnected | confirmed | Virtual at secondary slot [3] (+0x4) of BaseTcpApi, with the owner at this-4 and the driver at this+0x0 (BaseTcpApi+8 as read through this+4). Fills a 64-byte … |
| `010d8520` | ClientServerCore::BaseTcpApi::OnConnectionCompleted | confirmed | Virtual at the secondary slot [2] of BaseTcpApi. Logs 'BaseTcpApi connection completed, address=%s' through 00704dd0 using the category at +0x1ac and the addre… |
| `010d8560` | ClientServerCore::BaseTcpApi::ConnectStoredAddress | confirmed | Virtual, slot [3] of BaseTcpApi. Stores the argument byte at this+0x1a8 and calls 010d8010 (BaseTcpApi EstablishConnection) with the value at +0x94 and the val… |
| `010d8580` | ClientServerCore::BaseTcpApi::~BaseTcpApi | confirmed | Destructor body. Stores ClientServerCore::BaseTcpApi::vftable at this and at this+4, logs 'BaseTcpApi destructed' through 00704dd0 with the category at +0x1b0,… |
| `010d8780` | ClientServerCore::BaseTcpApi::ConnectRequest | confirmed | Virtual, slot [2] of BaseTcpApi. Logs 'BaseTcpApi connect request, address=%s timeout=%d autoReconnect=%d' through 00704dd0 with the category at +0x1b0, copies… |
| `010d87d0` | ClientServerCore::BaseTcpApi::QueryConnectionInfo | confirmed | Virtual, slot [12] of BaseTcpApi and the ExternalLoginTcpApi sibling. When the driver at BaseTcpApi+8 is present it reads the text through 010e78d0 from the dr… |
| `010d8810` | ClientServerCore::BaseTcpApi::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the BaseTcpApi destructor body 010d8580 on this, then calls _free on the object when bit 0 of the argument is set; returns th… |
| `010d8ab0` | ClientServerCore::BaseTcpApi::OnRoutePacket | confirmed | Virtual at the secondary slot +0x4[1] of BaseTcpApi. Calls the hook object at this+0x1fc through its vtable +4 with the length argument, takes a 64-bit tick, c… |
| `010d81a0` | ClientServerCore::BaseTcpApi::ServiceUntilTimeout | likely | Virtual, slot [8] of BaseTcpApi and the ExternalLoginTcpApi sibling. Reads the 64-bit tick (007027f0) and loops while the probe at slot +0x18 is true: each pas… |
| `010d8830` | ClientServerCore::BaseTcpApi::PublishTcpStats | likely | Thiscall with a sink pointer and a prefix argument. Calls 010eb280 first, then formats metric names with 00402fd0 from the prefix: '%s/Tcp/IncomingByteRate', '… |

**TcpLibrary::AsyncAddressResult** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010eb8e0` | TcpLibrary::AsyncAddressResult::AsyncAddressResult | confirmed | Calls the SoeUtil::Job constructor 0083f1e0 on this, then stores the AsyncAddressResult vptr. The string at +0x58 gets the StringFixed<char,256> vptr and the e… |
| `010eb9e0` | TcpLibrary::AsyncAddressResult::ScalarDeletingDtor | confirmed | Virtual slot 0 of AsyncAddressResult. Calls the DtorBody 010eb980 on this, then calls _free on the object when bit 0 of the flag argument is set. Returns this … |
| `010eb810` | TcpLibrary::AsyncAddressResult::StepResolve | likely | Calls vtable slot 10 (+0x28) of the object at this+0x16c with (this+0x174 as the address record, dword this+0x5c, dword this+0x170). Stores at this+0x17c a byt… |

**TcpLibrary::TcpConnection** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e8e70` | TcpLibrary::TcpConnection::TcpConnection | likely | Constructor body: sets the SoeUtil::RefCounted vtable, then the TcpLibrary::TcpConnection vtable; zeroes the dwords at +0x0C..+0x60 and +0x78 and +0x90; embeds… |
| `010e9030` | DtorBody | likely | Destructor body of TcpLibrary::TcpConnection: sets the TcpConnection vtable; when the object at +0x20 is non-null calls 010e7d20(this,0,5) (close with reason 5… |
| `010e91d0` | ScalarDeletingDtor | likely | Vtable slot 0 of TcpLibrary::TcpConnection: calls the destructor body 010e9030 on this and calls _free(this) when bit 0 of param_2 is set; returns this. |

**TcpLibrary::TcpConnectionDriver** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e96b0` | DtorBody | likely | Stores the TcpLibrary::TcpConnectionDriver vtable at offset 0 and returns. Does not free memory. |
| `010e96c0` | ScalarDeletingDtor | likely | Vtable slot 0 of TcpLibrary::TcpConnectionDriver: stores the TcpConnectionDriver vtable at offset 0 and calls _free(this) when bit 0 of param_2 is set; returns… |

**TcpLibrary::TcpConnectionPlatformDriver** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e9a40` | TcpLibrary::TcpConnectionPlatformDriver::SetDword18 | confirmed | Stores its single stack argument (a dword) into this+0x18 and returns with ret 4 (mov eax,[esp+4]; mov [ecx+0x18],eax). |
| `010e9a50` | TcpLibrary::TcpConnectionPlatformDriver::GetLocalSockAddr | confirmed | Calls getsockname on the socket at this+0x14 with a 16-byte sockaddr. On success it writes sa_data[2..5] (the IPv4 address bytes) to param_2[0..3] and ntohs(sa… |
| `010e9af0` | TcpLibrary::TcpConnectionPlatformDriver::GetLocalPort | confirmed | Calls getsockname on the socket at this+0x14. On -1 it returns 0 (xor eax,eax). Otherwise it returns ntohs(sa_data[0..1]), the local port in host order, zero-e… |
| `010e9b60` | TcpLibrary::TcpConnectionPlatformDriver::GetPeerSockAddr | confirmed | Calls getpeername on the socket at this+0x14 with a 16-byte sockaddr. On success it writes sa_data[2..5] (the IPv4 address bytes) to param_2[0..3] and ntohs(sa… |
| `010e9c00` | TcpLibrary::TcpConnectionPlatformDriver::SendBytes | confirmed | Calls send(socket at this+0x14, buffer, length, 0) and returns the send result, or 0 when it is negative (-1). ret 8. |
| `010ea0d0` | TcpLibrary::TcpConnectionPlatformDriver::DtorBody | confirmed | Stores the TcpConnectionPlatformDriver vptr at +0. If the counted pointer at +8 is nonnull, locked-decrements its count at +4, calls its vtable slot 1 at zero,… |
| `010ea8b0` | TcpLibrary::TcpConnectionPlatformDriver::ScalarDeletingDtor | confirmed | Virtual slot 0 of the connection platform driver. Calls the DtorBody (010ea0d0) on this, then if bit 0 of the flag argument is set calls _free on the object. R… |

**TcpLibrary::TcpDriver** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010eb7f0` | TcpLibrary::TcpDriver::TcpDriver | confirmed | Stores the TcpDriver vptr (018895a0) at this and zeroes the dword at +4. Returns nothing (fastcall: this in ECX, no stack arguments). |
| `010eb850` | TcpLibrary::TcpDriver::DtorBody | confirmed | Stores the TcpDriver vptr at this. When the pointer at +4 is nonnull it calls 00fc60d0, then calls 00fc6e40 and _free on the object at +4, then zeroes +4. Retu… |
| `010eb890` | TcpLibrary::TcpDriver::ScalarDeletingDtor | confirmed | Virtual slot 0 of TcpDriver. Stores the TcpDriver vptr, releases the queue at +4 as DtorBody 010eb850 does (00fc60d0, 00fc6e40, _free, then zeroes +4), and cal… |
| `010eb800` | TcpLibrary::TcpDriver::DrainQueue | likely | If the pointer at this+4 (a queue object owned by the driver, freed by DtorBody 010eb850) is nonnull, calls 00fc6820 on it with a budget of -1 (no limit). Retu… |
| `010eba00` | TcpLibrary::TcpDriver::StartAddressResolve | likely | Vtable slot 12 of TcpDriver (inherited by TcpPlatformDriver). On first use it allocates 0x328 bytes into this+4, constructs the queue state (00fc6db0) and star… |

**TcpLibrary::TcpManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010eb4f0` | TcpLibrary::TcpManager::DtorBody | confirmed | Stores the TcpManager vptr at this. When the dword at +0x14 is nonnegative it calls driver slot 4 (+0x10) on the driver at +0x8c. When +0x64 is zero (the manag… |
| `010eb5c0` | TcpLibrary::TcpManager::ScalarDeletingDtor | confirmed | Virtual slot 0 of TcpManager. Calls the DtorBody 010eb4f0 on this, then calls _free on the object when bit 0 of the flag argument is set. Returns this (ret 4). |
| `010eb660` | TcpLibrary::TcpManager::TcpManager | confirmed | Sets the SoeUtil::RefCounted vptr, count 1, then the TcpManager vptr. Calls 010eb5e0 with param_2, zeroes the fields at +0x74..+0x88 and the state at +0x90, st… |
| `010eb0f0` | TcpLibrary::TcpManager::PumpWithBudget | likely | Adds a reference to this (+4), increments the 64-bit counter at +0x98 (carry into +0x9c) and calls vtable slot 2 (+8) of the driver at +0x8c (AcceptConnections… |
| `010eb280` | TcpLibrary::TcpManager::SnapshotStats | likely | Copies the 0x14-dword counter block at this+0x98 to param_2[0..0x13]. Sets param_2[0x12] from +0x7c. Stores the milliseconds elapsed since the 64-bit tick at +… |

**TcpLibrary::TcpManagerHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d7ec0` | TcpLibrary::TcpManagerHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores TcpLibrary::TcpManagerHandler::vftable at this, calls _free on the object when bit 0 of the argument is set, and returns thi… |
| `010d7eb0` | TcpLibrary::TcpManagerHandler::~TcpManagerHandler | likely | Non-deleting destructor body. Stores TcpLibrary::TcpManagerHandler::vftable at this and returns; it frees nothing. |

**TcpLibrary::TcpPlatformDriver** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e9900` | TcpLibrary::TcpPlatformDriver::GetLocalHostIPv4 | confirmed | Calls gethostname (1024-byte buffer) and gethostbyname on it, then walks h_addr_list. The first address is always taken; later addresses replace it only when t… |
| `010e9e80` | TcpLibrary::TcpPlatformDriver::OpenListenSocket | confirmed | Creates a socket with socket(AF_INET=2, SOCK_STREAM=1, 0) and stores it at param_1[2]+4. If FUN_010e96e0 applies the options at +4/+8/+0xc successfully, it bui… |
| `010ea620` | TcpLibrary::TcpPlatformDriver::AcceptConnections | confirmed | If the listening socket at config+4 (config = this+8) is not -1, it calls accept in a loop until accept returns INVALID_SOCKET. For each accepted socket it all… |
| `010eaa20` | TcpLibrary::TcpPlatformDriver::SetEndpointConfig | confirmed | Stores p2 at config+0, p4 at config+8 and p3 at config+0xc, where config is the 100-byte object at this+8. It then assigns the string p5 into config+0x10 throu… |
| `010eab70` | TcpLibrary::TcpPlatformDriver::TcpPlatformDriver | confirmed | Calls 010eb7f0 first, then stores the TcpPlatformDriver vptr at this. It allocates 100 bytes (00d833a6) for the endpoint config and initializes it: the string … |
| `010eaea0` | TcpLibrary::TcpPlatformDriver::ScalarDeletingDtor | confirmed | Virtual slot 0 of the platform driver. Calls the DtorBody 010eac40 on this, then frees the object with _free when bit 0 of the flag argument is set. Returns th… |
| `010e97b0` | CloseSocket | likely | Vtable slot 4 of TcpLibrary::TcpPlatformDriver: reads the SOCKET at [[this+8]+4]; when it is not INVALID_SOCKET (0xFFFFFFFF) calls closesocket on it and stores… |
| `010e97e0` | GetLocalAddress | likely | Vtable slot 5 of TcpLibrary::TcpPlatformDriver: calls getsockname on the socket at [[this+8]+4] into a sockaddr; on failure (-1) returns; otherwise writes the … |

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010eae40` | FormatIPv4AddrPort | confirmed | Returns 0 when the first byte of the record is zero. Otherwise, when param_2 is nonzero, it formats the four address bytes as %d.%d.%d.%d into the string at pa… |
| `010e96e0` | SetSocketOptions | likely | Applies socket options to the SOCKET passed as its argument and returns true only when all calls succeed: ioctlsocket with FIONBIO (0x8004667E, passed as -0x7F… |
| `010e9d70` | ParseInt16Text | likely | Skips leading whitespace (wctrans space tests), then reads an optional sign and decimal digits. A '-' followed by digits accumulates a negative value; '+' or n… |
| `010ea540` | ParseDelimitedCharField | likely | Zeroes a 2048-byte stack buffer, calls ParseCsvField (006fba40) with (src=param_1, buffer, size 0x800, delimiter=param_3, trim=1) and passes the buffer and par… |
| `010ea8d0` | ParseIPv4AddrPortText | likely | With a null text pointer (param_2==0) it zeroes two dwords at the record. Otherwise it parses 'a.b.c.d:port' with ParseDelimitedCharField (delimiters 0x2e for … |
| `010eb400` | ListLinkAfterTail | likely | Takes a reference on param_2 (+4). Stores the old tail (this+0x78) at param_2+0xc. If the list was not empty it links the old tail's +0x10 to param_2, else it … |
| `010eb440` | ListUnlinkNode | likely | Unlinks param_2 from the list headed at this+0x74 (tail at +0x78, count at +0x7c), using the node's prev at +0xc and next at +0x10 and clearing both. Decrement… |
| `010eb4a0` | ListLinkAfterTail2 | likely | Takes a reference on param_2 (+4) and stores the old tail of the second list (this+0x84) at param_2+0x14. If the list was not empty it links the old tail's +0x… |

### `net.udp`

**ClientServerCore::BaseUdpManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d6a10` | ClientServerCore::BaseUdpManager::~BaseUdpManager | confirmed | Destructor body. Stores ClientServerCore::BaseUdpManager::vftable at this, releases the IStringFixed<char,32> member at +0x274 (count +0x280) through its destr… |
| `010d6e70` | ClientServerCore::BaseUdpManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the BaseUdpManager destructor body 010d6a10 on this, then calls _free on the object when bit 0 of the argument is set, and re… |
| `010d74b0` | ClientServerCore::BaseUdpManager::BaseUdpManager | confirmed | Thiscall constructor. Runs the base constructor 010d7a70, installs ClientServerCore::BaseUdpManager::vftable, calls 010d5290 with profile 2 or 4 (chosen from t… |
| `010d6440` | ClientServerCore::BaseUdpManager::ServiceStart | likely | Increments the reference count at this+0xc0. On the first reference it stores this at +0xc8, allocates 0x578 bytes through operator new (00d833a6) and, when th… |
| `010d6560` | ClientServerCore::BaseUdpManager::ServiceRelease | likely | Decrements the reference count at this+0xc0. When it reaches zero and the driver at +0x270 is present it: calls 010de160 when the byte at +0xc4 is set, calls 0… |

**ClientServerCore::UdpCompressionHandler** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d7820` | ClientServerCore::UdpCompressionHandler::~UdpCompressionHan… | confirmed | Non-deleting destructor body. Stores ClientServerCore::UdpCompressionHandler::vftable at this, calls 00fc45a0 to release the compression state, then restores U… |
| `010d7ac0` | ClientServerCore::UdpCompressionHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores ClientServerCore::UdpCompressionHandler::vftable at this, calls 00fc45a0, restores UdpLibrary::UdpManagerHandler::vftable an… |
| `010d7840` | ClientServerCore::UdpCompressionHandler::CompressOrStoreFra… | likely | Thiscall. Takes a payload pointer and length and an output buffer. When the length is above 23 it compresses the payload after a leading flag byte with 010ebc1… |
| `010d7920` | ClientServerCore::UdpCompressionHandler::UnframeOrDecompress | likely | Virtual, slot [3] of UdpCompressionHandler (and the sibling API vtables that share it). When the frame flag byte is not 1 it copies length-1 raw bytes after th… |
| `010d7a70` | ClientServerCore::UdpCompressionHandler::UdpCompressionHand… | likely | Thiscall constructor body. Stores ClientServerCore::UdpCompressionHandler::vftable at this, calls 00fc4580 with 4000, runs the 00fc45f0 / 00fc4610 pair around … |

**UdpLibrary::FixedLogicalPacket<1024>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010db080` | FixedLogicalPacket1024Ctor | confirmed | Calls base constructor 010e58d0 on this, stores UdpLibrary::FixedLogicalPacket<1024>::vftable, stores param_3 at this+0x414, and copies param_3 bytes from para… |
| `010db510` | DeleteVirtualSlot4 | confirmed | Calls 010e58f0 (stores the UdpRefCount vtable pointer), then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |
| `010db0c0` | GetPayloadPtr | likely | Returns this+0x14. Virtual slot 5 of FixedLogicalPacket<1024>. |

**UdpLibrary::FixedLogicalPacket<128>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010daf00` | FixedLogicalPacket128Ctor | confirmed | Calls base constructor 010e58d0 (LogicalPacket: refcount 1, flag 0, LogicalPacket vtable) on this, stores UdpLibrary::FixedLogicalPacket<128>::vftable, stores … |
| `010db4b0` | DeleteVirtualSlot4 | confirmed | Calls 010e58f0 (stores the UdpRefCount vtable pointer), then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |
| `010daf40` | GetPayloadPtr | likely | Returns this+0x14 (lea eax,[ecx+0x14]; ret). Virtual slot 5 of FixedLogicalPacket<128>. |

**UdpLibrary::FixedLogicalPacket<256>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010daf80` | FixedLogicalPacket256Ctor | confirmed | Calls base constructor 010e58d0 on this, stores UdpLibrary::FixedLogicalPacket<256>::vftable, stores param_3 at this+0x114, and copies param_3 bytes from param… |
| `010db4d0` | DeleteVirtualSlot4 | confirmed | Calls 010e58f0 (stores the UdpRefCount vtable pointer), then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |
| `010dafc0` | GetPayloadPtr | likely | Returns this+0x14. Virtual slot 5 of FixedLogicalPacket<256>. |

**UdpLibrary::FixedLogicalPacket<512>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010db000` | FixedLogicalPacket512Ctor | confirmed | Calls base constructor 010e58d0 on this, stores UdpLibrary::FixedLogicalPacket<512>::vftable, stores param_3 at this+0x214, and copies param_3 bytes from param… |
| `010db4f0` | DeleteVirtualSlot4 | confirmed | Calls 010e58f0 (stores the UdpRefCount vtable pointer), then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |
| `010db040` | GetPayloadPtr | likely | Returns this+0x14. Virtual slot 5 of FixedLogicalPacket<512>. |

**UdpLibrary::GroupLogicalPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e59e0` | UdpLibrary::GroupLogicalPacket::~GroupLogicalPacket | likely | Stores the GroupLogicalPacket vtable, calls 010dadd0 cdecl with (dword at this+0x14, 0, 1), then stores the UdpRefCount vtable at offset 0. Does not free this. |
| `010e5c40` | UdpLibrary::GroupLogicalPacket::ScalarDeletingDtor | likely | Calls 010e59e0(this); if bit 0 of the flag argument is set calls _free(this); returns this. |

**UdpLibrary::LogicalPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e58d0` | UdpLibrary::LogicalPacket::LogicalPacket | likely | Sets dword +4 to 1, byte at +8 to 0, dwords at +0xc and +0x10 to 0, and stores the UdpLibrary::LogicalPacket vtable at +0. |
| `010e5be0` | UdpLibrary::LogicalPacket::ScalarDeletingDtor | likely | Stores the UdpRefCount vtable at offset 0; if bit 0 of the flag argument is set, calls _free(this); returns this. |

**UdpLibrary::PooledLogicalPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e57e0` | UdpLibrary::PooledLogicalPacket::AddRef | likely | Increments the dword at this+4 without any lock (bytes 83 41 04 01 c3: add dword [ecx+4],1; ret). Returns nothing. |
| `010e5aa0` | UdpLibrary::PooledLogicalPacket::PooledLogicalPacket | likely | Sets dword +4 to 1, byte +8 to 0, dwords +0xc and +0x10 to 0, stores the PooledLogicalPacket vtable, zeroes +0x24..+0x30, stores param_3 at +0x1c, allocates pa… |
| `010e5ba0` | UdpLibrary::PooledLogicalPacket::Release | likely | Calls this->vtable[3]() (call [vtable+0xc]); if it returns 1 and the pointer at +0x20 is non-null, calls 010dcf70(this) and returns. Otherwise decrements +4 an… |
| `010e5c60` | UdpLibrary::PooledLogicalPacket::ScalarDeletingDtor | likely | Calls 010e5b30(this); if bit 0 of the flag argument is set calls _free(this); returns this. |

**UdpLibrary::SimpleLogicalPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e5900` | UdpLibrary::SimpleLogicalPacket::SimpleLogicalPacket | likely | Sets dword +4 to 1, byte at +8 to 0, dwords +0xc and +0x10 to 0, stores the SimpleLogicalPacket vtable, stores param_3 at +0x18, allocates param_3 bytes via th… |

**UdpLibrary::UdpConnection** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e2c60` | UdpLibrary::UdpConnection::~UdpConnection | confirmed | Destructor body: stores the UdpConnection vtable, under the lock 010da6d0 walks the four pointers at +0x174..+0x180 (each: 010e6740, then _free), frees the blo… |
| `010e2e90` | UdpLibrary::UdpConnection::ScalarDeletingDtor | confirmed | Virtual, UdpConnection vtable slot 4. Calls the UdpConnection destructor body 010e2c60 on this, then calls _free on this when bit 0 of the flag is set, and ret… |
| `010e2a10` | UdpLibrary::UdpConnection::Init | likely | Thiscall(owner, arg2, arg3, arg4). Stores the owner at this+0x94, copies arg3 through 010da870, stores arg4 at this+0x6c, copies owner fields into this (+0x74 … |
| `010e34c0` | UdpLibrary::UdpConnection::CloseOrActivate | likely | Thiscall(state, reason). Records the reason at this+0x168 when it is still zero. When the state argument is below 1 it closes the connection: sets the mode at … |
| `010e3dd0` | UdpLibrary::UdpConnection::DispatchRecord | likely | Thiscall(record, length). Requires an owner at this+0x94. Dispatches one transport record by the type byte at record[1] (record[0] must be zero, length at leas… |
| `010e5220` | UdpLibrary::UdpConnection::UdpConnection | likely | Constructor body: stores UdpConnection vtable at +0, sets +0x10 to -1, zeroes +0x78..+0x84, calls 010da690 twice, vtable[0](), 010da6d0, 010da860(&param_3), 01… |
| `010e5390` | UdpLibrary::UdpConnection::UdpConnection | likely | Second constructor body: stores UdpConnection vtable, sets +0x10 to -1, zeroes +0x78..+0x84, calls 010da690 twice, vtable[0](), 010da6d0, 010da860(param_3+4), … |

**UdpLibrary::UdpConnectionHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d50e0` | UdpLibrary::UdpConnectionHandler::~UdpConnectionHandler | confirmed | Non-deleting destructor body. Stores UdpLibrary::UdpConnectionHandler::vftable at this and returns; it frees nothing. |
| `010d50f0` | UdpLibrary::UdpConnectionHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores UdpLibrary::UdpConnectionHandler::vftable at this, calls _free (00d82463) on this when bit 0 of the argument is set, and ret… |

**UdpLibrary::UdpDriver** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da0a0` | UdpLibrary::UdpDriver::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores UdpLibrary::UdpDriver::vftable at this, calls _free on the object when bit 0 of the argument is set, and returns this. |

**UdpLibrary::UdpGuardedRefCount** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da7b0` | UdpLibrary::UdpGuardedRefCount::UdpGuardedRefCount | confirmed | Constructor body. Sets the count at +4 to 1, the flag byte at +8 to 0, installs UdpLibrary::UdpGuardedRefCount::vftable, allocates a 0x18-byte critical section… |
| `010da820` | UdpLibrary::UdpGuardedRefCount::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the destructor body 010da720 on this, then calls _free on the object when bit 0 of the argument is set, and returns this. |
| `010da720` | UdpLibrary::UdpGuardedRefCount::~UdpGuardedRefCount | likely | Destructor body. Calls DeleteCriticalSection on the pointer at this+0xc, frees it through _free, and restores UdpLibrary::UdpRefCount::vftable at this. |
| `010e5650` | UdpLibrary::UdpGuardedRefCount::AddRef | likely | Calls 010da6d0 with ECX = this+0x0C, increments the dword at this+4, then tail-jumps to 010da6e0 with the same ECX. Returns nothing. |
| `010e5670` | UdpLibrary::UdpGuardedRefCount::Release | likely | Calls 010da6d0 with ECX = this+0x0C, decrements the dword at this+4, records whether it reached zero, calls 010da6e0, and if it did calls this->vtable[4](1). R… |

**UdpLibrary::UdpLinkedList<UdpLibrary::LogicalPacket>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e5e30` | UdpLinkedList_LogicalPacket::~UdpLinkedList | likely | Stores the UdpLinkedList<LogicalPacket> vtable at offset 0 and returns. Does not free memory. |
| `010e5ea0` | UdpLinkedList_LogicalPacket::ScalarDeletingDtor | likely | Stores the UdpLinkedList<LogicalPacket> vtable at offset 0; if bit 0 of the flag argument is set calls _free(this); returns this. |

**UdpLibrary::UdpLinkedList<UdpLibrary::PooledLogicalPacket>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010dc3e0` | DtorLinkedListPooledLogicalPacket | confirmed | Stores the UdpLinkedList<PooledLogicalPacket> vtable into the object and returns. No free; EH unwind funclets call it. |
| `010dc9e0` | DeleteVirtualSlot0 | confirmed | Sets the PooledLogicalPacket list vtable, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |

**UdpLibrary::UdpLinkedList<UdpLibrary::UdpConnection>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010dc050` | DtorLinkedListConnection | confirmed | Stores the UdpLinkedList<UdpConnection> vtable into the object and returns. It has no free and no other work; EH unwind funclets call it. |
| `010dc980` | DeleteVirtualSlot0 | confirmed | Sets the UdpLinkedList<UdpConnection> vtable, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |

**UdpLibrary::UdpLinkedList<UdpLibrary::UdpManager::CallbackEvent>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010dc4d0` | DtorLinkedListCallbackEvent | confirmed | Stores the UdpLinkedList<UdpManager::CallbackEvent> vtable into the object and returns. No free; EH unwind funclets call it. |
| `010dca00` | DeleteVirtualSlot0 | confirmed | Sets the CallbackEvent list vtable, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |

**UdpLibrary::UdpLinkedList<UdpLibrary::UdpManager::ExpectIncomingEntry>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010dc200` | DtorLinkedListExpectIncomingEntry | confirmed | Stores the UdpLinkedList<UdpManager::ExpectIncomingEntry> vtable into the object and returns. No free and no other work; EH unwind funclets call it. |
| `010dc9a0` | DeleteVirtualSlot0 | confirmed | Sets the ExpectIncomingEntry list vtable, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |

**UdpLibrary::UdpLinkedList<UdpLibrary::UdpManager::SimulateQueueEntry>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010dc2f0` | DtorLinkedListSimulateQueueEntry | confirmed | Stores the UdpLinkedList<UdpManager::SimulateQueueEntry> vtable into the object and returns. No free; EH unwind funclets call it. |
| `010dc9c0` | DeleteVirtualSlot0 | confirmed | Sets the SimulateQueueEntry list vtable, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |

**UdpLibrary::UdpManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010e0800` | UdpLibrary::UdpManager::UdpManager | confirmed | Thiscall constructor body. Runs the UdpGuardedRefCount constructor 010da7b0, stores the UdpManager vtable at +0, calls 010d5290 with 0, zeroes the counters and… |
| `010e0fd0` | UdpLibrary::UdpManager::~UdpManager | confirmed | Destructor body: stores the UdpManager vtable, drains the connection list at +0x1f8 (each node: vtable slot 0, 010e34c0(node, 0, 4), then vtable slot 1), calls… |
| `010e1450` | UdpLibrary::UdpManager::ScalarDeletingDtor | confirmed | Virtual, UdpManager vtable slot 4. Calls the UdpManager destructor body 010e0fd0 on this, then calls _free on this when bit 0 of the flag is set, and returns t… |

**UdpLibrary::UdpManagerHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d77f0` | UdpLibrary::UdpManagerHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores UdpLibrary::UdpManagerHandler::vftable at this and calls _free on the object when bit 0 of the argument is set; returns this. |
| `010db760` | CopyBytesReturnLen | likely | Copies param_4 bytes from param_3 to param_2 with memcpy and returns param_4 (four stack arguments, ret 0x10; the first is unused). |
| `010db7a0` | CopyBytesReturnLen | likely | Copies param_4 bytes from param_3 to param_2 with memcpy and returns param_4. Same body as 010db780 and 010db7c0. |

**UdpLibrary::UdpManagerThread** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010dbf00` | UdpManagerThread::Ctor | confirmed | Calls the base constructor 010dac40 (UdpPlatformThreadObject ctor) on this, stores UdpLibrary::UdpManagerThread::vftable, stores param_2 at this[5] and calls i… |
| `010de2e0` | UdpManagerThread::Dtor | confirmed | Stores the UdpManagerThread vtable, sets the stop byte at this+0x18 to 1, and loops while the function pointer in data PTR_FUN_01888ef4 returns nonzero, callin… |
| `010df200` | UdpManagerThread::DeleteVirtualSlot4 | confirmed | Calls the UdpManagerThread destructor 010de2e0, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |
| `010e0350` | UdpLibrary::UdpManagerThread::Run | confirmed | Virtual, UdpManagerThread vtable slot 7. Loops while the stop byte at this+0x18 is zero: calls 010dfbb0(500,1) on the owner at this+0x14, then calls vtable slo… |

**UdpLibrary::UdpPlatformDriver** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da120` | UdpLibrary::UdpPlatformDriver::~UdpPlatformDriver | confirmed | Destructor body. Stores UdpLibrary::UdpPlatformDriver::vftable at this, calls WSACleanup, frees the buffer at this+4 through _free, and restores UdpLibrary::Ud… |
| `010da2d0` | UdpLibrary::UdpPlatformDriver::CloseSocket | confirmed | Virtual, slot [2] of UdpPlatformDriver. When the socket handle is not INVALID_SOCKET it calls closesocket and stores -1 in the handle. |
| `010da2f0` | UdpLibrary::UdpPlatformDriver::ReceiveFrom | confirmed | Virtual, slot [3] of UdpPlatformDriver. Calls recvfrom into the caller's buffer with its length. On success it stores the sender's IPv4 address dword and the s… |
| `010da3c0` | UdpLibrary::UdpPlatformDriver::SendTo | confirmed | Virtual, slot [4] of UdpPlatformDriver. Builds an IPv4 sockaddr from the address dword at the 4th argument and the port (htons of the 5th argument) and calls s… |
| `010da430` | UdpLibrary::UdpPlatformDriver::SendToWithTtl5 | confirmed | Virtual, slot [5] of UdpPlatformDriver. Sets the socket's IP_TTL option (level 0, option 4) to 5, calls the vtable slot +0x10 (SendTo) with its four arguments,… |
| `010da490` | UdpLibrary::UdpPlatformDriver::GetLocalAddress | confirmed | Virtual, slot [6] of UdpPlatformDriver. Calls getsockname on the socket and stores the IPv4 address dword of the result at the output pointer. |
| `010da550` | UdpLibrary::UdpPlatformDriver::ResolveAddress | confirmed | Virtual, slot [8] of UdpPlatformDriver. Tries inet_addr on the string argument; when that fails it calls gethostbyname and takes the first address of the host … |

**UdpLibrary::UdpPlatformThreadObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da6f0` | UdpLibrary::UdpPlatformThreadObject::ThreadEntry | likely | Thread entry used as the start routine of 010da780. It sets the byte at (this->+0x10)+4 to 1, calls vtable slot +0x1c on the object, clears that byte again, ca… |

**UdpLibrary::UdpRefCount** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da050` | UdpLibrary::UdpRefCount::AddRef | confirmed | Virtual, slot [0] of UdpRefCount and of the sibling UDP vtables that share it. Increments the dword at this+4 by one and returns. |
| `010da060` | UdpLibrary::UdpRefCount::Release | confirmed | Virtual, slot [1] of UdpRefCount. Decrements the count at this+4; when it reaches zero it calls the vtable slot +0x10 (slot 4) with argument 1 on the object. |
| `010da9d0` | UdpRefCount::DeleteVirtualSlot4 | confirmed | Stores UdpLibrary::UdpRefCount::vftable into the object. When bit 0 of the stack byte argument is set it calls _free on the object. Returns this with ret 4. Sh… |
| `010da080` | UdpLibrary::UdpRefCount::GetRefCount | likely | Virtual, slot [3] of UdpRefCount and of the sibling UDP vtables. Returns the dword at this+4, the reference count. |
| `010e58f0` | UdpLibrary::UdpRefCount::~UdpRefCount | likely | Stores the UdpLibrary::UdpRefCount vtable at offset 0 and returns. Does not free memory. |

**(no class)** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da880` | FormatIPv4Dotted | confirmed | If param_3 is below 16 it writes an empty string to param_2. Otherwise it sprintf-formats the first four bytes of param_1 as %d.%d.%d.%d into param_2. Returns … |
| `010e2460` | DisconnectReasonName | confirmed | Returns the name string for DisconnectReason index param_1 from a 17-entry table at 01cb9870. On the first call it fills the table with the DisconnectReason li… |
| `010e2550` | FormatEndpointString | confirmed | Thiscall(out, size). Returns without writing when size is below 0x16. Otherwise copies the stored IPv4 address through 010e1870, reads the port at this+0x6c un… |
| `010e78d0` | DisconnectReasonName | confirmed | Switch that maps a disconnect-reason code (0x00..0x13) to its name string; any other value returns the string UnknownReason. Returns a char pointer; no side ef… |
| `010dbb30` | RollingSumAdvanceA | likely | Takes the lock object at this+0x290 (010da6d0 / 010da6e0), reads the 64-bit value at this+0x288 and divides it by 25 (__alldiv) to get a 25 ms bucket index. Co… |
| `010dbc10` | RollingSumAdvanceB | likely | Same algorithm as 010dbb30 on a second window: time at this+0x288 under the lock at this+0x290, divided by 25 ms buckets, last index at this+0x390, ring of 40 … |
| `010de0b0` | EnsureManagerThread | likely | Under the lock 010da6d0 / 010da6e0 it creates the object at this+0x1e8 when it is null: allocates 0x20 bytes through 00d833a6 and constructs it with 010dbf00 (… |
| `010df940` | ProcessEventQueueBudget | likely | Thiscall. Calls slot 0 of this at entry and slot 1 of this at exit. It reads the time through 010d51d0, then pops entries with 010ded60 (PopHeadAccount) until … |
| `010e2fc0` | FrameCrcSend | likely | Thiscall(payload, length, copyFlag). When the owner at this+0x94 is set and the mode at this+0x8c is 1 or 3: runs up to two transform stages (function pointers… |

## Source shards

[[S0217|S0217]] (1), [[S0218 00896b90-0089d930|S0218]] (84), [[S0219 0089d990-008a3c80|S0219]] (152), [[S0220 008a3ce0-008a8500|S0220]] (67), [[S0221 008a86b0-008b1d80|S0221]] (33), [[S0222 008b1e00-008b7730|S0222]] (131), [[S0223 008b7790-008bcf40|S0223]] (23), [[S0224 008bcf60-008c5290|S0224]] (14), [[S0226 008ca9e0-008d08b0|S0226]] (34), [[S0227 008d0920-008d5c60|S0227]] (26), [[S0228|S0228]] (2), [[S0229 008d92f0-008dea46|S0229]] (54), [[S0231|S0231]] (8), [[S0232 008e82e0-008ec1d0|S0232]] (37), [[S0235|S0235]] (106), [[S0236|S0236]] (35), [[S0237 008ff3d0-00904e70|S0237]] (6), [[S0238|S0238]] (38), [[S0239 0090a890-009105b0|S0239]] (22), [[S0240 009105e0-009161d0|S0240]] (6), [[S0241 009162f0-0091aa10|S0241]] (20), [[S0242 0091af50-009200d0|S0242]] (35), [[S0243 00920130-00925ec0|S0243]] (28), [[S0244 00925fe0-0092b110|S0244]] (32), [[S0247 00934740-00939200|S0247]] (10), [[S0248 009392c0-0093d740|S0248]] (5), [[S0249 0093daf0-00942c60|S0249]] (1), [[S0250 00942c80-00947180|S0250]] (4), [[S0251 009471a0-0094a910|S0251]] (3), [[S0253|S0253]] (6), [[S0264 00989810-0098eae0|S0264]] (3), [[S0265 0098ec00-00995660|S0265]] (1), [[S0270 009acf20-009b2140|S0270]] (14), [[S0271 009b2210-009b7170|S0271]] (42), [[S0272 009b7660-009bd590|S0272]] (52), [[S0273 009bd650-009c2c50|S0273]] (21), [[S0274 009c2cc0-009c8d80|S0274]] (18), [[S0275 009c8e90-009cdee0|S0275]] (5), [[S0276 009cdf80-009d3fe0|S0276]] (6), [[S0277|S0277]] (15), [[S0278 009d89c0-009e00c0|S0278]] (3), [[S0280 009e4570-009ea840|S0280]] (8), [[S0283 009f39b0-009f8370|S0283]] (10), [[S0284 009f8470-009feec0|S0284]] (9), [[S0285 009fef30-00a059e0|S0285]] (48), [[S0286 00a05a50-00a0c390|S0286]] (41), [[S0287 00a0c3a0-00a13140|S0287]] (39), [[S0288 00a13190-00a18a10|S0288]] (7), [[S0289 00a18a90-00a1dd30|S0289]] (32), [[S0290 00a1de60-00a243e0|S0290]] (22), [[S0291 00a244a0-00a29500|S0291]] (27), [[S0292 00a29620-00a2fe20|S0292]] (103), [[S0293 00a2fef0-00a35000|S0293]] (35), [[S0294 00a35150-00a39a80|S0294]] (6), [[S0295 00a39b00-00a3f350|S0295]] (29), [[S0296 00a3f560-00a45650|S0296]] (42), [[S0297|S0297]] (9), [[S0298 00a4b480-00a53380|S0298]] (51), [[S0299 00a533a0-00a59140|S0299]] (13), [[S0300 00a59220-00a5da20|S0300]] (39), [[S0303 00a674c0-00a6c3b0|S0303]] (17), [[S0304 00a6c630-00a70790|S0304]] (5), [[S0307 00a7b930-00a81420|S0307]] (12), [[S0308 00a81470-00a87b30|S0308]] (49), [[S0309 00a87ba0-00a8cdc0|S0309]] (21), [[S0310 00a8ce60-00a91b00|S0310]] (25), [[S0311 00a91bf0-00a97400|S0311]] (10), [[S0312 00a97450-00a9d360|S0312]] (92), [[S0313 00a9d3c0-00aa24e0|S0313]] (88), [[S0314 00aa2560-00aa90c0|S0314]] (104), [[S0315 00aa90e0-00aae6b0|S0315]] (31), [[S0316 00aae6e0-00ab2c20|S0316]] (14), [[S0317 00ab2c80-00ab7380|S0317]] (21), [[S0318 00ab73e0-00abcb30|S0318]] (20), [[S0319 00abcb90-00ac2280|S0319]] (1), [[S0320 00ac2a10-00ac7f40|S0320]] (34), [[S0321 00ac8340-00acd3f0|S0321]] (47), [[S0322 00acd6d0-00ad3250|S0322]] (16), [[S0323 00ad3270-00ad83b0|S0323]] (8), [[S0324 00ad8880-00adee10|S0324]] (92), [[S0325 00adee60-00ae4e70|S0325]] (13), [[S0334 00b0a370-00b10840|S0334]] (28), [[S0336 00b150e0-00b1b340|S0336]] (79), [[S0337 00b1b5a0-00b21f20|S0337]] (44), [[S0339 00b29890-00b2eaf0|S0339]] (58), [[S0343 00b3f370-00b44f00|S0343]] (27), [[S0344 00b44fa0-00b4b360|S0344]] (34), [[S0347 00b56090-00b5b410|S0347]] (45), [[S0348 00b5b470-00b5ff50|S0348]] (17), [[S0352 00b6cd80-00b71320|S0352]] (1), [[S0353 00b71490-00b77eb0|S0353]] (99), [[S0354|S0354]] (53), [[S0357 00b84e90-00b8a5f0|S0357]] (47), [[S0358 00b8a640-00b906a0|S0358]] (6), [[S0359 00b90750-00b94a30|S0359]] (10), [[S0360 00b94bf0-00b9a300|S0360]] (70), [[S0361 00b9a660-00ba0260|S0361]] (65), [[S0362 00ba0350-00ba4650|S0362]] (61), [[S0363 00ba48a0-00ba7460|S0363]] (2), [[S0364 00ba74d0-00bac070|S0364]] (21), [[S0365 00bac0d0-00bb1110|S0365]] (15), [[S0366 00bb11b0-00bb7710|S0366]] (15), [[S0367 00bb7790-00bbdbc0|S0367]] (2), [[S0371|S0371]] (10), [[S0372 00bd37e0-00bd5d40|S0372]] (11), [[S0374 00bd9880-00be0090|S0374]] (44), [[S0375 00be0120-00be58f0|S0375]] (9), [[S0376 00be5b30-00bea020|S0376]] (12), [[S0378 00becaa0-00bf1530|S0378]] (7), [[S0380 00bf5ff0-00bf9410|S0380]] (5), [[S0382 00bfcfe0-00c056a0|S0382]] (6), [[S0383 00c05780-00c0db00|S0383]] (54), [[S0384 00c0dcd0-00c171d0|S0384]] (68), [[S0385 00c17390-00c1f0a0|S0385]] (28), [[S0386 00c1f190-00c26aa0|S0386]] (34), [[S0392 00c42400-00c482e0|S0392]] (9), [[S0393 00c48300-00c4e490|S0393]] (90), [[S0399 00c68550-00c6de90|S0399]] (16), [[S0401 00c73bb0-00c7a5b0|S0401]] (5), [[S0402 00c7a610-00c7f590|S0402]] (74), [[S0403 00c7f620-00c84e90|S0403]] (56), [[S0404 00c84f00-00c89db0|S0404]] (27), [[S0405 00c89e70-00c8f7c0|S0405]] (25), [[S0409 00ca0f50-00ca6310|S0409]] (33), [[S0410 00ca6401-00cab4e0|S0410]] (7), [[S0417 00ccbd60-00cd11e0|S0417]] (2), [[S0418 00cd1210-00cd5dd0|S0418]] (8), [[S0419 00cd5e90-00cdb1a0|S0419]] (5), [[S0420 00cdb370-00ce0140|S0420]] (7), [[S0422 00ce4e30-00ceb920|S0422]] (3), [[S0423 00ceb9b0-00cf13e0|S0423]] (14), [[S0426 00cfc4e0-00d01c70|S0426]] (7), [[S0437 00d297c0-00d2f870|S0437]] (2), [[S0438 00d2f8c0-00d34430|S0438]] (13), [[S0530|S0530]] (2), [[S0540|S0540]] (39), [[S0541|S0541]] (26), [[S0542|S0542]] (18), [[S0549|S0549]] (1), [[S0572|S0572]] (3), [[S0582|S0582]] (2), [[S0596|S0596]] (95), [[S0597|S0597]] (105), [[S0598|S0598]] (58), [[S0599|S0599]] (78), [[S0600|S0600]] (54), [[S0688|S0688]] (17), [[S0733|S0733]] (9), [[S0735|S0735]] (17), [[S0740|S0740]] (10), [[S0776|S0776]] (104), [[S0777|S0777]] (127), [[S0779|S0779]] (14)
