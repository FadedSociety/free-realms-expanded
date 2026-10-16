# Class: ClientGateway::PacketOnlineStatusRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc4b0` | `ClientGateway::PacketOnlineStatusRequest::PacketOnlineStatusRequest` | ✅ | ctor | Constructor: installs BasePacket::vftable, stamps the packet id 8 into the word at +4 and installs ClientGateway::PacketOnlineStatusRequest::vftable. |
| `00acc510` | `ClientGateway::PacketOnlineStatusRequest::Destructor` | ✅ | dtor | 73-byte destructor body: rewrites the vtable to ClientGateway::PacketOnlineStatusRequest::vftable and then to ClientGateway::BasePacket::vftable (MSVC's two-step reset when the base destructor is inlined); reached only … |
| `00acc560` | `ClientGateway::PacketOnlineStatusRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to ClientGateway::PacketOnlineStatusRequest::vftable then BasePacket::vftable, and frees the object with _free when bit 0 of the flag argument is set. |
