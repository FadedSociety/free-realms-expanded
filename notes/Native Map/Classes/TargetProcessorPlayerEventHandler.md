# Class: TargetProcessorPlayerEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b3470` | `TargetProcessorPlayerEventHandler::TargetProcessorPlayerEventHandler` | ✅ | ctor | Installs TargetProcessorEventHandler::vftable, zeroes +4 and +8, then overwrites the first dword with TargetProcessorPlayerEventHandler::vftable, so the finished object carries the derived vtable and two null fields. |
| `008b3530` | `— ⚠reject` | ⚪ | dtor | Stores TargetProcessorEventHandler::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
