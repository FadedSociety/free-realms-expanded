# Class: ClientGateway::PacketServerForcedLogout

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc360` | `ClientGateway::PacketServerForcedLogout::Destructor` | 🟢 | dtor | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable; it is reached from an unwind funclet and from the dispatcher's case 4 cleanup. |
| `00acd230` | `— ⚠reject` | ⚪ | ctor | 126-byte constructor/parser: installs BasePacket::vftable, stamps the id 4 into the word at +4, installs ClientGateway::PacketServerForcedLogout::vftable and then, when the body (param_2,param_3) holds at least two byte… |
| `00acd2b0` | `ClientGateway::PacketServerForcedLogout::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of the flag argument is set. |
