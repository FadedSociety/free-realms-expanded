# Class: EncounterMatchmaking::MatchmakingQueueDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7ef70` | `EncounterMatchmaking::MatchmakingQueueDefinition::~MatchmakingQueueDefinition` | ✅ | dtor | Seven-byte destructor: the whole body is mov dword [ecx], EncounterMatchmaking::MatchmakingQueueDefinition::vftable; ret - it only rewrites the object's vtable pointer. It is reached exclusively from MSVC EH unwind func… |
| `00a7f0a0` | `EncounterMatchmaking::MatchmakingQueueDefinition::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: rewrites the object's vtable pointer to MatchmakingQueueDefinition::vftable and frees the object with _free when bit 0 of param_2 is set. |
| `00a994c0` | `EncounterMatchmaking::MatchmakingQueueDefinitionInit` | 🟢 | ctor | Moves its this argument into ESI and calls the MatchmakingQueueDefinition constructor 0106e850 with this in ECX, then returns this. The only value the decompiler shows at the call site is the pushed SEH cookie, which is… |
