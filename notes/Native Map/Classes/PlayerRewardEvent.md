# Class: PlayerRewardEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8a4d0` | `PlayerRewardEvent::PlayerRewardEvent` | ✅ | ctor | Constructor of the player reward event: runs the base constructor FUN_008b8160, installs PlayerRewardEvent::vftable and constructs the embedded ClientRewardBundle copy through 00b8a460. |
| `00b8a560` | `PlayerRewardEvent::~PlayerRewardEvent` | ✅ | dtor | Destructor body of the player reward event: resets to PlayerRewardEvent::vftable, destroys the embedded ClientRewardBundle via FUN_0103d010, resets to PlayerEvent::vftable and runs the event base teardown FUN_007e7980. |
| `00b8a5d0` | `PlayerRewardEvent_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor of the player reward event (vtable slot 3): runs the destructor body 00b8a560 and frees the event through _free when bit 0 of param_2 is set. |
| `00b8a5f0` | `— ⚠reject` | ⚪ | virtual | Vtable slot 1 of PlayerRewardEvent: with bit 0 of the argument set it resolves the local player through FUN_00ae2470, and when the player exists and has a non-null object at +0x738 it presents the embedded reward bundle… |
