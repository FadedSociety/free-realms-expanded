# Class: PlayerUpdatePacketAddEffectTagCompositeEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1380` | `PlayerUpdatePacketAddEffectTagCompositeEffect::~PlayerUpdatePacketAddEffectTagC…` | ✅ | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x29 and also runs from EH funclet 01591730. |
| `008c4d50` | `PlayerUpdatePacketAddEffectTagCompositeEffect::Constructor` | ✅ | ctor | PlayClient case 0x29 constructs this packet. Base constructor sets family opcode 0x23/subcode 0x29; this body installs its vtable and initializes identifier and payload fields. |
| `008c4df0` | `PlayerUpdatePacketAddEffectTagCompositeEffect::ScalarDeletingDestructor` | ✅ | dtor | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
