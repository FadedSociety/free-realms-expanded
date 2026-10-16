# Class: SelectTaskRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86b30` | `SelectTaskRequestCtor` | ✅ | ctor | Constructor of SelectTaskRequest: runs the BaseUiPacket constructor with sub-opcode 6 (class id 47), stores the SelectTaskRequest vtable and saves the three caller-supplied arguments into +0x10, +0x14 and +0x18. |
| `00a87f60` | `SelectTaskRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of SelectTaskRequest: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
