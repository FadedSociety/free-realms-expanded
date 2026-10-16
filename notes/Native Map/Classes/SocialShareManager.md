# Class: SocialShareManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae09b0` | `— ⚠reject` | ⚪ | ctor | Installs SocialShareManager vtable, initializes its HTTP base, string fields and three provider states, sets timeout/config values, allocates a request buffer, initializes HTTP handling, and runs manager setup. |
| `00ae0b10` | `— ⚠reject` | ⚪ | dtor | Installs manager vtable, shuts down HTTP support, destroys three provider records and the manager string; frees its allocated request object when present, then restores the TinyHttp base vtable. |
| `00ae0c00` | `SocialShareManager::ScalarDeletingDtor` | ✅ | dtor | Calls manager destruction body, frees this when flag bit 0 is set, and returns this otherwise. |
