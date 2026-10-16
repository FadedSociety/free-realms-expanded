# Class: PlayerUpdateProfileNameplateImageIdPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1a00` | `PlayerUpdateProfileNameplateImageIdPacket::~PlayerUpdateProfileNameplateImageId…` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; both branches of dispatcher case 0x47 call it before the shared base-destructor epilogue, and EH funclet 01591843 also calls it. |
| `008b5d90` | `PlayerUpdateProfileNameplateImageIdPacket::PlayerUpdateProfileNameplateImageIdP…` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x47), installs PlayerUpdateProfileNameplateImageIdPacket::vftable, zeroes +0xc and stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14. |
| `008b5e00` | `PlayerUpdateProfileNameplateImageIdPacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
