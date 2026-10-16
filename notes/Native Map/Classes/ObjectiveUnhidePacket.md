# Class: ObjectiveUnhidePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b9f40` | `ObjectiveUnhidePacket::ObjectiveUnhidePacket` | ✅ | ctor | Constructs the objective-family packet: FUN_009aa4c0(5) stores the family opcode 0x2d (45) with sub-opcode 5 and installs BaseObjectivePacket::vftable; this constructor then installs ObjectiveUnhidePacket::vftable and i… |
| `009b9fa0` | `ObjectiveUnhidePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the objective-family 45/5 unhide packet: calls the packet destructor body 009aa910 (destructor of a member owning RewardBundleBase, per its accepted label). Then frees the object through _… |
