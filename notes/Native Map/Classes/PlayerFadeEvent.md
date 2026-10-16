# Class: PlayerFadeEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00959e50` | `PlayerFadeEventCtor ?` | 🟡 | ctor | Calls PlayerEvent base setup, installs PlayerFadeEvent vtable, and stores one constructor argument at field 0x50. |
| `00959ee0` | `PlayerFadeEventDtor ?` | 🟡 | dtor | Restores PlayerFadeEvent then PlayerEvent vtables and calls base event cleanup. |
| `00959f40` | `— ⚠reject` | ⚪ | virtual | Obtains a global context; when present and deletion flag bit 0 is set, invokes its slot 9 with this+0x50, then calls 0076bbe0 using that value. |
| `00959f80` | `PlayerFadeEvent::ScalarDeletingDtor ?` | 🟡 | dtor | Calls PlayerFadeEvent destructor and frees the allocation when deletion flag bit 0 is set. |
