# Class: DeepParticle::RandomFrameController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00884230` | `— ⚠reject` | ⚪ | ctor | Thiscall constructor, three stack args (the reader object and two others); ret 0xc. Calls FUN_008839d0 on this, stores DeepParticle::RandomFrameController::vftable, calls FUN_00883f40 and FUN_00883fa0 on local array mem… |
| `00884560` | `— ⚠reject` | ⚪ | dtor | Fastcall (ECX=this), no arguments. Calls FUN_00883640 on this (the SoeUtil::Array<unsigned char,128,1> destructor body), then stores DeepParticle::ParticleController::vftable at [this]. Returns void. |
| `008845c0` | `RandomFrameController::ScalarDeletingDtor` | ✅ | dtor | Thiscall with flag byte [esp+8]. Calls FUN_00884560 on this (its destructor body, analysed separately), then if flag bit 0 is set calls _free(this). Returns this. Ret 4. |
