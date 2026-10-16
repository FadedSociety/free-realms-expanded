# Class: TaskCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a869d0` | `— ⚠reject` | ⚪ | ctor | Constructor of TaskCompletePacket: runs the task base constructor with sub-opcode 3 and the caller's id argument, then stores the TaskCompletePacket vtable. |
| `00a87ea0` | `TaskCompletePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of TaskCompletePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
