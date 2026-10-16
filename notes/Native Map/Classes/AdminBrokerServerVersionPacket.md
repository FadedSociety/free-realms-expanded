# Class: AdminBrokerServerVersionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00beedd0` | `— ⚠reject` | ⚪ | ctor | Constructs administrative subtype 0x11, installs the concrete vtable, and initializes version text storage and numeric fields. |
| `00beee40` | `AdminBrokerServerVersionPacket::Dtor` | ✅ | dtor | Releases the owned version string reference and restores BaseAdminBrokerPacket then BasePacket vtables. |
| `00bef070` | `AdminBrokerServerVersionPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the server-version packet destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
