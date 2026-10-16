# Class: ClientGateway::PacketTunneledClientPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089adc0` | `ClientGateway::PacketTunneledClientPacketCtor` | ✅ | ctor | Constructor of ClientGateway::PacketTunneledClientPacket: installs ClientGateway::BasePacket::vftable, stores the gateway opcode 5 at +4, then installs ClientGateway::PacketTunneledClientPacket::vftable. |
| `008b0ec0` | `PacketTunneledClientPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of ClientGateway::PacketTunneledClientPacket::vftable: inside an SEH frame it writes ClientGateway::BasePacket::vftable at offset 0 and, when bit 0 of the flag argument is set, frees… |
