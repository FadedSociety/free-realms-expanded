# Class: SensoryEffectDefinitionManager<ActorCompositeEffectDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0093faf0` | `SensoryEffectManagerCtor` | ✅ | ctor | Constructs the sensory-effect loader base, installs both manager vtables, calls 0093f810 with ECX=this+0x2c to initialize the embedded ActorCompositeEffectDefinition map, then calls 009072d0 with ECX=this+0x440 and retu… |
| `0093fba0` | `SensoryEffectManagerScalarDtor → SensoryEffectDefinitionManager<ActorCompositeE…` | ✅ | dtor | Bytes subtract 4 from ECX and jump to 00940a80, adjusting a secondary-base pointer before shared deletion. |
| `0093fbb0` | `SensoryEffectManagerHasPositiveCount` | ✅ | virtual | Returns true exactly when the signed 32-bit field at this+0x3c is greater than zero. |
| `0093fbc0` | `SensoryEffectManagerGetField34` | ✅ | virtual | Returns the 32-bit value at this+0x34. |
| `0093fbd0` | `SensoryEffectManagerGetField3c` | ✅ | virtual | Returns the 32-bit value at this+0x3c. |
| `0093fd10` | `SensoryEffectManagerGetCastField40` | ✅ | virtual | Uses RTTI to cast its argument from SensoryEffectDefinition to ActorCompositeEffectDefinition, then returns the cast object dword at +0x40. |
| `0093fed0` | `SensoryEffectManagerInsertEntryAndDispatch` | ✅ | virtual | Calls 00705b00 on the dword at the input object+4, inserts the resulting string entry into the manager map at +0x440 through 0093fe40, stores param_2 at entry+0x10, and copies the input IString into that entry. It then … |
| `0093ff40` | `DestroySensoryEffectManager` | ✅ | dtor | Restores manager vtables, destroys the name-map subobject at +0x440 through 008e28d0, drains the ActorCompositeEffectDefinition map at +0x2c through 0093f7a0, then calls the sensory-effect loader destructor 008cf9d0. |
| `00940a80` | `SensoryEffectManagerScalarDtor` | ✅ | dtor | Calls 0093ff40 to destroy owned maps and loader, then frees the object only when the deleting flag is set. |
