# Class: PacketLoadWelcomeScreen

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090b5f0` | `PacketLoadWelcomeScreen::PacketLoadWelcomeScreen ⚠downgrade` | 🟢 | ctor | Sets packet opcode 0x5d, initializes the welcome-screen flag, and installs empty ContentInfo and ClaimCodeInfo arrays. |
| `0090b680` | `PacketLoadWelcomeScreen::~PacketLoadWelcomeScreen` | ✅ | dtor | Restores the claim-code and content array vtables, frees their allocations, then restores BasePacket::vftable. |
| `00914200` | `ScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] (thiscall, one byte argument). Calls thiscall FUN_0090b680 with ECX=this (the destructor body), then if (arg & 1) != 0 calls _free(this); returns this in EAX. |
