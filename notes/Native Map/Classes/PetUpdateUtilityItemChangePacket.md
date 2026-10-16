# Class: PetUpdateUtilityItemChangePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b545f0` | `PetUpdateUtilityItemChangePacket::PetUpdateUtilityItemChangePacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x15 (21), stores PetUpdateUtilityItemChangePacket::vftable, zeroes the dwords at +0x10/+0x14, constructs an embedded member through 0101cd40 and initial… |
| `00b546a0` | `PetUpdateUtilityItemChangePacket::~PetUpdateUtilityItemChangePacket` | ✅ | dtor | Destructor: releases the three embedded IStrings in the order +0x40, +0x30, +0x20 (each: rewrite the IString vtable, and when the string owns a buffer atomically decrement the counter at data-4, calling the IString virt… |
| `00b54de0` | `PetUpdateUtilityItemChangePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor 00b546a0 (three IString releases plus base vtable reset) and then calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe R… |
