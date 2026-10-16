# Class: GuiEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00822c10` | `GuiEventHandler::DtorBody` | 🟢 | dtor | Stores GuiEventHandler::vftable at [ecx] and returns (bytes c7 01 a8 2e 7f 01 c3); no members released. |
| `00822c30` | `GuiEventHandler::ScalarDeletingDtor` | 🟢 | dtor | Stores GuiEventHandler::vftable at this, then calls _free (thunk 00d82463) on this only when bit 0 of the stack flag byte is set; returns this (bytes: test byte [esp+4],1; push esi; mov esi,ecx; mov [esi],vtable; je; pu… |
