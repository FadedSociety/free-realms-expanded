# Class: PlayerUpdatePacketScriptedAnimation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0810` | `PlayerUpdatePacketScriptedAnimation::~PlayerUpdatePacketScriptedAnimation` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x18 and EH funclet 015916b4. |
| `008b3fc0` | `PlayerUpdatePacketScriptedAnimation::PlayerUpdatePacketScriptedAnimation ⚠downg…` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x18) and installs PlayerUpdatePacketScriptedAnimation::vftable; nothing else is written. |
| `008b4020` | `PlayerUpdatePacketScriptedAnimation::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
