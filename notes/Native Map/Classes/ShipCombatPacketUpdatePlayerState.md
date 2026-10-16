# Class: ShipCombatPacketUpdatePlayerState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca3ab0` | `ShipCombatPacketUpdatePlayerState::Ctor` | ✅ | ctor | Calls the base packet constructor with sub-opcode 0x11, installs the UpdatePlayerState vtable, and clears its payload words and flag. |
| `00ca3b50` | `ShipCombatPacketUpdatePlayerState::ScalarDeletingDtor` | ✅ | dtor | Restores the packet base vtables, frees the allocation when the low deleting flag is set, and otherwise returns the packet pointer. |
