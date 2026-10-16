# Class: BattleMagesPacketUpdatePlayerState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd4d90` | `BattleMagesPacketUpdatePlayerState::BattleMagesPacketUpdatePlayerState` | ✅ | ctor | Calls the BaseBattleMagesPacket constructor 00b6f2b0 with sub-opcode 7; stores BattleMagesPacketUpdatePlayerState::vftable at this; stores DAT_01bf0a38 at +0x10, DAT_01bf0a3c at +0x14 and zero at +0x18; returns this. |
| `00cd4e00` | `BattleMagesPacketUpdatePlayerState::RestoreBaseVtablesThenFreeIfFlag` | ✅ | virtual | Virtual slot 0: stores BaseBattleMagesPacket::vftable and then BasePacket::vftable at this, frees this when bit 0 of the flag argument is set, and returns this. No separate destructor body is called. |
