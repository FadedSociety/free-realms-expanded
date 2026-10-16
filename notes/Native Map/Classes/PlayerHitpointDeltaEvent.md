# Class: PlayerHitpointDeltaEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b81e0` | `PlayerHitpointDeltaEvent::PlayerHitpointDeltaEvent` | ✅ | ctor | Calls PlayerEvent base construction, installs PlayerHitpointDeltaEvent::vftable, and stores one dword, one byte, and two dwords at offsets +0x50 through +0x5c. |
| `008b8280` | `PlayerHitpointDeltaEvent::ScalarDeletingDtor` | ✅ | dtor | Calls the derived cleanup routine and frees the object when the deleting flag bit is set. |
| `00ae2130` | `PlayerHitpointDeltaEventBaseDtor` | ✅ | dtor | Writes PlayerHitpointDeltaEvent then PlayerEvent vtables and invokes the PlayerEvent base cleanup helper. |
| `00ae2350` | `PlayerHitpointDeltaEvent::Apply` | ✅ | virtual | When enabled and player context exists, formats the signed delta, emits a hitpoint change, checks current hitpoint pair against event values and invokes a secondary context update, then calls the base follow-up slot. |
