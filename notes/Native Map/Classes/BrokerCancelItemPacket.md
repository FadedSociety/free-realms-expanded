# Class: BrokerCancelItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0cc50` | `— ⚠reject` | ⚪ | ctor | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(6), installs BrokerCancelItemPacket::vftable and copies the two static default words into +0x10/+0x14. |
| `00c0ccc0` | `BrokerCancelItemPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of BrokerCancelItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
