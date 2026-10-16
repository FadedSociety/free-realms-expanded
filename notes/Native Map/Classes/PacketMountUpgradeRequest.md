# Class: PacketMountUpgradeRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bff390` | `PacketMountUpgradeRequest::Ctor` | ✅ | ctor | Calls the mount-family packet base constructor with sub-opcode 14, installs PacketMountUpgradeRequest::vftable, and initializes two fields to -1. |
| `00bff4a0` | `PacketMountUpgradeRequest::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and calls _free(this) only when flag bit 0 is set; otherwise returns this. |
