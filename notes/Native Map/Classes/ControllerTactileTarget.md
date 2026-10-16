# Class: ControllerTactileTarget

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00afdcf0` | `ControllerTactileTarget::ForwardSlot71ThenHelper ?` | 🟡 | virtual | Thiscall with one arg, vtable slot 51 (+0xcc): calls this->vtable[+0x11c](arg) then 0094c9d0(arg). Plain function of two calls. |
| `00afe290` | `ControllerTactileTarget::~ControllerTactileTarget` | ✅ | dtor | Fastcall dtor body (ECX=this, ret 0): stores ControllerTactileTarget::vftable at +0x00, releases the IString members at +0x368 and +0x378 by decrementing their reference counts and calling their vtable slot 2 when the c… |
| `00afe470` | `ControllerTactileTarget::ControllerTactileTarget` | ✅ | ctor | Thiscall ctor (ECX=this), three stack args forwarded to base 00950530, returns this (ret 0x14). Stores ControllerTactileTarget::vftable at +0x00, calls 00afe130 on the sub-objects at this+0x1a0, +0x238 and +0x2d0, insta… |
| `00afe5b0` | `ControllerTactileTarget::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting dtor shape (thiscall, one flag byte arg, ret 4): calls the dtor body 00afe290 on this, then calls _free(this) (00d82463) when the flag bit 0 is set. Returns this. |
| `00afe5d0` | `ControllerTactileTarget::UpdatePointerTrackers ?` | 🟡 | virtual | Thiscall with one stack arg (dt), vtable slot 50 (+0xc8). Reads the flags word at +0x2c and calls 00afe3a0 three times, passing dt, a flag bit derived from +0x2c (bit 3 for the first call, bit 5 and bit 4 for the next t… |
| `00afe670` | `ControllerTactileTarget::UpdateNearestHitTarget ?` | 🟡 | virtual | Fastcall, vtable slot 71 (+0x11c) on ControllerTactileTarget (also referenced at ControllerPet slot 71). Returns unless 0094d650() gives an object whose vtable +0x24 is nonzero. Clears dword this+0x388, then walks a can… |
| `00afebe0` | `ControllerTactileTarget::FormatTrackerDebugText` | ✅ | virtual | Thiscall with no args, vtable slot 42 (+0xa8). On first use sets a static guard bit (DAT_01be8e78 bit 0), initialises the static IString DAT_01be8e68 (vtable SoeUtil::IString<char>, empty buffer DAT_01be8e64, count 0) a… |
