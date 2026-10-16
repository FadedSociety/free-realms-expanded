# Class: BaseShipCombatPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4eba0` | `— ⚠reject` | ⚪ | ctor | Default constructor of the ship-combat packet base: stores BasePacket::vftable, writes the shared packet opcode 0x5e (94) into field +4, stores BaseShipCombatPacket::vftable and zeroes the sub-opcode field +8. |
| `00a4ec00` | `BaseShipCombatPacket::BaseShipCombatPacket_Sub` | ✅ | ctor | Shared base constructor of every concrete ship-combat packet: stores BasePacket::vftable, writes opcode 0x5e (94) into field +4, stores BaseShipCombatPacket::vftable and stores the caller-supplied sub-opcode (message di… |
| `00a4ec70` | `BaseShipCombatPacket::~BaseShipCombatPacket` | ✅ | dtor | Destructor body of the ship-combat packet base: resets the object's vtable to BaseShipCombatPacket then to BasePacket; no members are destroyed. Called from 112 exception-unwind funclets, the standard marker of a destru… |
| `00a4ecc0` | `BaseShipCombatPacket::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor occupying vtable slot 0: resets the vtable to BaseShipCombatPacket then BasePacket and frees the packet through the _free thunk when the deletion flag (param_2 & 1) is set. |
| `00a575c0` | `— ⚠reject` | ⚪ | dtor | Destroys the string member at offset +0x60, sets the base ship-combat packet vtable, then replaces it with BasePacket vtable. |
| `00a57620` | `BaseShipCombatPacket_DtorWithFixed64 → BaseShipCombatPacket_Dtor_Fixed64At60And…` | 🟢 | dtor | Destroys two embedded fixed-64 string members under SEH, then resets the object through BaseShipCombatPacket and BasePacket vtables. |
| `00a577a0` | `BaseShipCombatPacket_Dtor → BaseShipCombatPacket_Dtor_Fixed32At14` | 🟢 | dtor | Runs FUN_00703350 for a base member inside an SEH frame, then restores BaseShipCombatPacket and BasePacket vtables. |
| `00a57810` | `BaseShipCombatPacket_Dtor → BaseShipCombatPacket_Dtor_Fixed14At34` | 🟢 | dtor | Destroys an embedded fixed string through FUN_00a55550, then resets the object to BaseShipCombatPacket and BasePacket vtables. |
| `00ca2c80` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Replaces the object's vtable first with BaseShipCombatPacket and then with BasePacket during base destruction. |
| `00ca2cd0` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2d20` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2d70` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2e30` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2ee0` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2f30` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca2f90` | `BaseShipCombatPacket::Dtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables in sequence. |
| `00ca64e0` | `— ⚠reject` | ⚪ | handler | Constructs a local packet via 00ca3df0, applies a (param_1, param_2) length test, calls 00ca59a0 to parse, and when the parsed id pair differs from DAT_01bef6b0/DAT_01bef6b4 and 00ca46a0 finds an object, forwards the pa… |
| `00ca66a0` | `— ⚠reject` | ⚪ | handler | Constructs a local packet via 00ca3f10, runs a range test through 00ca5a60 on (param_1, param_2) with a non-null check on param_1, and on the valid path calls 00ca46a0 and, when an object is found, 00c9a5b0 on it, destr… |
| `00ca68f0` | `BaseShipCombatPacket::ParseApplyRecord_00ca68f0` | 🟢 | handler | Constructs a local packet via 00ca3870 and range-tests (param_1, param_2) through 00ca5cb0. On the valid path with a non-sentinel id it looks up the object with 00ca46a0, sets bit 0x100 at +0x1c, copies 16 dwords from t… |
| `00ca6c90` | `— ⚠reject` | ⚪ | handler | Takes two float arguments; when the first is non-zero it constructs a local packet via 00ca3980, range-tests the pair through 00ca5f90, looks up the object via 00ca46a0, takes a 2D offset from object transform data (00f… |
