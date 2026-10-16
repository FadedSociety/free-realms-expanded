# Class: BaseCoinStorePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6e960` | `— ⚠reject` | ⚪ | ctor | Installs BasePacket::vftable, writes family opcode 0xa5 (165) to word [1], installs BaseCoinStorePacket::vftable, then stores the sub-opcode argument in word [2]. Returns nothing (void). |
| `00c6e9d0` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable into the object. No free and no callees; 73-byte base teardown body. |
| `00c6ea20` | `BaseCoinStorePacket::ScalarDeletingDtor` | ✅ | dtor | Reinstalls BaseCoinStorePacket then BasePacket vtables, runs the base body, and when bit 0 of the second argument is set calls _free(this) through thunk 00d82463. Returns this. |
| `00c6ed50` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable into the object. No free and no callees; 76-byte base teardown body. |
| `00c6ee10` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable. Identical body to 00c6e9d0 (76 bytes), no free. |
| `00c6eff0` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c6f0a0` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c6f0f0` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c6f140` | `BaseCoinStorePacket::BaseDtor` | ✅ | dtor | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c74360` | `— ⚠reject` | ⚪ | dtor | Destructor body: calls FUN_00c00d50 on the object, then resets the vtable to BaseCoinStorePacket and finally to BasePacket (the base-class vtable is the last store). |
| `00c74430` | `BaseCoinStorePacket::DtorBody` | ✅ | dtor | Destructor body (second copy, same as 00c74360): calls FUN_00c00d50 on the object, then resets the vtable to BaseCoinStorePacket and finally to BasePacket. |
