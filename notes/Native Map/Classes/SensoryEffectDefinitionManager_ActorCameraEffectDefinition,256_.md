# Class: SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00923c60` | `SensoryEffectDefinitionManager_ActorCamera_Ctor` | ✅ | ctor | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256> vtable at this and this+4, constructs the member map through 0092… |
| `00923d10` | `SensoryEffectActorCamera_Slot0Thunk` | ✅ | virtual | Adjustor thunk: subtracts 4 from ECX and jumps to 00925180. |
| `00923d20` | `SensoryEffectActorCamera_CountPositive` | ✅ | virtual | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00923d30` | `SensoryEffectActorCamera_ReturnAt34` | ✅ | virtual | Returns the dword at this+0x34. |
| `00923d40` | `SensoryEffectActorCamera_ReturnAt3c` | ✅ | virtual | Returns the dword at this+0x3c. |
| `00923e80` | `SensoryEffectActorCamera_CastGetAt0xc0` | 🟢 | virtual | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and ActorCameraEffectDefinition and 0 (a dynamic-cast helper), then returns the dword at +0xc0 of the result. |
| `00924040` | `— ⚠reject` | ⚪ | virtual | Virtual slot [6]: builds a source record through 00705b00 from the source at param_3+4, inserts it with 00923fb0, stores param_2 at element+0x10, calls 006f9e60 on the source, then calls the manager's virtual slot [5] (… |
| `009240b0` | `SensoryEffectDefinitionManager_ActorCamera_Dtor` | ✅ | dtor | Destructor body: stores the SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x440, scans the buckets through 008e2ab0, calls 00920bd0 (the Ac… |
| `00925180` | `SensoryEffectActorCamera_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 009240b0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
