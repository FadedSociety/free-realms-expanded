# Class: BrokerInformationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bee850` | `BrokerInformationPacket::Ctor ⚠downgrade` | 🟢 | ctor | Constructs subtype 12 of BaseBrokerPacket, installs BrokerInformationPacket vtable, and clears four payload fields. |
| `00beeac0` | `BrokerInformationPacket::ScalarDeletingDtor` | ✅ | dtor | Restores the broker base vtables, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
