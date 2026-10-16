# Class: ControllerVehicleRace

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00afc510` | `ControllerVehicleRace::ControllerVehicleRace` | ✅ | ctor | Thiscall ctor returning this with ret 0x10 (4 stack args forwarded). Calls base ControllerVehicle::ControllerVehicle (00cc14b0) with those args, stores ControllerVehicleRace::vftable at +0x00 and at +0xe0, then calls 00… |
| `00afc5e0` | `ControllerVehicleRace::~ControllerVehicleRace` | ✅ | dtor | Fastcall dtor body (ECX=this, ret 0): restores ControllerVehicleRace::vftable at +0x00 and +0xe0, sets the unwind state to -1, calls 00cc1070 on this (ControllerVehicle destroy), then returns. |
| `00afc640` | `ControllerVehicleRace::ReadChain_74_11c_2cc ?` | 🟡 | virtual | Thiscall with no args, vtable slot 53 (+0xd4). Returns dword this->[+0x74]->[+0x11c]->[+0x2cc], or 0 when any link is null (tests at each step, then xor eax). Plain ret. |
| `00afc660` | `ControllerVehicleRace::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting dtor shape (ret 4, one flag byte arg at [esp+8]): calls the dtor body 00afc5e0 on this, then if flag bit 0 is set calls _free(this) (00d82463 thunk). Returns this in eax. |
| `00afc680` | `— ⚠reject` | ⚪ | virtual | Thiscall, one float arg (time step), vtable slot 50 (+0xc8). Walks a linked list headed at this+0xc8 (node: +0x00 element ptr, +0x04 next). Elements with type 0 and flag at +0x0d dispatch action codes 0x13-0x16 to 009cf… |
