# Class: Engine::FRTWHoverGlow

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256520` | `Engine::FRTWHoverGlow::FRTWHoverGlow` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWHoverGlow::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256590` | `Engine::FRTWHoverGlow::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name HoverGlow; no arguments or state. |
| `012565b0` | `Engine::FRTWHoverGlow::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the HoverGlow registration tag (name, version 0, source file and line); no arguments or state. |
| `012565c0` | `Engine::FRTWHoverGlow::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01256620` | `— ⚠reject` | ⚪ | virtual | Vtable slot [2] body (security-cookie frame): calls FUN_012eb290(ElementID literal, 0) to get an ElementID value, then FUN_013276b0(0xe5) and FUN_013275f0 with arguments 0, 0, 0x5f623f0, ElementID, 5, 0, then FUN_013276… |
