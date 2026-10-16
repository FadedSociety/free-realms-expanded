# Class: EncounterMatchmaking::AddMatchRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c08790` | `— ⚠reject` | ⚪ | ctor | Matchmaking request constructor: calls the matchmaking base constructor FUN_00a7f0d0(3), installs EncounterMatchmaking::AddMatchRequestPacket::vftable, then constructs the embedded member at +0xc through FUN_0106f3f0. |
| `00c08850` | `AddMatchRequestPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of EncounterMatchmaking::AddMatchRequestPacket::vftable: runs the shared teardown 00c087f0 and calls _free on the object when bit 0 of the flag byte argument is set. |
