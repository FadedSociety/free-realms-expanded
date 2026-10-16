# Class: Audio::AudioService

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123a580` | `Audio::AudioService::SetParamOrDefault ?` | 🟡 | virtual | Virtual slot [3] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: when the argument is outside the range (0,15000) or negative it replaces it with 0x3d5a; when the sub-object at +0x34 is presen… |
| `0123a600` | `Audio::AudioService::StopHandlesMatchingKey ?` | 🟡 | virtual | Virtual slot [4] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: when the sub-object at +0x34 is present it walks the list at +0x38/+0x40, and for each entry whose key equals the argument it c… |
| `0123a770` | `Audio::AudioService::StopAllHandles ?` | 🟡 | virtual | Virtual slot [6] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: calls its own slot [5], then when the sub-object at +0x34 is present walks the list at +0x38..+0x40 calling the sub-object's sl… |
| `0123a8a0` | `Audio::AudioService::GetNameString ?` | 🟡 | virtual | Virtual slot [8] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: calls FUN_0123aaa0 to locate a list position and compares it with the position stored at +0x28; on equality it assigns the stri… |
