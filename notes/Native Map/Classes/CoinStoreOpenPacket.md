# Class: CoinStoreOpenPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c74490` | `CoinStoreOpenPacket::Construct` | ✅ | ctor | Constructor: calls the base constructor FUN_00c6e960 with 7, stores the CoinStoreOpenPacket vtable, calls FUN_00c72420 and FUN_0079e0d0 for members, and stores 0 at dword index 0xe3 (+0x38c), -1 at dword index 0xe4 (+0x… |
| `00c74520` | `— ⚠reject` | ⚪ | dtor | Destructor body for the store-open packet: calls FUN_0079e170 on the object, then FUN_00c72e50 on its member, then resets the vtable to BaseCoinStorePacket and BasePacket. |
| `00c74f70` | `— ⚠reject` | ⚪ | method | Parses the store-open body from the bounded reader at param_2: calls FUN_008dab40 into the sub-structure at this+0x334 and FUN_00c72060 into this+0xc, then reads three dwords into this+0x38c, +0x390 and +0x394, clamping… |
| `00c751d0` | `CoinStoreOpenPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c74520 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
