# Class: PlayerUpdatePacketCameraUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0160` | `PlayerUpdatePacketCameraUpdate::PlayerUpdatePacketCameraUpdate` | 🟢 | ctor | Constructor of an embedded member object: installs BasePacket::vftable, writes 0x7e (126) into the id slot param_1[1] and installs PlayerUpdatePacketCameraUpdate::vftable. It does not call the shared FUN_0089fbb0 family… |
| `008a01c0` | `PlayerUpdatePacketCameraUpdate::~PlayerUpdatePacketCameraUpdate` | 🟢 | dtor | Destructor body: restores PlayerUpdatePacketCameraUpdate::vftable then BasePacket::vftable. Called only from EH unwind funclets (01592e2c, 01592f14). |
| `008a0210` | `PlayerUpdatePacketCameraUpdate::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores PlayerUpdatePacketCameraUpdate and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
