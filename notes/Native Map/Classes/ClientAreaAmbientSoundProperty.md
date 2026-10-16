# Class: ClientAreaAmbientSoundProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bed720` | `ClientAreaAmbientSoundProperty::GetTrackId` | ✅ | virtual | Loads and returns the 32-bit value stored at object offset 0x38. |
| `00ceb920` | `ClientAreaAmbientSoundProperty::Log ?` | 🟡 | virtual | Logs inherited fields plus TrackId, SoundDefinitionId, FinishPlaying, RetriggerDelay, and floorTolerance. |
| `00ceb9b0` | `— ⚠reject` | ⚪ | method | Reads TrackId and SoundDefinitionId into the property and conditionally updates finish-playing, retrigger delay, and floor-tolerance fields. |
| `00cebd20` | `ClientAreaAmbientSoundProperty::Update ⚠downgrade` | 🟢 | virtual | When enabled, resolves the configured sound definition and starts or stops playback based on its state and client audio flag. |
| `00cebd80` | `ClientAreaAmbientSoundProperty::Enable` | ✅ | virtual | Increments the referenced sound definition, sets the enabled byte, then runs the ambient playback start routine. |
| `00cebdc0` | `— ⚠reject` | ⚪ | virtual | Stops active track when the area audio system is available, clears the playing byte, and decrements the sound-definition reference. |
