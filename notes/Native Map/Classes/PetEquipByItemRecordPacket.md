# Class: PetEquipByItemRecordPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50f30` | `PetEquipByItemRecordPacket::PetEquipByItemRecordPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x23 (35), stores PetEquipByItemRecordPacket::vftable, then initialises an embedded ItemRecord sub-object at +0xc: ItemRecord::vftable at +0xc with two z… |
| `00b50fa0` | `PetEquipByItemRecordPacket::~PetEquipByItemRecordPacket` | ✅ | dtor | Destructor: stores PetEquipByItemRecordPacket::vftable, resets the embedded ItemRecord at +0xc to ItemRecord::vftable, then stores BasePacket::vftable; one unwind funclet calls it. |
| `00b51000` | `PetEquipByItemRecordPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor 00b50fa0 (embedded ItemRecord reset plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argument is set. |
