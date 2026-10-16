# Class: BrokerErrorPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00beeed0` | `— ⚠reject` | ⚪ | ctor | Constructs broker subtype 0x82 base state, installs the BrokerErrorPacket vtable, and initializes its shared error string and fields. |
| `00beef40` | `BrokerErrorPacket::Dtor` | ✅ | dtor | Releases the owned shared string and restores BaseBrokerPacket then BasePacket vtables. |
| `00bef090` | `BrokerErrorPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the broker error packet destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
