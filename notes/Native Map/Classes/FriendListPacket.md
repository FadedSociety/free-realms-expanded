# Class: FriendListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c81040` | `— ⚠reject` | ⚪ | ctor | Initializes packet family value 1, installs FriendListPacket and typed FriendData-list vtables, and zeros list fields. |
| `00c81230` | `FriendListPacket::ScalarDeletingDtor` | ✅ | dtor | Calls FriendListPacket cleanup and conditionally frees the object when flag bit 0 is set. |
