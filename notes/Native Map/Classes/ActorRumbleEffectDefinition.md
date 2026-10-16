# Class: ActorRumbleEffectDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cfcb0` | `ActorRumbleEffectDefinition::~ActorRumbleEffectDefinition` | ✅ | dtor | Destructor of an actor rumble-effect definition: rewrites ActorRumbleEffectDefinition::vftable at +0 and +0x90, sets the +0x90 sub-object to AssetManagement::AssetRequestInterface::AssetRequester::vftable, releases the … |
| `008cfd60` | `ActorRumbleEffectDefinition::FloatEqSentinel_ac` | 🟢 | function | Float equality predicate (27 bytes): loads the float at this+0xac and returns 1 when it equals the sentinel/default constant DAT_01762a30, else 0. It is slot [4] of ActorRumbleEffectDefinition::vftable (01802998). |
| `008cfd90` | `ActorRumbleEffectDefinition::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the ActorRumbleEffectDefinition destructor body FUN_008cfcb0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00927620` | `— ⚠reject` | ⚪ | ctor | Initializes requester state, copies default fields, installs definition vtables, and clears a byte field. |
| `00927760` | `ActorRumbleEffectDefinition::InitWithData` | 🟢 | method | Constructs the rumble-effect definition and stores its key at offset 200. |
