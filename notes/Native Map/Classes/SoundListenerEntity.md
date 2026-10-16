# Class: SoundListenerEntity

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089f020` | `SoundListenerEntity::SoundListenerEntity` | ✅ | ctor | Object constructor: first runs the Deep::Entity-derived base constructor FUN_007648d0 on this (it writes Deep::Entity::vftable, clears the base fields and applies the object's active-flag handling), then stores param_2 … |
| `0089f0b0` | `SoundListenerEntity::~SoundListenerEntity` | ✅ | dtor | Destructor body: writes SoundListenerEntity::vftable into the object and then calls the Deep::Entity-derived base destructor FUN_00764e10 on it, which republishes Deep::Entity::vftable, drops the reference at +0xc (atom… |
| `0089f110` | `SoundListenerEntity::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: writes SoundListenerEntity::vftable, runs the base destructor FUN_00764e10 (reference-count release and handle cleanup) and then calls _free(this) when bit 0 of param_2 is set. |
| `008b2fa0` | `SoundListenerEntity::UpdateTransformsFromSource` | 🟢 | method | Copies the four dwords at param_2+0x30..+0x3c into this+0x20..+0x2c, then calls 007bd8f0 with the source block at param_2+0x30 and 007bd930 with the source block at param_2+0x10; both helpers are entered with ECX = *(th… |
