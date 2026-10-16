# Class: PlayerUpdatePacketFlourish

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b3aa0` | `PlayerUpdatePacketFlourish::PlayerUpdatePacketFlourish` | ✅ | ctor | Stores BasePacket::vftable, writes packet id 0xb5 (181) into +4, installs PlayerUpdatePacketFlourish::vftable and copies the global pair DAT_01be4850/DAT_01be4854 into +8/+0xc. |
| `008b3b10` | `— ⚠reject` | ⚪ | dtor | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
