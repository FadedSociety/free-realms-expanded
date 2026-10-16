# Class: ClientGateway::PacketLoginReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc260` | `ClientGateway::PacketLoginReply::Destructor` | 🟢 | dtor | 67-byte destructor body: rewrites the object's vtable to ClientGateway::BasePacket::vftable and returns; it is reached from compiler unwind funclets and as the cleanup at the end of the dispatcher's case 2. |
| `00acd140` | `— ⚠reject` | ⚪ | ctor | Constructor/parser of the login reply inside an SEH frame: installs BasePacket::vftable, stamps the reply id 2 into the word at +4, installs ClientGateway::PacketLoginReply::vftable, then reads the body (param_2,param_3… |
| `00acd1d0` | `ClientGateway::PacketLoginReply::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor inside an SEH frame: resets the object's vtable to ClientGateway::BasePacket::vftable and frees the object with _free when bit 0 of the flag argument is set. |
