# Class: PetTrickSkillLevelPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51020` | `PetTrickSkillLevelPacket::PetTrickSkillLevelPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0xc (12), stores PetTrickSkillLevelPacket::vftable and zeroes three dwords at +0xc/+0x10/+0x14 plus the byte at +0x18. |
| `00b51090` | `PetTrickSkillLevelPacket::~PetTrickSkillLevelPacket` | ✅ | dtor | Destructor: stores PetTrickSkillLevelPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b510e0` | `PetTrickSkillLevelPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickSkillLevelPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
