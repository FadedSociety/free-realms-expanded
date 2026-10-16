# Class: SoccerProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b759e0` | `SoccerProcessor::SetTeamObject` | ✅ | virtual | Virtual slot 1 of SoccerProcessor: picks the team slot (+0x618 for team 0, +0x630 for team 1, returning 1 immediately for other indices), releases the previous object at slot+0x10 (terminal hook vtable +0x3c when refcou… |
| `00b80080` | `SoccerProcessor::Ctor` | ✅ | ctor | Installs AssetRequester then SoccerProcessor vtables; initializes team fields, strings, status and pickup lists, arrays, flags and timers; assigns bbe_goal_made.xml and creates three referenced objects. |
| `00b807c0` | `SoccerProcessor::Dtor` | ✅ | dtor | Restores the SoccerProcessor vtable, releases held objects and sound slots, destroys pickup/status lists and string members, releases three referenced objects, and restores the AssetRequester base vtable. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | ✅ | virtual | Virtual slot [4] (+0x10) of SoccerProcessor::vftable. Takes a buffer range (param_2..param_3), builds a BaseSoccerPacket temporary with FUN_00b73040, reads the two-word inner header with FUN_00b77fc0 and switches on the… |
| `00b83330` | `SoccerProcessor::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor. Calls the dtor body FUN_00b807c0 on ECX (this), then calls _free on this when bit 0 of the flag argument is set, and returns this. |
