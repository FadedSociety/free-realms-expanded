# Class: EncounterCompleteEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa1e40` | `EncounterCompleteEvent::EncounterCompleteEvent` | ✅ | ctor | Constructor taking (a, byte, b, c): calls the GameEvent base constructor FUN_007e78e0, installs EncounterCompleteEvent::vftable at +0, constructs two member objects through FUN_0091c110(0,0) plus FUN_0103a1e0 (the latte… |
| `00aa1f10` | `EncounterCompleteEvent::~EncounterCompleteEvent` | ✅ | dtor | Destructor body: rewrites the EncounterCompleteEvent vtable at +0, destroys the two member objects through FUN_0103d010 (called twice, second under the SEH state change) and finally calls the GameEvent base destructor F… |
| `00aa1f90` | `EncounterCompleteEvent::Apply` | ✅ | virtual | Virtual slot 1 override (the GameEvent base slot returns 0 without doing anything): when bit 0 of its flag argument is set it calls FUN_00769710 to obtain the client singleton, then calls FUN_009bf4d0 with the target ob… |
| `00aa1fd0` | `EncounterCompleteEvent::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00aa1f10 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
