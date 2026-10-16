# Class: ClientGatewayApi

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc810` | `ClientGatewayApi::ClientGatewayApi` | ✅ | ctor | Constructor: registers the API name 'CGAPI_527' by running the crypto/API base constructor FUN_00d33270 (ClientServerCrypto::CryptoBaseApi) with the caller's third argument, stores the callback object from param_2 at th… |
| `00acc8f0` | `ClientGatewayApi::Destructor` | ✅ | dtor | 92-byte destructor body inside an SEH frame: reinstalls the three ClientGatewayApi vtables and calls FUN_00d33320, the ClientServerCrypto::CryptoBaseApi destructor that releases the crypto object at this+0xbe8 and tears… |
| `00acc950` | `ClientGatewayApi::SetApiKind2` | 🟢 | method | 17-byte wrapper (bytes 8b 44 24 04 6a 00 50 6a 02 e8 ..): pushes 0, the single argument and the constant 2 and calls FUN_00d333c0 with this (ecx) unchanged, returning its result; FUN_00d333c0 replaces the connection's A… |
| `00acc970` | `ClientGatewayApi::OnFailed` | ✅ | virtual | Virtual slot 15 of ClientGatewayApi: prints 'ClientGatewayApi::OnFailed...\n' with _wprintf and, when the callback object at this+0xbf0 exists, calls its virtual slot +8 with this. |
| `00acc9a0` | `ClientGatewayApi::OnConnect` | ✅ | virtual | Virtual slot 16 of ClientGatewayApi: prints 'ClientGatewayApi::OnConnect...\n' with _wprintf and, when the callback object at this+0xbf0 exists, calls its virtual slot 0 with this. |
| `00acc9d0` | `ClientGatewayApi::OnDisconnect` | ✅ | virtual | Virtual slot 17 of ClientGatewayApi: prints 'ClientGatewayApi::OnDisconnect...\n' with _wprintf and, when the callback object at this+0xbf0 exists, calls its virtual slot +4 with this. |
| `00acca00` | `ClientGatewayApi::ScalarDeletingDtorAdjustor_c4` | 🟢 | virtual | 11-byte adjustor thunk (bytes 81 e9 c4 00 00 00 e9 ..): subtracts 0xc4 from this and jumps to the scalar deleting destructor 00accae0; it is slot 0 of the secondary ClientGatewayApi vtable installed at this+0xc4. |
| `00acca10` | `ClientGatewayApi::ScalarDeletingDtorAdjustor_c0` | 🟢 | virtual | 11-byte adjustor thunk (bytes 81 e9 c0 00 00 00 e9 ..): subtracts 0xc0 from this and jumps to the scalar deleting destructor 00accae0; it is slot 0 of the other secondary ClientGatewayApi vtable, installed at this+0xc0. |
| `00accae0` | `ClientGatewayApi::ScalarDeletingDtor` | ✅ | dtor | 27-byte scalar deleting destructor: calls the destructor body 00acc8f0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00acd3f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 18 of ClientGatewayApi: reads the first u16 of the body (0 when fewer than two bytes) and switches on it - 2 builds a PacketLoginReply (00acd140) and calls the listener's virtual +0xc with (api, reply) befo… |
