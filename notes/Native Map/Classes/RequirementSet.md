# Class: RequirementSet

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bba490` | `RequirementSet::~RequirementSet` | 🟢 | dtor | Forwards the incoming receiver to RequirementSet cleanup at 010821b0 inside an exception frame, restores the exception-list pointer, and returns. |
| `00bbc690` | `RequirementSet::~RequirementSet` | 🟢 | dtor | Runs the RequirementSet destructor body on the receiver, calls _free(this) only when flag bit 0 is set, and restores the SEH frame. |
| `01082040` | `RequirementSetCtor` | 🟢 | ctor | Constructor body: stores the RequirementSet vftable at this+0x0, zeroes +0x4, +0x8, +0x14, +0x18, +0x1c, +0x20, +0x24 and +0x28, and stores the argument at +0xc. |
| `010821b0` | `RequirementSetDtorBody` | 🟢 | dtor | Destructor body (bytes: lea ecx,[esi+0x14]; store vftable; call 01082120; lea ecx,[esi+0x20]; jmp 01082120): stores the RequirementSet vftable at this+0x0, drains the list at this+0x14 through a call, then tail-jumps to… |
| `010821d0` | `RequirementSetScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: stores the RequirementSet vftable, drains two lists through 01082120, then frees this with FreeMemory (alignment 0) when flag bit 0 of the argument is set, and returns this. |
