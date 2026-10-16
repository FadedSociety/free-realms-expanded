# Class: EncounterMatchmaking::MatchmakingRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a82b30` | `EncounterMatchmaking::MatchmakingRequestDtor` | ✅ | dtor | Destructor body for one MatchmakingRequest: stores the MatchmakingRequest vtable, calls 0070a790 with ECX set to the embedded member at this+0x64, then calls 00a82200 with ECX set to the embedded hash-container member a… |
| `00a82ba0` | `EncounterMatchmaking::MatchmakingRequestScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of MatchmakingRequest: runs the destructor body 00a82b30 and frees the object with _free when the delete flag bit 0 is set. |
| `00a82f10` | `MatchmakingRequestDeletingDtorCopy` | 🟢 | dtor | Second deleting-destructor-shaped copy of the MatchmakingRequest teardown: an SEH-framed wrapper that calls the destructor body 00a82b30 and then frees the object with _free when the delete flag bit 0 is set; it has no … |
| `00a83360` | `EncounterMatchmaking::MatchmakingRequestCopyCtor` | ✅ | ctor | Copy constructor of one matchmaking request node: stores the MatchmakingRequest vtable, copies the scalar fields at +8..+0x20 from the source, copy-constructs the embedded SoeUtil::HashList<EncounterParticipant> at +0x2… |
| `00a83620` | `MatchmakingRequestCopyCtorWrapper` | 🟢 | ctor | Thin SEH-framed wrapper that copy-constructs a MatchmakingRequest at param_1 from the source param_2 by calling 00a83360 and returns param_1; used by the list append path when a request is copied into a node. |
