# Class: PlayerUpdatePacketRemoveEffectTagCompositeEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a13d0` | `PlayerUpdatePacketRemoveEffectTagCompositeEffect::Dtor` | ✅ | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2a and also runs from EH funclet 0159173b. |
| `008b51c0` | `PlayerUpdatePacketRemoveEffectTagCompositeEffect::Ctor` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x2a), installs PlayerUpdatePacketRemoveEffectTagCompositeEffect::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and zeroes +0x18. |
| `008b5230` | `PlayerUpdatePacketRemoveEffectTagCompositeEffect::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
