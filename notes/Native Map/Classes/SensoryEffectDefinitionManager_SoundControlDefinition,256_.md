# Class: SensoryEffectDefinitionManager<SoundControlDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00924b60` | `SensoryEffectDefinitionManager_SoundControl_Ctor` | ✅ | ctor | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<SoundControlDefinition,256> vtable at this and this+4, constructs the member map through 009210b0,… |
| `00924c10` | `SensoryEffectSoundControl_Slot0Thunk` | ✅ | virtual | Adjustor thunk: subtracts 4 from ECX and jumps to 00925240. |
| `00924c20` | `SensoryEffectSoundControl_CountPositive` | ✅ | virtual | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00924c30` | `SensoryEffectSoundControl_ReturnAt34` | ✅ | virtual | Returns the dword at this+0x34. |
| `00924c40` | `SensoryEffectSoundControl_ReturnAt3c` | ✅ | virtual | Returns the dword at this+0x3c. |
| `00924d80` | `SensoryEffectSoundControl_CastGetAt0xd0` | 🟢 | virtual | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and SoundControlDefinition and 0 (a dynamic-cast helper), then returns the dword at +0xd0 of the result. |
| `00924f40` | `— ⚠reject` | ⚪ | virtual | Virtual slot [6]: builds a source record through 00705b00 from the source at param_3+4, inserts it with 00924eb0, stores param_2 at the element, releases the source through 006f9e60, then calls the manager's virtual slo… |
| `00924fb0` | `SensoryEffectDefinitionManager_SoundControl_Dtor` | ✅ | dtor | Destructor body: stores the SensoryEffectDefinitionManager<SoundControlDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x440, scans the buckets through 008e30d0, calls 00920d60 (the SoundCo… |
| `00925240` | `SensoryEffectSoundControl_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 00924fb0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
