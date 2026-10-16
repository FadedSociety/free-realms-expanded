# Class: SoeUtil::Array<DeepParticle::RandomFrameController::Probability,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00883200` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1]. Args: count [esp+4], out-capacity pointer [esp+8], mode byte [esp+0xc]. Mode 0: if capacity (this+0xc) < count, new capacity = ceil(count*5/4); if capacity > count*4/3, new capacity = count*6/5; otherw… |
| `008832a0` | `ReleaseHeapBuffer` | ✅ | virtual | Virtual slot [2]; ECX is unused. Calls FUN_00401c00(ptr, 4) on the stack pointer at [esp+4] with no inline-buffer check. Epilogue is ret 8. |
| `008832c0` | `SwapHeaderOrForwardToPeer` | ✅ | virtual | Virtual slot [3]. Peer object is the stack dword [esp+4]; flag is the byte [esp+8]. If flag is nonzero, tail-jumps to the peer's vtable slot [3] with ECX=peer and args (this, 0). Otherwise swaps this+4 with peer+4 and X… |
| `00883700` | `Array::~Array` | ✅ | dtor | Fastcall (ECX=this). Stores the SoeUtil::Array<DeepParticle::RandomFrameController::Probability,0,1> vtable, sets count (+8) to 0, calls FUN_00401c00(buffer,4) on the buffer at +4, sets the buffer to 0. Ret 0. |
| `00883900` | `Array::ScalarDeletingDtor` | ✅ | dtor | Thiscall with flag byte [esp+4]. Stores the Array<Probability,0,1> vtable, sets count to 0, calls FUN_00401c00(buffer,4) on the buffer at +4, sets buffer to 0; if flag bit 0 is set calls _free(this). Returns this. |
