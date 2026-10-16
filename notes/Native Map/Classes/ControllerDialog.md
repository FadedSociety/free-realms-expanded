# Class: ControllerDialog

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00afecc0` | `ControllerDialog::StoreTwoDwords_174 ? ⚠rename` | 🟡 | virtual | Thiscall with two dword args (ret 8), vtable slot 54 (+0xd8): stores the first arg at this+0x174 and the second at this+0x178. |
| `00afece0` | `ControllerDialog::ResolvePairAndNotify ?` | 🟡 | virtual | Thiscall with one flag arg, vtable slot 8 (+0x20). Calls 00769710() and reads the pointer at +0x04 and +0xc0 of the result; when both are set, looks up the pair at this+0xe8/+0xec through 00984960 to get an object (via … |
| `00afee40` | `ControllerDialog::SetClampedFloat17c ?` | 🟡 | virtual | Thiscall with one float arg (ret 4), vtable slot 23 (+0x5c). Clamps the arg into the range [DAT_017ea718, DAT_017ed82c] and stores it at this+0x17c. |
| `00afee70` | `ControllerDialog::DispatchHotKeyToggle` | ✅ | virtual | Fastcall, vtable slot 50 (+0xc8). Walks a list headed at this+0xc8 (element at node+0x00, next at node+0x04). For an element with type 0 (+0x18 == 0) and flag +0x0d set it reads action code +0x20: code 0x1d clears the f… |
| `00afef60` | `ControllerDialog::RequestIfPairPredicateFails ?` | 🟡 | virtual | Thiscall with no args (plain ret, bytes checked), vtable slot 34 (+0x88): calls 00afede0() on this and when it returns false calls 00769710(), then 008c5d90(1) with ECX set to the dword at the returned pointer +0x04. |
| `00afef80` | `ControllerDialog::UpdateViewAndCollision ?` | 🟡 | virtual | Fastcall, vtable slot 51 (+0xcc) on ControllerDialog. Reads the target pair at this+0xe0/+0xe4 and, when it is the sentinel DAT_01be8f30/DAT_01be8f34, uses the object from 0094cf60 (camera-like matrix at +0x610..+0x61c … |
| `00affb10` | `ControllerDialog::~ControllerDialog` | ✅ | dtor | Fastcall dtor body (ECX=this, ret 0): stores ControllerDialog::vftable at +0x00, sets the unwind state, calls 00769710, then 00984960 to find an entity, and sets byte +0x226 to 1 on that entity's vtable +0x24 object, th… |
| `00affc70` | `ControllerDialog::GetFloat17c ?` | 🟡 | virtual | Thiscall getter, no args, vtable slot 24 (+0x60): returns the float at this+0x17c. |
| `00affc80` | `ControllerDialog::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting dtor shape (thiscall, one flag byte arg): calls dtor body 00affb10 on this, then calls _free(this) (00d82463) when the flag bit 0 is set. Returns this. |
| `00affca0` | `ControllerDialog::SetTargetPair ?` | 🟡 | virtual | Thiscall, vtable slot 53 (+0xd4), with the pair (param_2, param_3) and a string arg param_4 (assigned with 00402ce0 when nonzero). When the pair differs from this+0xe0/+0xe4: looks up the old target (00769710/00984960 o… |
| `00affeb0` | `ControllerDialog::ControllerDialog` | ✅ | ctor | Thiscall ctor (ECX=this) with five stack args, returns this. Calls base 0094f6c0 with the forwarded args, stores ControllerDialog::vftable at +0x00, sets the target pairs at +0xe0/+0xe4 and +0xe8/+0xec to the sentinel D… |
