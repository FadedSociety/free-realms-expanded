# Class: ClientGateway::PacketLogin

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00accb60` | `ClientGateway::PacketLogin::PacketLogin` | ✅ | ctor | Constructor of ClientGateway::PacketLogin: installs BasePacket::vftable, stamps the packet id 1 into the word at +4, installs ClientGateway::PacketLogin::vftable and initialises three SoeUtil::IString members (vtables a… |
| `00accc00` | `— ⚠reject` | ⚪ | dtor | 220-byte destructor body inside an SEH frame: for each of the three SoeUtil::IString members (+0x30, +0x20, +8) it reinstalls the IString vtable and, when the buffer's reference count is positive, decrements the count a… |
| `00accd80` | `ClientGateway::PacketLogin::ScalarDeletingDtor` | ✅ | dtor | 27-byte scalar deleting destructor: calls the destructor body 00accc00 (which releases the three strings) and frees the object with _free when bit 0 of the flag argument is set. |
