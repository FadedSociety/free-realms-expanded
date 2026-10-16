# Class: PlayerUpdatePacketRemoveTemporaryAppearance

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a09d0` | `PlayerUpdatePacketRemoveTemporaryAppearance::~PlayerUpdatePacketRemoveTemporary…` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketRemoveTemporaryAppearance::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a0a30` | `PlayerUpdatePacketRemoveTemporaryAppearance::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the RemoveTemporaryAppearance, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b4310` | `PlayerUpdatePacketRemoveTemporaryAppearance::PlayerUpdatePacketRemoveTemporaryA…` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0xf) and installs PlayerUpdatePacketRemoveTemporaryAppearance::vftable; no field is initialised. |
