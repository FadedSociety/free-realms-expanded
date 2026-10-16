# Class: WallOfDataPlayerKeyboardPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00add3b0` | `WallOfDataPlayerKeyboardPacket::~WallOfDataPlayerKeyboardPacket` | ✅ | dtor | Releases the array member through FUN_008c77e0 and then resets the vtable to BasePacket::vftable; called by the scalar deleting dtor 00add520 and by an EH unwind funclet. |
| `00add4c0` | `WallOfDataPlayerKeyboardPacket::WallOfDataPlayerKeyboardPacket` | ✅ | ctor | Base constructor call 00adcd60 with event type 2, then installs WallOfDataPlayerKeyboardPacket::vftable and default-constructs its SoeUtil::Array<unsigned __int64,4,1> member (00add460). |
| `00add520` | `WallOfDataPlayerKeyboardPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: calls the array-releasing destructor 00add3b0 and then frees the object when the flag argument has bit 0 set; it is vtable slot 0 of WallOfDataPlayerKeyboardPacket. |
