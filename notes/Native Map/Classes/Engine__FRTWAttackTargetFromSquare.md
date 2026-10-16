# Class: Engine::FRTWAttackTargetFromSquare

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256280` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWAttackTargetFromSquare::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012562f0` | `Engine::FRTWAttackTargetFromSquare::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name AttackTargetFromSquare; no arguments or state. |
| `01256310` | `Engine::FRTWAttackTargetFromSquare::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the AttackTargetFromSquare registration tag (name, version 0, source file and line); no arguments or state. |
| `01256320` | `Engine::FRTWAttackTargetFromSquare::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
