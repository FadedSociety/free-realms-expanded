# Class: ClientGateway::PacketLogout

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc2b0` | `ClientGateway::PacketLogout::PacketLogout` | ✅ | ctor | Constructor of ClientGateway::PacketLogout: installs BasePacket::vftable, stamps the packet id 3 into the packet word at +4, then installs ClientGateway::PacketLogout::vftable. |
| `00acca20` | `ClientGateway::PacketLogout::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of the flag argument is set. |
