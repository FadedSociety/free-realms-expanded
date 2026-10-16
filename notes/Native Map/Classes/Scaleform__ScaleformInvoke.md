# Class: Scaleform::ScaleformInvoke

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01250840` | `Scaleform::ScaleformInvoke::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Scaleform::ScaleformInvoke::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `012508b0` | `Scaleform::ScaleformInvoke::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Scaleform::ScaleformInvoke::vftable. Returns the constant C string 'Invoke'. |
| `012508d0` | `Scaleform::ScaleformInvoke::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'Invoke : 0 : ..\common\coreclient\scaleform\scaleformhandlers.cc:43'. |
| `012508e0` | `Scaleform::ScaleformInvoke::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01250a90` | `Scaleform::ScaleformInvoke::InvokeMethodByName ⚠downgrade` | 🟢 | virtual | Slot [2] of Scaleform::ScaleformInvoke::vftable, 226 bytes. Calls FUN_012e0ba0 and FUN_00f9c010; if that returns an object it reads the 'MethodName' value through FUN_012eb250, copies it into a std::string and calls FUN… |
