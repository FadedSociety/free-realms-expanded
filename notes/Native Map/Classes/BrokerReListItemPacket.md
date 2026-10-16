# Class: BrokerReListItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0ca80` | `BrokerReListItemPacket_Ctor ⚠downgrade` | 🟢 | ctor | Broker packet constructor: calls the broker base constructor FUN_00bee5f0(4), installs BrokerReListItemPacket::vftable, copies two static default words (DAT_01bec9a0/DAT_01bec9a4) into +0x10/+0x14 and zeroes the four fi… |
| `00c0cb00` | `BrokerReListItemPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of BrokerReListItemPacket::vftable: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
