# Class: PacketClientGameSettings

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089bc20` | `PacketClientGameSettingsCtor` | ✅ | ctor | Constructor of PacketClientGameSettings: installs BasePacket::vftable, stores opcode 0x90 (144) at +4, installs the derived vtable, clears +0x10, sets the byte at +0x14 to 1 and copies the shared dword _DAT_0175b420 int… |
| `008b1670` | `PacketClientGameSettings::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientGameSettings::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
