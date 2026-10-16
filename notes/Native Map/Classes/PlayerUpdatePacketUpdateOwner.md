# Class: PlayerUpdatePacketUpdateOwner

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0350` | `PlayerUpdatePacketUpdateOwner::~PlayerUpdatePacketUpdateOwner` | 🟢 | dtor | Destructor body that also carries an SEH frame and a stack-cookie check: restores BasePlayerUpdatePacket::vftable and BasePacket::vftable, calls FUN_0101b370 with the cookie global DAT_01b839d8, and touches no other fie… |
| `008b3c70` | `PlayerUpdatePacketUpdateOwner::PlayerUpdatePacketUpdateOwner` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x33), installs PlayerUpdatePacketUpdateOwner::vftable, writes Target::vftable into +0x18, zeroes +0x1c and the byte at +0x20 (the target sub-object embedded in the packet… |
| `008b3ce0` | `— ⚠reject` | ⚪ | dtor | Calls the player-update packet destructor at 008a0350 (accepted map label: PlayerUpdatePacketUpdateOwner::~PlayerUpdatePacketUpdateOwner) and then calls operator delete on this when bit 0 of the flag argument is set; re… |
