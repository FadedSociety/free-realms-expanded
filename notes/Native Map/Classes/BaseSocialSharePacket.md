# Class: BaseSocialSharePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a5040` | `BaseSocialSharePacket::BaseSocialSharePacket` | ✅ | ctor | Social-share family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 205 (0xcd) at +4, installs BaseSocialSharePacket::vftable and zeroes +8; no calls. |
| `008a50a0` | `BaseSocialSharePacket::~BaseSocialSharePacket` | ✅ | dtor | Destructor body: restores BaseSocialSharePacket::vftable then BasePacket::vftable; 14 EH funclets (0158a320 and the 015caxxx block) call it. |
| `008a50f0` | `BaseSocialSharePacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: in an SEH frame restores BaseSocialSharePacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
| `00ade960` | `BaseSocialSharePacket::BaseSocialSharePacket` | ✅ | ctor | Base constructor for the social-share family: installs BasePacket::vftable, writes 0xcd (205) into the 2-byte packet id slot at +4, installs BaseSocialSharePacket::vftable and stores the sub-opcode argument at +8. |
| `00adea30` | `BaseSocialSharePacket::~BaseSocialSharePacket` | ✅ | dtor | Destructor body that restores BaseSocialSharePacket::vftable and then BasePacket::vftable; a single EH unwind funclet calls this copy. |
| `00adeae0` | `BaseSocialSharePacket::~BaseSocialSharePacket` | ✅ | dtor | Destructor body that restores BaseSocialSharePacket::vftable and then BasePacket::vftable; a single EH unwind funclet calls this second copy. |
