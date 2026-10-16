# Class: ClientGateway::PacketTunneledClientGatewayPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc400` | `ClientGateway::PacketTunneledClientGatewayPacket::PacketTunneledClientGatewayPa…` | 🟢 | ctor | Constructor: installs BasePacket::vftable, stamps the packet id 10 into the word at +4 and installs ClientGateway::PacketTunneledClientGatewayPacket::vftable. |
| `00acca80` | `ClientGateway::PacketTunneledClientGatewayPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of the flag argument is set. |
