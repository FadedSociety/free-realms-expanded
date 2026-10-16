# Class: BattleMagesPacketUpdateGameState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6f520` | `BattleMagesPacketUpdateGameState::BattleMagesPacketUpdateGameState ⚠downgrade` | 🟢 | ctor | Constructor for sub-opcode 6: calls the shared base constructor with 6, stores BattleMagesPacketUpdateGameState::vftable at +0 and zeroes the 32-bit field at +0xc. |
| `00b6ff40` | `BattleMagesPacketUpdateGameState::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the UpdateGameState packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
