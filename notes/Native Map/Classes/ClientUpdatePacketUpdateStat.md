# Class: ClientUpdatePacketUpdateStat

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ea870` | `ClientUpdatePacketUpdateStatCtor` | ✅ | ctor | Constructs base client-update packet with sub-opcode 7, installs derived vtable and initializes CharacterStat list. |
| `008ea8e0` | `ClientUpdatePacketUpdateStatDtor` | ✅ | dtor | Destroys embedded CharacterStat list then restores base packet vtables. |
| `008ea950` | `ClientUpdatePacketUpdateStat::ScalarDeletingDtor` | ✅ | dtor | Calls packet destructor, frees object if flag bit zero is set, otherwise returns pointer. |
