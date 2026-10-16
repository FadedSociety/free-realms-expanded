# Class: FactoryPacketDoUpsell

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d9960` | `FactoryPacketDoUpsell::FactoryPacketDoUpsell` | ✅ | ctor | Constructor for FactoryPacketDoUpsell: calls FUN_009d90d0(0x1e), installs FactoryPacketDoUpsell::vftable and clears the words at +0x0c and +0x10. |
| `009db740` | `FactoryPacketDoUpsell::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketDoUpsell::v… |
