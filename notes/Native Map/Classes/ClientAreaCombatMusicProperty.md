# Class: ClientAreaCombatMusicProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00becaa0` | `ClientAreaCombatMusicProperty::Hash ⚠downgrade` | 🟢 | virtual | Reads the property string through vtable slot +0x28, applies the 0x401 byte hash recurrence, and calls FUN_00705bc0 for long strings. |
| `00bed070` | `ClientAreaCombatMusicProperty::TypeName ⚠downgrade` | 🟢 | virtual | Returns a pointer to the literal property type name CombatMusic. |
| `00bed080` | `ClientAreaCombatMusicProperty::ScalarDeletingDtor` | ✅ | dtor | Calls the class destructor, conditionally frees the object when flag bit 0 is set, and returns the object pointer otherwise. |
| `00bed0a0` | `ClientAreaCombatMusicProperty::GetTrackId` | ✅ | virtual | Loads and returns the 32-bit value stored at object offset 0x38. |
| `00bed0b0` | `ClientAreaCombatMusicProperty::SetTrackId` | ✅ | virtual | Stores the supplied 32-bit value at object offset 0x38 and returns. |
| `00ced770` | `ClientAreaCombatMusicProperty::WriteFields` | ✅ | virtual | Writes base property fields and the SoundDefinitionId value. |
| `00ced7c0` | `ClientAreaCombatMusicProperty::ReadSoundDefinitionId` | ✅ | virtual | Reads SoundDefinitionId through the property reader and stores the resulting ID at object offset 0x38. |
| `00ced7f0` | `ClientAreaCombatMusicProperty::Start` | ✅ | virtual | Increments the referenced sound definition, resolves the global combat-music state, clears its active byte, stores this sound ID, and calls the music update routine. |
| `00ced860` | `ClientAreaCombatMusicProperty::Stop ⚠downgrade` | 🟢 | virtual | Decrements the referenced sound-definition count, resolves the global combat-music state, clears its active byte, and zeroes the selected sound ID. |
