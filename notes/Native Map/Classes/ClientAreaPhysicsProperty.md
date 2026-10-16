# Class: ClientAreaPhysicsProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be9840` | `ClientAreaPhysicsProperty::Ctor` | ✅ | ctor | Stores ClientAreaProperty vtable and initialises the base fields as in ClientAreaRoomTypeProperty::Ctor (arguments at +0x18 and +0x1c, constant _DAT_0175b420 at +0x24), then installs ClientAreaPhysicsProperty vtable and… |
| `00be9930` | `ClientAreaPhysicsProperty::DtorBody` | ✅ | dtor | Stores the ClientAreaPhysicsProperty vtable at *this, calls 00ceb240(0) with an unwind frame, then stores the ClientAreaProperty vtable at *this and returns. |
| `00be9990` | `ClientAreaPhysicsProperty::NameHash` | ✅ | virtual | Same body as 00be9240 (byte-identical except the rel32 displacement of the 00705bc0 call): calls vtable slot +0x28 on this, computes the one-at-a-time hash of the returned string up to 0x31 characters with the same fina… |
| `00be9f40` | `ClientAreaPhysicsProperty::GetTypeName` | ✅ | virtual | Returns a pointer to the literal string 'Physics' (mov eax, offset; ret). |
| `00be9f50` | `ClientAreaPhysicsProperty::ScalarDtor` | 🟢 | dtor | Calls 00be9930 (physics destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00ceaf20` | `ClientAreaPhysicsProperty::Log ?` | 🟡 | virtual | Logs inherited property fields plus JumpHeightScalar, GravityScalar, BounceScalar, BounceVelocityCap, and AutoJump. |
| `00ceb120` | `ClientAreaPhysicsProperty::Load ?` | 🟡 | virtual | Loads four named physics values with atof, reads AutoJump as a boolean, conditionally updates a live object, and clamps a nested value to 250. |
| `00ceb240` | `ClientAreaPhysicsProperty::Reset ?` | 🟡 | virtual | For an active object with type 1, resets selected physics fields at +0x490/+0x498 to the default float and zeros +0x49c/+0x4a0 when property values are non-default. |
| `00ceb2d0` | `ClientAreaPhysicsProperty::Apply ?` | 🟡 | virtual | For an active object of type 1, copies physics properties to it; if AutoJump is enabled, sets bit 0x10 at object+0x142 and calls a state helper. |
