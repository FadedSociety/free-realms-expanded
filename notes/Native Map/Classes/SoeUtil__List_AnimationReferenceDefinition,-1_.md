# Class: SoeUtil::List<AnimationReferenceDefinition,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0078a8a0` | `ListAnimationReferenceDefinition::~List` | ✅ | dtor | Stores SoeUtil::List<AnimationReferenceDefinition,-1>::vftable at +0 and calls 007861e0(head) while the head pointer at +4 is nonzero. Does not free the object. |
| `0078a8e0` | `ListAnimationReferenceDefinition::VSlot2_PoolDispatch ?` | 🟡 | virtual | Calls 006fb9c0(), then 006fb860(); if that returns an object it calls the object's vtable slot [1] and returns. Otherwise, if the global DAT_01bdf280 is non-null, it calls that object's slot [1] and returns. Otherwise i… |
| `0078a920` | `ListAnimationReferenceDefinition::VSlot3_FreeAlign4 ?` | 🟡 | virtual | Takes one stack argument and calls 00401c00(argument, 4) (alignment 4), then returns with ret 4. Does not use this. Virtual slot [3] of the List vtable. |
| `0078a940` | `ListAnimationReferenceDefinition::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): stores the List vtable at +0, calls 007861e0(head) while the head at +4 is nonzero, and calls _free(this) when bit 0 of the flag argument is set. Returns this. |
