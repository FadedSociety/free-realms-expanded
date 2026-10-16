# Class: ActorLightEmitterDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cfa90` | `ActorLightEmitterDefinition::~ActorLightEmitterDefinition` | ✅ | dtor | Destructor of an actor light-emitter definition: rewrites ActorLightEmitterDefinition::vftable at +0 and at +0x90, sets the +0x90 sub-object vtable to AssetManagement::AssetRequestInterface::AssetRequester::vftable, rel… |
| `008cfb40` | `ActorLightEmitterDefinition::FloatEqSentinel_bc` | 🟢 | function | Float equality predicate (30 bytes): loads the float at this+0xbc and returns 1 when it equals the sentinel/default constant DAT_01762a30, else 0. It is slot [4] of ActorLightEmitterDefinition::vftable (01802910). |
| `008cfb60` | `ActorLightEmitterDefinition::GetByte_c8` | 🟢 | function | Seven-byte getter (mov al,[ecx+0xc8]; ret): returns the byte at param_1+0xc8. It is slot [5] of ActorLightEmitterDefinition::vftable (01802910). |
| `008cfb80` | `ActorLightEmitterDefinition::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the ActorLightEmitterDefinition destructor body FUN_008cfa90 and frees the object with _free when bit 0 of the flag argument is set. |
| `00926bf0` | `ActorLightEmitterDefinition::Ctor` | ✅ | ctor | Initializes requester state, stores its pointer, installs definition vtables, copies defaults, and clears a byte field. |
| `00926d40` | `ActorLightEmitterDefinition::InitWithData` | 🟢 | method | Constructs an ActorLightEmitterDefinition, stores the supplied key at +0xd8, and returns this. |
