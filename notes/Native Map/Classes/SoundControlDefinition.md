# Class: SoundControlDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1100` | `SoundControlDefinition::~SoundControlDefinition` | ✅ | dtor | Non-deleting destructor of the effect definition: rewrites its own vtable to SoundControlDefinition::vftable, then calls the base-destructor body FUN_007e0b10 (which rewrites ActorEffectDefinition::vftable and releases … |
| `008d1190` | `SoundControlDefinition::Slot4_Is9cZero` | 🟢 | virtual | Twelve-byte predicate decoded from the raw bytes (33 c0 / 39 81 9c 00 00 00 / 0f 94 c0 / c3): xors eax with itself, compares the dword at this+0x9c against zero, returns the sete result; a boolean state query with no si… |
| `008d11a0` | `SoundControlDefinition::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot [0]): rewrites the object vtable to SoundControlDefinition::vftable, inlines the destructor body by calling the base cleanup FUN_007e0b10, and frees the object through the CR… |
| `008da880` | `SoundControlDefinition::~SoundControlDefinition` | ✅ | dtor | Destructor body: stores SoundControlDefinition::vftable (raw bytes c7 01 5c 2a 80 01) into the object's vtable slot, then calls 007e0b10, the base-class destructor body that itself references ActorEffectDefinition::vfta… |
| `00928290` | `SoundControlDefinition::Ctor` | ✅ | ctor | Initializes the base definition, installs its RTTI vtable, and initializes an embedded member through 007bd400. |
| `00928360` | `SoundControlDefinition::InitWithKey` | 🟢 | method | Constructs a SoundControlDefinition and stores its key at object offset 0xd8. |
