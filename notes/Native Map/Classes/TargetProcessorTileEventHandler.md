# Class: TargetProcessorTileEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b3590` | `TargetProcessorTileEventHandler::TargetProcessorTileEventHandler` | ✅ | ctor | Installs TargetProcessorEventHandler::vftable, zeroes +4 and +8, then stores TargetProcessorTileEventHandler::vftable into the first dword. |
| `008b3650` | `— ⚠reject` | ⚪ | dtor | Stores TargetProcessorEventHandler::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
