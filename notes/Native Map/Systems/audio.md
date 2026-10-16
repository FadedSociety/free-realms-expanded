# System: audio

32 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008aa860` | `HashListMap_SoundControlDefinition_Construct` | SoeUtil::HashListMap | Bucket map of sound-control definitions in the audio definition store. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008aa890` | `HashMap_NameEntry_SoundControlDefinition_Construct` | SoeUtil::HashMap | Name index paired with 008aa860 in the sensory-effect/audio definition manager. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008aadb0` | `HashMap_PendingOneShotEntry_Construct` | SoeUtil::HashMap | Initialises the pending-load table of the one-shot sound definition loader. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `0091e750` | `ApplySoundVolume` |  | Applies the user sound-volume setting to the audio engine at startup/settings-apply time. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `00b0adc0` | `ClientAreaData::RemoveAndResetBackgroundAudio` | ClientAreaData | Removes an area node and clears missing background sound or music references. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00c1bb40` | `SoundGroup::Ctor` | SoundGroup | Initializes a SoundGroup object and its sound-definition array. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1bc80` | `SoundGroup::ScalarDeletingDtor` | SoundGroup | Thunk for SoundGroup scalar deleting destruction through an adjusted base subobject. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1bc90` | `SoundGroup::ScalarDeletingDtor` | SoundGroup | Thunk for SoundGroup scalar deleting destruction through an adjusted base subobject. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1da10` | `SoundGroup::Dtor → SoundGroup::DtorBody` | SoundGroup | Destroys SoundGroup state and releases its contained sound definitions. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1db60` | `SoundGroup::ScalarDeletingDtor` | SoundGroup | Scalar deleting destructor for SoundGroup. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00ca4b50` | `InitializePirateBattleSoundEvents` |  | Binds pirate battle treasure and pickup audio events to runtime objects. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |

## 🟢 likely (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a59810` | `CreatePirateBattleVocalSoundEmitters ⚠downgrade` |  | Create and attach the happy and sad pirate-battle vocal sound emitters. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00ad1eb0` | `CreateTcgAudioEmitter ⚠downgrade` | ClientTCGInterface | Creates or updates the sound-emitter record used by this TCG interface callback. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00b9d2d0` | `CreateSoundEmitterForEvent` |  | Per-event emitter creation step of the race sound setup: 00b9f360 calls it once for each of the 24 Vehicle_* events; 0078fea0 is the SoundEmitter factory (its RTTI vtable is 017ed770). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00c15cc0` | `SoundLoadSucceeded` |  | Reports to scripts whether a named sound finished loading; the name is the table row 0x0182df78 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15e60` | `PauseSound` |  | Pauses a playing sound by id; the name is the table row 0x0182df68 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15eb0` | `StopSound` |  | Stops a playing sound by id; the name is the table row 0x0182df60 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15f00` | `ResumeSound` |  | Resumes a paused sound by id; the name is the table row 0x0182df70 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c160d0` | `StopSoundEmitter` |  | Stops a sound emitter; the name is the table row 0x0182dfa8 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16120` | `PauseSoundEmitter` |  | Pauses a sound emitter; the name is the table row 0x0182dfb0 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16170` | `ResumeSoundEmitter` |  | Resumes a paused sound emitter; the name is the table row 0x0182dfb8 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c1bd70` | `SoundGroup::FindEntryPointerIndex` | SoundGroup | Finds an entry pointer in the SoundGroup array; the SoundGroup role is supported by its caller passing the group object to this scan. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1d210` | `SoundGroup::ClearEntries` | SoundGroup | Releases the current SoundGroup entries and clears its count. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1d2f0` | `SoundGroup::Remove ⚠downgrade` | SoundGroup | Removes and releases matching sound-group entries. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1dac0` | `SoundGroup::SetEntryAtIndex → SoundGroup::InsertEntryAtIndex` | SoundGroup | Associates a referenced entry with a SoundGroup array index. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1db30` | `SoundGroup::SetEntryAtIndex` | SoundGroup | SoundGroup virtual slot 5 delegates registration and indexed storage; the external registration is conditional. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ceb920` | `ClientAreaAmbientSoundProperty::Log ?` | ClientAreaAmbientSoundProperty | Logs inherited fields plus TrackId, SoundDefinitionId, FinishPlaying, RetriggerDelay, and floorTolerance. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0090c8b0` | `— ⚠reject` |  | Checks sound state at owner+0x67010 and +0x67014, invokes a virtual method on the active sound object, advances a timer, then releases the sound allocation when its float field equals the global constant. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `00b62f80` | `— ⚠reject` |  | Resolves a track id to a live SoundEmitter and parks it in the caller's field; FUN_00b65d90 calls it once per MG_Fishing_* track name while setting up the fishing music. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00c15da0` | `— ⚠reject` |  | Adjusts the volume of a sound or emitter with an optional second parameter (ramp/fade); the name is the table row 0x0182df58 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15f50` | `— ⚠reject` |  | Adjusts the pitch of a sound emitter; the name is the table row 0x0182dfa0 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16010` | `— ⚠reject` |  | Adjusts the volume of a sound emitter; the name is the table row 0x0182df98 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

