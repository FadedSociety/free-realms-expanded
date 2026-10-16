# Class: SoccerPacketImpulseBall

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b73790` | `SoccerPacketImpulseBall::Ctor ⚠downgrade` | 🟢 | ctor | Constructor of the family-76 sub 0xd (13) packet SoccerPacketImpulseBall: calls the shared base ctor FUN_00b730a0(0xd) (family 0x4C at +4, sub 13 at +8) and installs SoccerPacketImpulseBall::vftable (0x018232F4). |
| `00b75150` | `SoccerPacketImpulseBall::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
