# Class: TargetProcessorEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b3380` | `TargetProcessorEventHandler::DestructorCleanup_008b3380` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable into *this and returns; a 7-byte fragment with no free, called from MSVC unwind funclets (0157f780, 0157f7b0, 0157f8a0, ...). |
| `008b3390` | `MatchOneAndZero` | ✅ | virtual | Returns 1 only when param_1 equals 1 and param_2 equals 0, and 0 for every other argument pair. |
| `008b33b0` | `— ⚠reject` | ⚪ | dtor | Stores TargetProcessorEventHandler::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
| `008b3400` | `TargetProcessorEventHandler::DestructorCleanup_008b3400` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable into *this and returns; 67 bytes of SEH prologue plus the single store, called from unwind funclets 01582320, 01582350, 015a36b0 and neighbours. |
| `008b34e0` | `TargetProcessorEventHandler::DestructorCleanup_008b34e0` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable into *this and returns; 67 bytes with SEH prologue, called from unwind funclets 01581580, 015815b0, 015822f0, 015b1ad0 and neighbours. |
| `008b3600` | `TargetProcessorEventHandler::DestructorCleanup_008b3600` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable into *this and returns; called from unwind funclets 015815e0, 01581610, 015823b0, 015a3740 and neighbours. |
| `008b8890` | `TargetProcessorEventHandler::TargetProcessorEventHandler` | ✅ | ctor | Writes TargetProcessorEventHandler::vftable to the object and returns it. |
| `008b8940` | `TargetProcessorEventHandler::DtorBody ⚠downgrade` | 🟢 | dtor | Writes TargetProcessorEventHandler::vftable to the object and returns. |
| `008c5750` | `TargetProcessorEventHandler::ScalarDeletingDtor ⚠downgrade` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable at this+0, then calls _free on the block when bit 0 of the stack flag byte is set. Returns this (ret 4). Exception frame is set up and torn down around it. |
| `008c5810` | `TargetProcessorEventHandler::TargetProcessorEventHandler` | ✅ | ctor | Sets up an exception frame, stores TargetProcessorEventHandler::vftable at this+0 (mov [ecx],0x01802274) and tears the frame down. No other fields are written and no base constructor is called in this body. |
| `009c9f30` | `TargetProcessorEventHandler::Dtor ? ⚠rename` | 🟡 | dtor | Restores the TargetProcessorEventHandler vtable at the base object. |
| `009ca2a0` | `TargetProcessorEventHandler::Ctor ? ⚠rename` | 🟡 | ctor | Installs the TargetProcessorEventHandler vtable at the object base. |
| `00a2ef80` | `TargetProcessorEventHandler::TargetProcessorEventHandler` | ✅ | ctor | Stores TargetProcessorEventHandler::vftable at +0 and nothing else. |
| `00a2f0b0` | `TargetProcessorEventHandler::TargetProcessorEventHandler ⚠downgrade` | 🟢 | ctor | Stores TargetProcessorEventHandler::vftable at +0 and nothing else. |
| `00a2f740` | `TargetProcessorEventHandler::TargetProcessorEventHandler ⚠downgrade` | 🟢 | ctor | Stores TargetProcessorEventHandler::vftable at +0 and nothing else. |
| `00aaacc0` | `TargetProcessorEventHandler::ctor ⚠downgrade` | 🟢 | ctor | Minimal constructor: stamps TargetProcessorEventHandler::vftable at this+0 and returns the object; no base class call and no member writes. |
| `00aaadd0` | `TargetProcessorEventHandler::ctor ⚠downgrade` | 🟢 | ctor | Minimal constructor: stamps TargetProcessorEventHandler::vftable at this+0 and returns the object; no base class call and no member writes. |
| `00aae440` | `TargetProcessorEventHandler::ctor` | ✅ | ctor | Minimal constructor: stamps TargetProcessorEventHandler::vftable at this+0 and returns the object; no base class call and no member writes. |
