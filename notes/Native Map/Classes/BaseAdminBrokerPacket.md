# Class: BaseAdminBrokerPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bee720` | `BaseAdminBrokerPacket::CtorWithSubtype` | ✅ | ctor | Installs BasePacket and BaseAdminBrokerPacket vtables, writes 0x84 to field 1, and stores the supplied subtype in field 2. |
| `00bee790` | `BaseAdminBrokerPacket::Dtor` | ✅ | dtor | Restores BaseAdminBrokerPacket and BasePacket vtables during destruction. |
| `00bee7e0` | `BaseAdminBrokerPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BaseAdminBrokerPacket and BasePacket vtables, frees the object if flag bit 0 is set, and otherwise returns its pointer. |
