# Class: BrokerSearchPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0c670` | `— ⚠reject` | ⚪ | ctor | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(2), installs BrokerSearchPacket::vftable, then initialises the five filter fields at +0xc..+0x20 to 0xffffffff and the byte at +0x24 to 0. |
| `00c0c9a0` | `BrokerSearchPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of BrokerSearchPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
