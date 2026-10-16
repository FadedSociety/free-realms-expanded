# Class: BrokerMyItemsPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cee2d0` | `BrokerMyItemsPacket::Ctor` | ✅ | ctor | Calls the base packet initializer with subtype 8, installs BrokerMyItemsPacket vtable, and stores the constructor argument at field 3. |
| `00ceeb00` | `BrokerMyItemsPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BaseBrokerPacket then BasePacket vtables, and frees the object when the deleting bit is set. |
