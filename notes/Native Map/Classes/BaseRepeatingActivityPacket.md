# Class: BaseRepeatingActivityPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad9620` | `BaseRepeatingActivityPacket::BaseRepeatingActivityPacket` | ✅ | ctor | Base constructor for the repeating-activity packet family: installs BasePacket::vftable, writes 0x8f (143) into the 2-byte packet id slot at +4, installs BaseRepeatingActivityPacket::vftable and stores the caller's acti… |
| `00ad9eb0` | `BaseRepeatingActivityPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for BaseRepeatingActivityPacket (slot 0 of its vtable): in an SEH frame it resets the vtable to BasePacket::vftable, then calls _free(this) when bit 0 of the flag argument is set. |
