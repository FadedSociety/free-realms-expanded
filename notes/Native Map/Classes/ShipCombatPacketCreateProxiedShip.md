# Class: ShipCombatPacketCreateProxiedShip

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a678f0` | `ShipCombatPacketCreateProxiedShip::ShipCombatPacketCreateProxiedShip` | ✅ | ctor | Constructor: runs the shared ship-combat base constructor FUN_00a4ec00(9) (which writes BasePacket::vftable, the family opcode 0x5e at +4, BaseShipCombatPacket::vftable and the sub-opcode 9 at +8), stores ShipCombatPack… |
| `00a679a0` | `ShipCombatPacketCreateProxiedShip::~ShipCombatPacketCreateProxiedShip` | ✅ | dtor | Destructor body: rewrites its own vtable into the object, then destroys the payload members in reverse construction order - FUN_00a55550 twice (the two SoeUtil::StringFixed<14> objects), FUN_00a67040 (the second record)… |
| `00a67a50` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor in vtable slot 0: bytes 56 8b f1 e8 48 ff ff ff f6 44 24 08 01 74 09 56 e8 fe a9 31 00 83 c4 04 8b c6 5e decode as push esi; mov esi,ecx; call 00a679a0 (the destructor body); test byte [e… |
| `00a69b90` | `ShipCombatPacketCreateProxiedShip_Read ? ⚠downgrade` | 🟡 | method | Parses the CreateProxiedShip payload into the packet object in ecx: binds the bounded reader over param_2 with FUN_00a5a0a0, reads the 8-byte guid into +0x10/+0x14, a byte flag into +0x18, a u32 into +0x1c, then four 4-… |
