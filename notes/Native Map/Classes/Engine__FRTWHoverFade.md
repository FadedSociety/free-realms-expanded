# Class: Engine::FRTWHoverFade

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256750` | `Engine::FRTWHoverFade::FRTWHoverFade` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWHoverFade::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012567c0` | `Engine::FRTWHoverFade::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name HoverFade; no arguments or state. |
| `012567e0` | `Engine::FRTWHoverFade::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the HoverFade registration tag (name, version 0, source file and line); no arguments or state. |
| `012567f0` | `Engine::FRTWHoverFade::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01256850` | `Engine::FRTWHoverFade::VTableSlot2` | 🟢 | virtual | Vtable slot [2] body (security-cookie frame): calls FUN_012eb290(ElementID literal, 0) to get an ElementID value, then FUN_013276b0(0xe8) and FUN_013275f0 with arguments 0, 0, 0x5f623f0, ElementID, 1, then FUN_013275d0(… |
