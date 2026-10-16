# Class: GroupPacketAnnounceEncounterReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a39c0` | `GroupPacketAnnounceEncounterReply::~GroupPacketAnnounceEncounterReply ? ⚠downgr…` | 🟡 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the announce-encounter reply with it after constructing the class from FUN_008b72f0 (group sub 13). |
| `008b72f0` | `GroupPacketAnnounceEncounterReply::GroupPacketAnnounceEncounterReply` | ✅ | ctor | Calls the group family initializer for top-level opcode 40 008a35c0(0xd) and installs GroupPacketAnnounceEncounterReply::vftable; no field is initialised. |
| `008b7370` | `GroupPacketAnnounceEncounterReply::CopyPayloadFields10And14` | ✅ | virtual | Copies the dwords at param_2+0x10 and param_2+0x14 into this+0x10 and this+0x14; a two-field copy with no other effect. |
| `008b7390` | `GroupPacketAnnounceEncounterReply::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
