# Class: PacketMountList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b05b80` | `PacketMountList::Constructor` | ✅ | ctor | Constructs PacketMountList with mount-family sub-opcode 5 and initializes its packet-info list. |
| `00b05bf0` | `PacketMountList::Destructor` | 🟢 | dtor | Sets PacketMountList vtable, destroys all packet-info list entries, restores list/base vtables, and returns. |
| `00b05c90` | `PacketMountList::ScalarDeletingDtor` | 🟢 | dtor | Calls PacketMountList cleanup and frees the packet allocation when delete flag bit 0 is set. |
