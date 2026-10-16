# Class: IgnoreListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c80df0` | `IgnoreListPacket::IgnoreListPacket` | ✅ | ctor | Initializes packet family value 1, installs IgnoreListPacket and typed IgnoreData-list vtables, and zeros the list fields. |
| `00c81130` | `IgnoreListPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the IgnoreListPacket base cleanup and conditionally frees the packet object when flag bit 0 is set. |
