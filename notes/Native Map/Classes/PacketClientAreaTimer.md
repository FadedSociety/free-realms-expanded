# Class: PacketClientAreaTimer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b1160` | `PacketClientAreaTimer::PacketClientAreaTimer` | ✅ | ctor | Initializes a PacketClientAreaTimer object: writes BasePacket::vftable, stores 0xac at this+4, zeroes this+8, installs PacketClientAreaTimer::vftable, stores -1 at this+0xc and copies DAT_01be4850/DAT_01be4854 into this… |
| `008b11d0` | `PacketClientAreaTimer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientAreaTimer::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
| `00ceb3d0` | `— ⚠reject` | ⚪ | ctor | Installs BasePacket then PacketClientAreaTimer vtables, stores 0xac size, two constructor arguments, and two global constants. |
| `00ceb500` | `PacketClientAreaTimer::SerializeFields ?` | 🟡 | method | Writes three fields from packet offsets +4/+8/+0xc into the supplied stream, clamps each write to remaining capacity, then serializes the embedded field at +0x10. |
| `00ceb610` | `— ⚠reject` | ⚪ | method | Serializes packet into a temporary writer, wraps it as a tunneled client packet, copies packet arguments into the wrapper, and calls a final packet processing helper. |
