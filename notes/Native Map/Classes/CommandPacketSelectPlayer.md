# Class: CommandPacketSelectPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a29a0` | `CommandPacketSelectPlayer::~CommandPacketSelectPlayer` | ✅ | dtor | Destructor body: restores BaseCommandPacket::vftable then BasePacket::vftable; the two callers that matter are 00934870 and 0093bff0, which both construct CommandPacketSelectPlayer first (via FUN_008b6af0) and destroy i… |
| `008b6af0` | `CommandPacketSelectPlayer::CommandPacketSelectPlayer ⚠downgrade` | 🟢 | ctor | Calls the command family constructor 008a2820(0x13, 0) and installs CommandPacketSelectPlayer::vftable; no field is initialised. |
| `008b6b50` | `CommandPacketSelectPlayer::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
