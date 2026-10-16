# Class: BaseAdminGuildPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b174c0` | `BaseAdminGuildPacket::BaseAdminGuildPacket ⚠downgrade` | 🟢 | ctor | Base constructor of the admin guild packet family: stores BasePacket::vftable, writes 0x83 (131) into the packet id slot at +4, installs BaseAdminGuildPacket::vftable and stores the sub-opcode argument at +8. |
| `00b17580` | `BaseAdminGuildPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of BaseAdminGuildPacket::vftable: under an exception frame it restores BaseAdminGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the fla… |
