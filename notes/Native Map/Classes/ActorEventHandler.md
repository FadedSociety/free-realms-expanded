# Class: ActorEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c57b0` | `ActorEventHandler::ActorEventHandler` | ✅ | ctor | Installs an SEH frame, calls FUN_008c49a0 with incoming this in ECX, stores ActorEventHandler::vftable at this+0 and param_2 at this+0xc, restores the prior exception chain, and returns this. The stack-cookie value rema… |
| `008c5860` | `ActorEventHandler::ScalarDeletingDtor` | ✅ | dtor | Stores TargetProcessorEventHandler::vftable at this+0. If flag bit 0 is set, calls _free(this); otherwise returns this. Returns this with ret 4. |
| `0093f630` | `ActorEventHandler::HandleEvent` | ✅ | handler | For param_8==8 and a nonnull actor, builds a stack Target using actor+0x268 and sends param_5/param_6 through direct helper 008a90c0. It then forwards through 0093bba0 unless the non-8 event gate at client+0x769 suppres… |
