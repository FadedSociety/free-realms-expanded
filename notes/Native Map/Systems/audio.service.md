# System: audio.service

30 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009157e0` | `AppServicesInitSoundSettings` |  | Initializes the app-services object with the sound settings XML path, reporting success or failure, then creates the audio service objects and reads the Rendering collection-mode settings. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `0123a340` | `Audio::CloneWarsAudioService::Ctor` | Audio::CloneWarsAudioService | Constructor of the Clone Wars audio service; it initialises its loop name and sub-objects. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123a4c0` | `Audio::CloneWarsAudioService::DtorBody` | Audio::CloneWarsAudioService | Destructor body of the Clone Wars audio service, run from the scalar deleting destructor and from EH cleanup. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123aa00` | `Audio::CloneWarsAudioService::ScalarDeletingDtor` | Audio::CloneWarsAudioService | Vtable slot [0] of Audio::CloneWarsAudioService: the scalar deleting destructor. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b098a0` | `SoundPlayer::Constructor` | SoundPlayer | Initializes SoundPlayer and its member containers. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09930` | `SoundPlayer::Cleanup` | SoundPlayer | Cleans SoundPlayer member containers; it is called by the deleting destructor. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b099b0` | `SoundPlayer::ScalarDeletingDtor` | SoundPlayer | Scalar deleting destructor for SoundPlayer. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b099d0` | `UpdateSoundCompletions` | SoundPlayer | The HUD callback string and SoundPlayer caller establish local completion reporting. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0123a580` | `Audio::AudioService::SetParamOrDefault ?` | Audio::AudioService | Virtual slot [3] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: when the argument is outside the range (0,15000) or negative it replaces it with 0x3d5a; when the sub-object at +0x34 is presen… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123a600` | `Audio::AudioService::StopHandlesMatchingKey ?` | Audio::AudioService | Virtual slot [4] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: when the sub-object at +0x34 is present it walks the list at +0x38/+0x40, and for each entry whose key equals the argument it c… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123a770` | `Audio::AudioService::StopAllHandles ?` | Audio::AudioService | Virtual slot [6] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: calls its own slot [5], then when the sub-object at +0x34 is present walks the list at +0x38..+0x40 calling the sub-object's sl… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123a8a0` | `Audio::AudioService::GetNameString ?` | Audio::AudioService | Virtual slot [8] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: calls FUN_0123aaa0 to locate a list position and compares it with the position stored at +0x28; on equality it assigns the stri… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## ⚪ unknown (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b06ec0` | `—` |  | Calls a base initializer with four supplied values and stores the final argument at object offset 0x58. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b06f40` | `—` |  | Checks the global audio manager; if an object exists at offset 0x24, calls its virtual method, otherwise returns success. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b06f60` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b06f90` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07000` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07040` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07070` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b071f0` | `—` |  | Calls FUN_0078ff90 with a stack-derived cookie, then restores the prior exception-list pointer. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b073a0` | `—` |  | Initializes an object using four dereferenced constructor arguments through the shared base initializer, then returns this. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b077c0` | `—` |  | Calls FUN_0078ff90 with a stack-derived cookie, then restores the prior exception-list pointer. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07890` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b078d0` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07910` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07950` | `— → ResumeSoundEmitterById` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b07980` | `—` |  | Searches emitters by id and returns whether the matching node?s active field is nonzero. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b079b0` | `—` |  | Unlinks an emitter node from the doubly linked list, decrements count, releases its reference, then calls the list node free slot. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b08070` | `—` |  | Local sound or emitter control. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `0123ac60` | `— ⚠reject` |  | Builds three byte-sized local flags with their addresses, calls FUN_0123afb0 with them and the this value, returns the this pointer and pops two stack arguments (ret 8). | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

