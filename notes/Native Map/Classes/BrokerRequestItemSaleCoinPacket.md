# Class: BrokerRequestItemSaleCoinPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0cd30` | `— ⚠reject` | ⚪ | ctor | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(7), installs BrokerRequestItemSaleCoinPacket::vftable and copies the two static default words into +0x10/+0x14. |
| `00c0cda0` | `BrokerRequestItemSaleCoinPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of BrokerRequestItemSaleCoinPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
