# Class: ClientGateway::PacketOnlineStatusReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc5d0` | `ClientGateway::PacketOnlineStatusReply::PacketOnlineStatusReply` | ✅ | ctor | Constructor: installs BasePacket::vftable, stamps the packet id 9 into the word at +4, installs ClientGateway::PacketOnlineStatusReply::vftable and clears the byte at +8. |
| `00acc630` | `ClientGateway::PacketOnlineStatusReply::Destructor` | ✅ | dtor | 73-byte destructor body: rewrites the vtable to ClientGateway::PacketOnlineStatusReply::vftable and then to ClientGateway::BasePacket::vftable; reached from an unwind funclet and from the dispatcher's case 9 cleanup. |
| `00acc680` | `ClientGateway::PacketOnlineStatusReply::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to ClientGateway::PacketOnlineStatusReply::vftable then BasePacket::vftable, and frees the object with _free when bit 0 of the flag argument is set. |
