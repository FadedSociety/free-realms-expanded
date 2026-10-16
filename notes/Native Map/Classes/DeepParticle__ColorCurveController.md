# Class: DeepParticle::ColorCurveController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00886420` | `ColorCurveController::ctor` | ✅ | ctor | Thiscall constructor, three stack args; ret 0xc. EH frame. Calls FUN_0087aa60 (base constructor; receiver not decoded from the bytes), stores DeepParticle::ColorCurveController::vftable, sets the 16-bit fields at this+0… |
| `008866c0` | `ColorCurveController::EvalColorRange` | ✅ | virtual | Thiscall virtual, slot [6]. Stack args: owner (param_2), begin (param_3), end (param_4). Calls FUN_00886320 on the owner's colour cache to get a 16-float cubic block, copying it to a local. Clamps a per-index time from … |
