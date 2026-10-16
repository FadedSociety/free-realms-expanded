# Class: BrokerPlaceItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0c730` | `— ⚠reject` | ⚪ | ctor | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(3), installs BrokerPlaceItemPacket::vftable, initialises the embedded guid/string member through FUN_0101cd40 and zeroes the four fields at +0x10… |
| `00c0ca10` | `BrokerPlaceItemPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of BrokerPlaceItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
