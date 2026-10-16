# Class: CommandPacketDirectInspect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1ab80` | `— ⚠reject` | ⚪ | ctor | Calls the command packet base constructor with sub-opcode 0x2d and zero, installs its vtable, and stores two arguments. |
| `00c1abf0` | `CommandPacketDirectInspect::ScalarDeletingDtor` | ✅ | dtor | Resets the inherited command and packet vptrs, then frees the allocation when the deleting flag bit is set. |
