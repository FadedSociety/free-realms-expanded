# Class: TargetProcessorActorEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c49a0` | `TargetProcessorActorEventHandler::Constructor` | ✅ | ctor | Installs TargetProcessorEventHandler vtable, zeros fields +4 and +8, then installs TargetProcessorActorEventHandler vtable. |
| `008c4a10` | `TargetProcessorActorEventHandler::ScalarDeletingDestructor` | ✅ | dtor | Restores TargetProcessorEventHandler vtable, then frees this when deleting-flag bit 0 is set; otherwise returns this. |
