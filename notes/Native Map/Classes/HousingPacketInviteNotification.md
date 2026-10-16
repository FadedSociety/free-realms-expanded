# Class: HousingPacketInviteNotification

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ab3be0` | `HousingPacketInviteNotification::Ctor` | ✅ | ctor | Constructs housing subtype 47, installs the invite-notification vtable, and initializes the embedded payload fields. |
| `00ab3c40` | `HousingPacketInviteNotification::ScalarDeletingDtor` | ✅ | dtor | Calls packet cleanup and conditionally frees the object when the deleting flag's low bit is set. |
| `00abd510` | `HousingPacketInviteNotification_ReadBody` | 🟢 | function | Builds a bounded reader over input data/length, constructs a local invite-notification object through FUN_00ab3be0, parses it through FUN_00ab8bd0, dispatches it through FUN_00a0bdb0 only when the error byte is clear an… |
