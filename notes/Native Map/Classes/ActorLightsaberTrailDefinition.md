# Class: ActorLightsaberTrailDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cfdb0` | `ActorLightsaberTrailDefinition::~ActorLightsaberTrailDefinition` | ✅ | dtor | Destructor of an actor lightsaber-trail definition: rewrites ActorLightsaberTrailDefinition::vftable at +0 and +0x90, releases the embedded SoeUtil::IString at +0x98 (capacity +0xa4) through the locked refcount, resets … |
| `008cfec0` | `ActorLightsaberTrailDefinition::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the ActorLightsaberTrailDefinition destructor body FUN_008cfdb0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00927b30` | `ActorLightsaberTrailDefinition::Ctor` | ✅ | ctor | Initializes requester state, installs definition and IString vtables, clears fields, and copies defaults. |
| `00927c70` | `ActorLightsaberTrailDefinition::InitWithData` | 🟢 | method | Constructs the trail definition and writes its key at offset 200. |
