# Class: SoundPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b098a0` | `SoundPlayer::Constructor` | 🟢 | ctor | Installs SoundPlayer vtable; initializes emitter list, DefinitionEntry lists, integer array and enabled flag. |
| `00b09930` | `SoundPlayer::Cleanup` | 🟢 | dtor | Resets the SoundPlayer vtable and cleans its integer array, DefinitionEntry container and EmitterEntry list. |
| `00b099b0` | `SoundPlayer::ScalarDeletingDtor` | 🟢 | dtor | Calls SoundPlayer cleanup and frees this only when delete flag bit 0 is set. |
| `00b099d0` | `UpdateSoundCompletions` | 🟢 | method | Updates each emitter, removes finished entries, accumulates completion ids, dispatches HUD:OnSoundComplete and clears the completion count. |
