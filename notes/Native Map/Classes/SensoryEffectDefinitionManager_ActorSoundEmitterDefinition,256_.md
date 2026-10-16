# Class: SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00944b00` | `SensoryEffectDefinitionManager::SensoryEffectDefinitionManager` | ✅ | ctor | Constructor: calls the SensoryEffectDefinitionLoader base constructor 008cf920 on this with its two stack arguments, installs SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256>::vftable at +0 and +4, runs m… |
| `00944ba0` | `SensoryEffectDefinitionManager::ThunkSubEcx4ToScalarDtor → SensoryEffectDefinit…` | ✅ | virtual | Secondary-vtable thunk (018089ac slot 0). Bytes 83 e9 04 subtract 4 from ECX, then e9 jumps to 00944fc0. It is not a body. |
| `00944bb0` | `SensoryEffectDefinitionManager::PredicateField3cPositive ?` | 🟡 | virtual | Returns 1 when the signed dword at this+0x3c is greater than zero, else 0 (xor eax,eax; cmp [ecx+0x3c],eax; setg al). |
| `00944bc0` | `SensoryEffectDefinitionManager::GetField34 ?` | 🟡 | virtual | Field getter: returns the dword at this+0x34 (bytes 8b 41 34 c3). |
| `00944bd0` | `SensoryEffectDefinitionManager::GetField3c ?` | 🟡 | virtual | Field getter: returns the dword at this+0x3c (bytes 8b 41 3c c3). |
| `00944d00` | `SensoryEffectDefinitionManager::DynCastGetField1b0 ?` | 🟡 | virtual | Stack-argument virtual (ret 4). Calls __RTDynamicCast(arg, 0, SensoryEffectDefinition RTTI, ActorSoundEmitterDefinition RTTI, 0) and returns the dword at +0x1b0 of the result. A failed cast is not checked. |
| `00944ec0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 6 (shared with ActorSoundEmitterDefinitionManager slot 6). Computes a key with 00705b00 from [param_3+4], inserts a node into the hash map at this+0x440 with 00944e30, stores param_2 at node+0x10, calls 006… |
| `00944f30` | `SensoryEffectDefinitionManager::~SensoryEffectDefinitionManager` | ✅ | dtor | Destructor body: writes SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256>::vftable at +0 and +4 and the HashMap<int,NameEntry> vtable at +0x440, calls 008e2ef0 (the map teardown; receiver not decoded), cal… |
| `00944fc0` | `SensoryEffectDefinitionManager::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: calls the body 00944f30 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
