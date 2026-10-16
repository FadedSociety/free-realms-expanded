# Class: Audio::CloneWarsAudioService

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123a340` | `Audio::CloneWarsAudioService::Ctor` | ✅ | ctor | Stores Audio::CloneWarsAudioService::vftable into the object, assigns the string at +8 to 'MX_SD_Loop_1', calls FUN_0041ba81(0,0), FUN_0123ac60 and FUN_0123ad50 to set up the sub-objects at +0x24 and +0x30..+0x38, and c… |
| `0123a4c0` | `Audio::CloneWarsAudioService::DtorBody` | ✅ | dtor | Stores Audio::CloneWarsAudioService::vftable into the object, then calls FUN_00508490, FUN_00505ffd, FUN_0123ac90 and FUN_0041ba81(1,0) in sequence, releasing sub-objects and the string field, and finishes with the stub… |
| `0123aa00` | `Audio::CloneWarsAudioService::ScalarDeletingDtor` | ✅ | dtor | Calls the destructor body FUN_0123a4c0 and, when bit 0 of the flag argument is set, frees the object with _free; returns the object pointer. |
