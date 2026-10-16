# Class: GaqItemGroupAnnounceEncounter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09610` | `GaqItemGroupAnnounceEncounter_ForwardToSlot0` | ✅ | virtual | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the encounter responder that runs the 00a97720/00a979a0 join-or-leave path with status 0. |
| `00a09650` | `GaqItemGroupAnnounceEncounter_Respond0` | ✅ | virtual | Virtual method: fetches the global singleton, clears the dword at client+0x6F0, then compares the item's two parameters with the globals DAT_01be65c0/DAT_01be65c4; on a match it calls FUN_00a97720(0), otherwise FUN_00a9… |
| `00a096a0` | `GaqItemGroupAnnounceEncounter_Respond1` | ✅ | virtual | Virtual method identical to the slot-0 responder but passing status 1 to FUN_00a97720 and FUN_00a979a0. |
