# Class: SoeUtil::HashListMap<int,EffectTrackingStub,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00958cb0` | `EffectTrackingStubMapCtor ?` | 🟡 | ctor | Installs HashListMap<int,EffectTrackingStub,512,-1> vtable, clears counters and links, zeroes 0x800 bytes of bucket storage, and returns this. |
| `00962880` | `EffectTrackingStubHashListMapDestructor` | ✅ | dtor | Installs the HashListMap vtable and repeatedly unlinks the node at param_1[2] through FUN_0095b740 until the list is empty. |
| `009628e0` | `EffectTrackingStubHashListMapScalarDeletingDestructor` | ✅ | dtor | Restores the map vtable, repeatedly removes the head node through FUN_0095b740, then frees param_1 when flag bit 0 is set. |
| `00962e80` | `EffectTrackingStubHashListMapDestructor` | ✅ | dtor | Installs the typed map vtable and repeatedly removes the current list node through FUN_0095b740 until no node remains. |
