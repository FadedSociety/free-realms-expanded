# Class: KeyCodeRedemptionNotificationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a05910` | `KeyCodeRedemptionNotificationPacket::KeyCodeRedemptionNotificationPacket` | 🟢 | ctor | Constructor: runs the base constructor 00a057e0 with sub-opcode 1, stores KeyCodeRedemptionNotificationPacket::vftable and stores the caller's argument at param_1[3]. |
| `00a05980` | `KeyCodeRedemptionNotificationPacket::~KeyCodeRedemptionNotificationPacket` | ✅ | dtor | Destructor body: rewrites the vtable to KeyCodeRedemptionNotificationPacket::vftable, then BasePromotionalPacket::vftable, then BasePacket::vftable; called from one unwind funclet. |
| `00a059e0` | `KeyCodeRedemptionNotificationPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor in an SEH frame: restores KeyCodeRedemptionNotificationPacket::vftable, BasePromotionalPacket::vftable and BasePacket::vftable, then frees this when the delete flag is set. |
