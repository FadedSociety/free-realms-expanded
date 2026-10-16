# Class: ActorSoundEmitterDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00945e10` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256> constructor 00944b00 on this with the first two stack arguments, installs ActorSoundEmitterDefinitionManager::vftable at +0 (018089f… |
| `00945f00` | `ActorSoundEmitterDefinitionManager::ThunkSubEcx4ToScalarDtor` | ✅ | virtual | Secondary-vtable thunk (018089e4 slot 0). Bytes 83 e9 04 subtract 4 from ECX, then e9 jumps to 009464a0. It is not a body. |
| `00946420` | `ActorSoundEmitterDefinitionManager::AddDefinition` | 🟢 | virtual | Virtual slot 5 of ActorSoundEmitterDefinitionManager (018089fc slot 5). Copies its stack argument into a local, then calls 00946390 on the member at this+0x2c with (&local, &argument, this+0xc, this+0x84c) as stack argu… |
| `00946450` | `ActorSoundEmitterDefinitionManager::~ActorSoundEmitterDefinitionManager` | ✅ | dtor | Destructor body: stores ActorSoundEmitterDefinitionManager::vftable at +0 and +4, then calls the SensoryEffectDefinitionManager destructor body 00944f30 on this. |
| `009464a0` | `ActorSoundEmitterDefinitionManager::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: calls the body 00946450 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
