# Class: PlayerUpdatePacketSlotCompositeEffectOverride

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0eb0` | `PlayerUpdatePacketSlotCompositeEffectOverride::~PlayerUpdatePacketSlotComposite…` | ✅ | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1f and EH funclet 015914c7. |
| `008b4b70` | `PlayerUpdatePacketSlotCompositeEffectOverride::PlayerUpdatePacketSlotCompositeE…` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x1f), installs PlayerUpdatePacketSlotCompositeEffectOverride::vftable and stores param_2..param_5 into +0x10..+0x1c. |
| `008b4bf0` | `PlayerUpdatePacketSlotCompositeEffectOverride::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
