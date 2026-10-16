# Class: PetListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b52f90` | `PetListPacket::PetListPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 5, stores PetListPacket::vftable, zeroes the dwords at +0xc through +0x28, writes the sentinel 0xffffffff at +0x2c and writes the shared default pair DAT… |
| `00b59670` | `PetListPacket::~PetListPacket` | ✅ | dtor | Installs the PetListPacket vtable, destroys its linked-list members, then restores BasePacket vtable. |
| `00b596d0` | `PetListPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the PetListPacket destructor and frees the object only when param_2 bit 0 is set. |
