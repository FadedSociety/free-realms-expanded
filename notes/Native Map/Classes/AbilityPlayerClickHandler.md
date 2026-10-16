# Class: AbilityPlayerClickHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2f030` | `AbilityPlayerClickHandler::AbilityPlayerClickHandler` | ✅ | ctor | Runs the base click-handler constructor FUN_008b3470, stores AbilityPlayerClickHandler::vftable, writes the argument at +0xc and zeroes the byte at +0x10. |
| `00a2f090` | `AbilityPlayerClickHandler::MatchKind` | ✅ | virtual | Returns true when the argument has any of the bits 0x800, 0x400 or 0x8 set (mask 0xc08), and false otherwise. |
| `00a2f100` | `AbilityPlayerClickHandler::ScalarDeletingDtor` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable (the only vtable step of this destructor chain) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPlayerClickHandler::vftable a… |
| `00a35a40` | `AbilityPlayerClickHandler::DispatchEventToProcessor` | ✅ | virtual | Virtual slot [3] of AbilityPlayerClickHandler::vftable (binary vtable read). Event code is the seventh argument (raw bytes: mov eax,[esp+0x18] at entry). Code 0x8: if processor (this+0xc) byte +0x60 is zero it returns; … |
