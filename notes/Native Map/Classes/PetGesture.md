# Class: PetGesture

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc7930` | `PetGestureDtor` | ✅ | dtor | Installs the PetGesture vtable, destroys tracked effects and both Vector2 arrays, frees their backing buffers, and releases gesture-owned runtime data. |
| `00cc8760` | `PetGestureScalarDeletingDtor` | ✅ | dtor | Calls PetGesture destructor and, when deleting flag is set, frees the object. Bytes confirm ordinary call wrapper followed by return. |
| `00cc9020` | `PetGestureCtor` | ✅ | ctor | Installs PetGesture vtable, initializes handler fields and two Vector2 arrays, sets effect and locale ids plus gesture limits, then loads PetTrickGestureSettings.txt. |
| `00cc9bb0` | `LoadPetGestureDataAndSettings` | ✅ | virtual | For points file, parses caret-delimited trick and sequence ids plus float x/y into per-trick Vector2 arrays; for settings file, parses keys and writes effect ids, thresholds, timings, and locale ids to fields. |
| `00cca3f0` | `PetGestureSetTerminalState ? ⚠rename` | 🟡 | virtual | Writes value eight to PetGesture state field +0x60 and returns. Bytes show the field store and cookie epilogue. |
