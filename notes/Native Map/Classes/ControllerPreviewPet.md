# Class: ControllerPreviewPet

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b03550` | `ControllerPreviewPet::PreserveVec3AndCallBase ⚠downgrade` | 🟢 | virtual | Override of the shared ControllerRoot vtable slot 53 for the pet-preview controller: reads the three dwords at this+0x164, +0x168 and +0x16c into locals, calls the base implementation FUN_0094d3b0(param_2) with the same… |
| `00b035c0` | `ControllerPreviewPet::ControllerPreviewPet` | ✅ | ctor | Constructor: runs the base constructor FUN_00950530(param_2,param_3,param_4), stores the globals DAT_0181be70, DAT_0179edb8 and DAT_0181cec0 at +0x168, +0x164 and +0x16c, installs ControllerPreviewPet::vftable, zeroes t… |
| `00b036b0` | `ControllerPreviewPet::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor held in vtable slot 7, the slot this controller hierarchy uses for it: writes ControllerPreviewPet::vftable, calls the base destructor body FUN_009509a0 (ControllerRoot), and when the low bit … |
