# Class: PlayerUpdateTriggerEffectPackagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1310` | `PlayerUpdateTriggerEffectPackagePacket::~PlayerUpdateTriggerEffectPackagePacket` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x24 and also runs from EH funclet 0159171a. |
| `008c4c40` | `PlayerUpdateTriggerEffectPackagePacket::Constructor` | ✅ | ctor | PlayClient case 0x24 constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x24; this body installs its vtable, copies defaults into +0x10..+0x1c, and zeros payload dwords +0x20 through +0x40. |
| `008c4ce0` | `PlayerUpdateTriggerEffectPackagePacket::ScalarDeletingDestructor` | ✅ | dtor | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
