# Class: ControllerPet

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00afd610` | `ControllerPet::~ControllerPet` | ✅ | dtor | Fastcall dtor body (ECX=this, ret 0): stores ControllerPet::vftable at +0x00, sets the unwind state to -1, and calls 00afe290 on this (base teardown). |
| `00afd680` | `ControllerPet::GetByte3b4 ?` | 🟡 | virtual | Thiscall getter, no args, vtable slot 72 (+0x120): returns the byte at this+0x3b4 (mov al,[ecx+0x3b4]; ret). |
| `00afd690` | `ControllerPet::SetByte3b4 ?` | 🟡 | virtual | Thiscall setter with one byte arg (ret 4), vtable slot 73 (+0x124): stores the arg byte into this+0x3b4. |
| `00afd6c0` | `ControllerPet::InvokeSlot53ThenSlot45 ?` | 🟡 | virtual | Thiscall, vtable slot 46 (+0xb8). Calls this->vtable[+0xd4] with one stack arg this+0xe0, then this->vtable[+0xb4] with one stack arg 0. |
| `00afd6f0` | `ControllerPet::ControllerPet` | ✅ | ctor | Thiscall ctor, 3 forwarded args to base 00afe470 then returns this (ret 0x14). Stores ControllerPet::vftable at +0x00, zeroes dword +0x3b0, sets byte +0x3b4 to 1 and byte +0x3b5 to 0, calls 00951b40(0x20) on this, sets … |
| `00afd7b0` | `ControllerPet::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting dtor shape (thiscall, one flag byte arg, ret 4): stores ControllerPet::vftable, sets unwind state -1, calls 00afe290 on this, then calls _free(this) (00d82463) when the flag bit 0 is set. Returns this. |
| `00afd810` | `ControllerPet::DistanceOfFourFloatsToSlot5c ?` | 🟡 | virtual | Thiscall with one arg, vtable slot 53 (+0xd4). Calls 0094d3b0(arg), this->vtable[+0x70](DAT_017eb654), this->vtable[+0xdc](1), 008a6660(), 00b536e0() to get an object whose vtable[+0x24] fills a 4-float out buffer, take… |
| `00afd8d0` | `ControllerPet::ForwardListNodes_c8 ?` | 🟡 | virtual | Thiscall with one arg, vtable slot 50 (+0xc8): calls 00afe5d0(arg), then this->vtable[+0x128](arg), then walks the singly linked list headed at this+0xc8 (next pointer at node+0x04), calling 0094c8b0() on each node and … |
| `00afd920` | `— ⚠reject` | ⚪ | virtual | Thiscall with one int arg (time step), vtable slot 74 (+0x128). Calls 0094d650() and returns if null. Updates the counter at +0xec by the arg and flag byte +0x3b5 from flags +0x219, +0x85 and +0xec (thresholds 0x1f4, 0x… |
