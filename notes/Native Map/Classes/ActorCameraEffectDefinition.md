# Class: ActorCameraEffectDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cfba0` | `ActorCameraEffectDefinition::~ActorCameraEffectDefinition` | ✅ | dtor | Destructor of an actor camera-effect definition: rewrites ActorCameraEffectDefinition::vftable at +0 and +0x90, sets the +0x90 sub-object to AssetManagement::AssetRequestInterface::AssetRequester::vftable, releases the … |
| `008cfc50` | `ActorCameraEffectDefinition::FloatEqSentinel_a8` | 🟢 | function | Float equality predicate (27 bytes): loads the float at this+0xa8 and returns 1 when it equals the sentinel/default constant DAT_01762a30, else 0. It is slot [4] of ActorCameraEffectDefinition::vftable (01802954). |
| `008cfc70` | `ActorCameraEffectDefinition::GetByte_b4` | 🟢 | function | Seven-byte getter (mov al,[ecx+0xb4]; ret): returns the byte at param_1+0xb4. It is slot [5] of ActorCameraEffectDefinition::vftable (01802954). |
| `008cfc90` | `ActorCameraEffectDefinition::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the ActorCameraEffectDefinition destructor body FUN_008cfba0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00927110` | `ActorCameraEffectDefinition::Ctor` | ✅ | ctor | Initializes requester state and pointer, copies defaults, installs definition vtables, and clears a byte field. |
| `00927250` | `ActorCameraEffectDefinition::InitWithData` | 🟢 | method | Constructs the definition from supplied values and stores its key at offset 200. |
