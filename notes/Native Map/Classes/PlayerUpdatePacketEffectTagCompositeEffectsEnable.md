# Class: PlayerUpdatePacketEffectTagCompositeEffectsEnable

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1420` | `PlayerUpdatePacketEffectTagCompositeEffectsEnable` | ✅ | ctor | Sub-opcode-43 constructor: calls FUN_0089fbb0(0x2b) (opcode 35, sub-opcode 43), installs its own vftable and zeroes one byte at param_1+0xc (param_1[3] as a byte). |
| `008a1480` | `PlayerUpdatePacketEffectTagCompositeEffectsEnable::Dtor` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2b and also runs from EH funclet 01591746. |
| `008b52a0` | `PlayerUpdatePacketEffectTagCompositeEffectsEnable::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
