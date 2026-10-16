# Class: SoeUtil::HashListMap<int,Actor::PendingAnimationLoad,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00787f80` | `HashListMapPendingAnimationLoad::~HashListMap` | ✅ | dtor | Stores HashListMap<int,Actor::PendingAnimationLoad,10,-1>::vftable at +0, then loops while the head pointer at +8 is nonzero, calling 00786010(head). Does not free the object. |
| `00787fb0` | `HashListMapPendingAnimationLoad::DeletingDtor` | ✅ | dtor | Scalar deleting destructor shape (vtable slot [0]): stores the HashListMap vftable at +0, drains the head chain at +8 through 00786010 (body inlined, not a call to 00787f80), then calls _free(this) when bit 0 of the fla… |
| `00788010` | `HashListMapPendingAnimationLoad::~HashListMapSeh` | ✅ | dtor | Same destructor body as 00787f80 but installs a SEH frame (ExceptionList push) around it: stores the HashListMap vftable at +0, drains the head chain at +8 through 00786010, then restores ExceptionList and returns. Does… |
