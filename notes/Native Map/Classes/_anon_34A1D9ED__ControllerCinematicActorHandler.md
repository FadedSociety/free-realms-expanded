# Class: _anon_34A1D9ED::ControllerCinematicActorHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6f540` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ApplyTimeAndNotifyEntries ?` | 🟡 | virtual | Virtual slot 1 of the cinematic actor handler. Clamps the float returned by 007707f0 to at least the constant DAT_01815b84 and stores it at +0x98 of the object at param_2+0x2b0 when that pointer is non-null. Sets the gl… |
| `00a6f770` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ReleaseGlobalHandleAndClear ? …` | 🟡 | virtual | Virtual slot 2 (+8) of the cinematic actor handler: releases the global handle DAT_01be701c through the manager's +0xf0 object slot 2 (+8), then sets the global to 0. |
| `00a6f7b0` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ClearGlobalHandleIfEqual ?` | 🟡 | virtual | Virtual slot 13 (+0x34), 24 bytes: when the global DAT_01be701c equals param_1 it sets the global to 0; otherwise does nothing. |
| `00a6f850` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ReleaseNameString` | ✅ | function | Releases the IString<char> at +4 of the handler: stores the SoeUtil::IString<char> vtable there and, when the length at +0x10 is positive, drops a reference on the buffer at +8 (lock-prefixed decrement) and frees it thr… |
| `00a6f8d0` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ControllerCinematicActorHandler` | ✅ | ctor | Constructor: installs the ActorHandler vtable and then the ControllerCinematicActorHandler vtable at offset 0, embeds an IString<char> at +4 with empty buffer DAT_01be7034, and zeroes the fields at +0xc and +0x10. |
| `00a6f940` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls 00a6f850 (text member release), then frees this through _free when bit 0 of the flag argument is set; returns this. |
