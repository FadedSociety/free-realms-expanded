# Class: SoeUtil::HashList<Actor::PendingAnimationLoad,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0078a800` | `HashListPendingAnimationLoad::HashListCtor` | ✅ | ctor | Writes HashListMap<int,Actor::PendingAnimationLoad,10,-1>::vftable and then HashList<Actor::PendingAnimationLoad,10,-1>::vftable at +0, and zeroes the dwords at +0x4 through +0x38. Makes no calls. |
| `0078a880` | `HashListPendingAnimationLoad::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls 00788010(this) (the HashListMap destructor body) and then _free(this) when bit 0 of the flag argument is set. Returns this. |
