# Class: SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00924140` | `SensoryEffectDefinitionManager_ActorRumble_Ctor` | ✅ | ctor | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256> vtable at this and this+4, constructs the member map through 0092… |
| `009241f0` | `SensoryEffectActorRumble_Slot0Thunk` | ✅ | virtual | Adjustor thunk: subtracts 4 from ECX and jumps to 009251a0. |
| `00924200` | `SensoryEffectActorRumble_CountPositive` | ✅ | virtual | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00924210` | `SensoryEffectActorRumble_ReturnAt34` | ✅ | virtual | Returns the dword at this+0x34. |
| `00924220` | `SensoryEffectActorRumble_ReturnAt3c` | ✅ | virtual | Returns the dword at this+0x3c. |
| `00924360` | `SensoryEffectActorRumble_CastGetAt0xc0` | 🟢 | virtual | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and ActorRumbleEffectDefinition and 0 (a dynamic-cast helper), then returns the dword at +0xc0 of the result. |
| `00924520` | `— ⚠reject` | ⚪ | virtual | Virtual slot [6]: builds a source record through 00705b00 from the source at param_3+4, inserts it with 00924490, stores param_2 at element+0x10, calls 006f9e60 on the source, then calls the manager's virtual slot [5] (… |
| `00924590` | `SensoryEffectDefinitionManager_ActorRumble_Dtor` | ✅ | dtor | Destructor body: stores the SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x440, scans the buckets through 008e2b90, calls 00920c40 (the Ac… |
| `009251a0` | `SensoryEffectActorRumble_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 00924590 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
