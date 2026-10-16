# Class: SoccerBall

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cdcda0` | `SoccerBall::CleanupAttachedHandles` | ✅ | virtual | Calls 0107ba90, adjusts ECX to this, then tail-jumps to 00cdcbe0 for the five-handle cleanup. |
| `00cdcdb0` | `SoccerBall::SetAttachedStateBits` | ✅ | virtual | Gets a component through this+0xc0, updates bits 0 and 1 in its word at +0x10 from param_2, invokes the matching virtual callbacks, and forwards param_2 to 0107bb60. |
| `00cdcfd0` | `SoccerBall::EnsureHandle164` | ✅ | virtual | When handle +0x164 is empty and the descriptor at +0x160 is ready, obtains its resource id, creates and stores a handle, then invokes handle vtable slot 7. |
| `00cdd100` | `SoccerBall::UpdateFiveEffectSlots ? ⚠rename` | 🟡 | virtual | Calls 0107ba30, then for positive counts at five slots submits each slot id, its type from 2 through 6, and this+0x114 to the effect manager. |
| `00cdd200` | `— ⚠reject` | ⚪ | virtual | Copies sixteen dwords from param_2; when param_3 is zero, blends three matrix columns with 0107bb00 using 1.0 minus this+0x128, then applies the matrix through the object at this+0xc0. |
| `00cdd300` | `— ⚠reject` | ⚪ | virtual | Updates ball effects from state and threshold checks, normalizes and smooths motion fields at +0x68 through +0x6b, then synchronizes dependent transforms through 00cdd040. |
| `00cdd9e0` | `SoccerBall::DestructorBody` | ✅ | dtor | Restores SoccerBall subobject vtables, releases attached handles and six embedded strings, destroys the asset requester and base object, and runs shared cleanup helpers. |
| `00cddde0` | `SoccerBall::FillFourDwordOutput → SoccerBall::GetPositionVec4` | ✅ | virtual | Gets the nested object through this+0xc0; fills four output dwords from its fields, or delegates to 007652e0 when its +0x1a0 field is set, with a fallback when absent. |
| `00cdde90` | `SoccerBall::FillThreeDwordOutput → SoccerBall::GetPositionVec3` | ✅ | virtual | Gets the nested object through this+0xc0; fills three output dwords from its fields, or delegates to 007652e0 when its +0x1a0 field is set, with a fallback when absent. |
| `00cddf40` | `SoccerBall::ScalarDeletingDestructor` | ✅ | dtor | Calls the SoccerBall destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |
| `00cde1c0` | `SoccerBall::Constructor` | ✅ | ctor | Initializes the proxied base, installs SoccerBall and embedded subobject vtables, stores constructor pointers, zeros handle/state fields, initializes embedded strings, and assigns five effect asset names. |
| `00cde9b0` | `SoccerBall::ApplyOrSendImpulse` | ✅ | virtual | Calls the base soccer-ball method, selects a component handle by param_5 and invokes its virtual operation; when param_6 is nonzero, fills the 36-byte SoccerPacketImpulseBall payload and submits it. |
