# Class: BrokerBuyItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0cb70` | `— ⚠reject` | ⚪ | ctor | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(5), installs BrokerBuyItemPacket::vftable, copies the two static default words into +0x10/+0x14 and zeroes +0x18. |
| `00c0cbe0` | `BrokerBuyItemPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of BrokerBuyItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
