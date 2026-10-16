# Class: WallOfDataUIEventPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00add540` | `WallOfDataUIEventPacket::~WallOfDataUIEventPacket` | ✅ | dtor | Releases the three IStringFixed<char,64> members through FUN_004022b0 and then resets the vtable to BasePacket::vftable. |
| `00add700` | `WallOfDataUIEventPacket::WallOfDataUIEventPacket` | ✅ | ctor | Base constructor call 00adcd60 with event type 4 (opcode 194/4, the catalogue's WallOfDataUIEventPacket), installs WallOfDataUIEventPacket::vftable and default-constructs three SoeUtil::StringFixed<64> members (FUN_0040… |
| `00add780` | `WallOfDataUIEventPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: calls the string-releasing destructor 00add540 and then frees the object when the flag argument has bit 0 set; it is vtable slot 0 of WallOfDataUIEventPacket. |
