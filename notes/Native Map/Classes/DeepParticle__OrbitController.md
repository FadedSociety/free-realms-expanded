# Class: DeepParticle::OrbitController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00886ea0` | `OrbitController::EvalOrbitRotate` | ✅ | virtual | Virtual slot [6] (ECX unused). Stack args: owner object (param_1; buffers read at +0x14/+0x18/+0x24/+0x34/+0x44 and a float at owner.+0xc.+0x20), begin (param_2), end (param_3). For each index in [begin,end) computes an… |
| `00887490` | `OrbitController::EvalOrbitRange` | ✅ | virtual | Thiscall virtual, slot [5]. Stack args: owner (param_2), begin (param_3), end (param_4). Per index: calls FUN_00886bf0 (RandomDirectionVec4) twice with the owner LCG state at owner.+0xc.+0x88, builds an orthonormal fram… |
