# Class: SensoryEffectDefinitionManager<ActorLightEmitterDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00923780` | `— ⚠reject` | ⚪ | ctor | Constructor: calls 008cf920 with the two stack arguments (SensoryEffectDefinitionLoader constructor), stores the SensoryEffectDefinitionManager<ActorLightEmitterDefinition,256> vtable at this and at this+4, constructs t… |
| `00923830` | `SensoryEffectLightEmitter_Slot0Thunk` | ✅ | virtual | Adjustor thunk: subtracts 4 from ECX and jumps to 00925160. |
| `00923840` | `SensoryEffectLightEmitter_CountPositive` | ✅ | virtual | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00923850` | `— ⚠reject` | ⚪ | virtual | Returns the dword at this+0x34. |
| `00923860` | `SensoryEffectLightEmitter_ReturnAt3c` | ✅ | virtual | Returns the dword at this+0x3c. |
| `009239a0` | `SensoryEffectLightEmitter_CastGetAt0xd0` | 🟢 | virtual | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and ActorLightEmitterDefinition and 0 (a dynamic-cast helper), then returns the dword at +0xd0 of the result. |
| `00923b60` | `SensoryEffectLightEmitter_InsertKeyed` | ✅ | virtual | Virtual slot [6]: builds a source record through 00705b00 from the source at param_3+4, inserts it with 00923ad0, stores param_2 at element+0x10, calls 006f9e60 on the source, then calls the manager's virtual slot [5] (… |
| `00923bd0` | `SensoryEffectDefinitionManager_ActorLightEmitter_Dtor` | ✅ | dtor | Destructor body: stores the SensoryEffectDefinitionManager<ActorLightEmitterDefinition,256> vtables at this and this+4, stores the HashMap<int,...NameEntry,256,-1> vtable at this+0x440, scans the buckets through 008e27f… |
| `00925160` | `SensoryEffectLightEmitter_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 00923bd0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
