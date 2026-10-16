# Class: ControllerDressingRoom

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00985790` | `ControllerDressingRoom_DestructBody` | ✅ | dtor | Destructor body of ControllerDressingRoom under SEH frame LAB_01599ec8: stores the ControllerDressingRoom vftable into [param_1], parks the object on the stack and calls the base teardown 0094fab0; returns through the f… |
| `009858d0` | `ControllerDressingRoom::ControllerDressingRoom` | ✅ | ctor | Constructor under SEH frame LAB_01599ef8: calls the base constructor 0094f6c0(param_2,param_3,param_4), installs the ControllerDressingRoom vftable, calls 0094c8a0(0), zeroes the bytes at object+0x130, +0x13c..+0x141, a… |
| `00985990` | `ControllerDressingRoom_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor registered in slot [7] of ControllerDressingRoom::vftable: under an SEH frame it stores the class vftable into [param_1], calls the base teardown 0094fab0, and calls the _free thunk 00d82463 o… |
| `00985ce0` | `ControllerDressingRoom_CollectEventFlags` | ✅ | virtual | Virtual method in slot [50] of ControllerDressingRoom::vftable: walks the list at [this+0xc8] through the node link at +4, and for every event node whose [node->+0x18] field is 0 and whose byte at +0xd is non-zero, swit… |
| `00985ed0` | `ControllerDressingRoom_UpdateMove` | ✅ | virtual | Per-frame continuation of the move started by 00985a10 (virtual slot [51] of the dressing-room vtable): when the accumulated time at +0x138 is below the duration at +0x134 it computes the normalised fraction, evaluates … |
