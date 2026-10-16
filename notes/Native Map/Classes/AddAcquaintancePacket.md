# Class: AddAcquaintancePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a19170` | `AddAcquaintancePacket::Ctor` | 🟢 | ctor | Constructs the acquaintance base packet, installs AddAcquaintancePacket vftable, initializes a pair field, and conditionally copies the supplied record fields and string. |
| `00a19220` | `AddAcquaintancePacket::ScalarDeletingDtor` | 🟢 | dtor | Calls the BasePacket destructor 00a150f0, conditionally frees this object, and otherwise returns it. |
