# Class: SoeUtil::HashListMap<int,GuiEventHandler*,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081ba10` | `HashListMapGuiEventHandler512_DtorBody` | ✅ | dtor | Destructor body (does not free the object): restores SoeUtil::HashListMap<int,GuiEventHandler*,512,-1>::vftable at [0], then unlinks and releases nodes one at a time through 008197f0 (512-bucket node removal, mask 0x1ff… |
| `0081bda0` | `HashListMapGuiEventHandler512_DtorBodyUnwind` | ✅ | dtor | Destructor body called from EH unwind funclets (does not free the object): restores the HashListMap<int,GuiEventHandler*,512,-1> vtable at [0] and unlinks nodes through 008197f0 until the head at [2] is null. Same body … |
