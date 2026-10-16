# Class: InspectProxy

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad66f0` | `InspectProxy::ScalarDeletingDtor ? ⚠rename` | 🟡 | dtor | Calls 0072d130 for object cleanup, then frees the allocation when the deleting-destructor flag bit is set. |
| `00ad6730` | `InspectProxy::MarkInspectReady ? ⚠rename` | 🟡 | virtual | Writes byte value 1 to the field at offset +0x100 of the object referenced by this+0x10. |
| `00ad73d0` | `InspectProxy::OnInspectEvent ? ⚠rename` | 🟡 | virtual | Calls virtual slot +0x10 on the object at this+8, passing the dword at this+0x13f0. |
| `00ad73f0` | `InspectProxy::OnInspectReady ? ⚠rename` | 🟡 | virtual | Calls target handler slot +0x10 with a stored id; when it returns nonzero, invokes 0077c0e0 with DAT_01bc2660. |
| `00ad7430` | `InspectProxy::OnInspectSelect` | ✅ | virtual | Calls target handler slot +0x10 with id +0x13f0; on success obtains an object through 0072f4d0 and invokes its slot +0x30. |
| `00ad7460` | `InspectProxy::UpdateProfile` | ✅ | virtual | Compares changed object ids, updates profile geometry and active state, and for ids 1 through 17 creates per-entry tracking records when referenced objects exist. |
| `00ad7660` | `InspectProxy::ClearProfileEntries` | ✅ | virtual | Iterates 17 pointer slots starting at +0x1444, releases active entries through 007e7f00, clears them and resets field +0x13f4. |
| `00ad76d0` | `InspectProxy::ClearTarget ?` | 🟡 | function | If target id at +0x13f4 is nonzero, calls target handler virtual slot +8 with it and clears the field. |
| `00ad7c10` | `InspectProxy::OnTargetChanged` | ✅ | virtual | Compares incoming object id with stored id +0x13ec; releases and clears matching target or releases and clears alternate target field +0x13f4. |
| `00ad7fc0` | `InspectProxy::Dtor → InspectProxy::~InspectProxy` | ✅ | dtor | Restores InspectProxy vtables, releases target references and nested resources, destroys CharacterStat and profile arrays and strings, then frees owned components. |
| `00ad8390` | `InspectProxy::ScalarDeletingDtor` | ✅ | dtor | Calls InspectProxy cleanup and frees the allocation when the deleting flag bit is set. |
| `00ad8880` | `InspectProxy::InspectProxy ⚠downgrade` | 🟢 | ctor | Constructor for an InspectProxy actor handler: installs ActorHandler::vftable, BaseAttachmentGroupManagerHandler::vftable and both InspectProxy vtable pointers, default-constructs its IString/List members and the Profil… |
