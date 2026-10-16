# Class: WallOfDataBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adcd60` | `WallOfDataBasePacket::WallOfDataBasePacket` | ✅ | ctor | Base constructor for the wall-of-data telemetry family: installs BasePacket::vftable, writes 0xc2 (194) into the 2-byte packet id slot at +4, installs WallOfDataBasePacket::vftable and stores the caller's event-type arg… |
| `00add0a0` | `WallOfDataBasePacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtable slot 0 of WallOfDataBasePacket. |
