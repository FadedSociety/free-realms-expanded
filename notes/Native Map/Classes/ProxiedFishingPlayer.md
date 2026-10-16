# Class: ProxiedFishingPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccf720` | `ProxiedFishingPlayer::ReleaseResources` | 🟢 | method | Invokes the deleting virtual method on non-null resources at offsets 0xd4 and 0x130, then clears both pointers. |
| `00ccf860` | `ProxiedFishingPlayer::ReleaseHandle ?` | 🟡 | method | When the stored resource at offset 0x40 is nonzero, calls its virtual release method and clears the resource pointer. |
| `00ccf890` | `ProxiedFishingPlayer::Ctor` | ✅ | ctor | Installs the ProxiedFishingPlayer vtable, stores three constructor values, zeros activity and resource fields, initializes two strings, and clears state fields. |
| `00ccf9c0` | `ProxiedFishingPlayer::Dtor` | ✅ | dtor | Restores the player vtable, releases two owned objects and one actor resource, detaches the actor when present, then decrements and releases both strings. |
| `00ccfb00` | `ProxiedFishingPlayer::SetAnimation` | 🟢 | method | Resolves the actor, clears a temporary actor flag when present, stores the requested animation id, and queues a presentation update when the actor is active. |
| `00ccfbd0` | `ProxiedFishingPlayer::ScalarDeletingDtor` | ✅ | dtor | Destroys the proxied player and frees its allocation when the deleting flag is set. |
| `00ccfbf0` | `ProxiedFishingPlayer::ShowResult` | 🟢 | method | Releases an old result object, creates result presentation when a result id is positive, copies player and fish context, and assigns a result-specific display category. |
| `00ccfdb0` | `— ⚠reject` | ⚪ | method | Clears an existing display, creates the result presenter for a positive result id, initializes it with player data and timing fields, and starts presentation when readiness checks pass. |
