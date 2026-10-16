# Class: AbilityPacketPurchaseAbility

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2ce70` | `AbilityPacketPurchaseAbility::AbilityPacketPurchaseAbility` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 7, stores AbilityPacketPurchaseAbility::vftable, and sets +0xc to 0xffffffff and +0x10 to 0. |
| `00a2df20` | `AbilityPacketPurchaseAbility::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketPurchaseAbility::vftable at 018138e… |
