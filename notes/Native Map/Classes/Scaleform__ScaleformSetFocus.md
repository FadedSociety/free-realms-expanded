# Class: Scaleform::ScaleformSetFocus

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01250950` | `Scaleform::ScaleformSetFocus::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Scaleform::ScaleformSetFocus::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `012509c0` | `Scaleform::ScaleformSetFocus::TypeName` | ✅ | virtual | Slot [1] of Scaleform::ScaleformSetFocus::vftable. Returns the constant C string 'SetFocus'. |
| `012509e0` | `Scaleform::ScaleformSetFocus::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'SetFocus : 0 : ..\common\coreclient\scaleform\scaleformhandlers.cc:29'. |
| `012509f0` | `Scaleform::ScaleformSetFocus::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01250a50` | `Scaleform::ScaleformSetFocus::ApplyFocus ⚠downgrade` | 🟢 | virtual | Slot [2] of Scaleform::ScaleformSetFocus::vftable, 62 bytes. Calls FUN_012e0ba0, then FUN_00f9c010; if it returns an object it takes FUN_012eb310('Focus', 0) and calls the object's virtual slot at +0x20 with that value;… |
