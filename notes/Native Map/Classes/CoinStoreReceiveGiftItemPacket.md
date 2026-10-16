# Class: CoinStoreReceiveGiftItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c74590` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the base constructor FUN_00c6e960 with 0x11, stores the CoinStoreReceiveGiftItemPacket vtable, sets the SoeUtil::IString<char> member at +0x18 with its vtable, sets the string pointer at +0x1c to a st… |
| `00c74620` | `— ⚠reject` | ⚪ | dtor | Destructor body for the gift item packet: calls FUN_00c00d50 on the object, resets the SoeUtil::IString<char> member vtable at +0x18, releases the string buffer at +0x1c when the count at +0x24 is positive (interlocked … |
| `00c751f0` | `CoinStoreReceiveGiftItemPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c74620 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
