# Class: ClientMiniGameManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b0c70` | `ClientMiniGameManager::UpdateStateFlags` | ✅ | virtual | Compares two stored state IDs with param_2 and sets or clears byte flags 0x40 and 0x80 based on param_3+4. |
| `009b8f90` | `ClientMiniGameManager::HandleGameDataUnlockedPacket` | ✅ | handler | Parses the message: shared 5-field header (FUN_009b2340), a 32-bit length-prefixed blob (FUN_008bfc00), and then three counted sections read from inside that blob - game data (FUN_009b1740, allocated with FUN_009af660),… |
| `009bbba0` | `— ⚠reject` | ⚪ | dtor | Destructor: restores both ClientMiniGameManager vtables, releases the raw table pointer at +0x2bea4 (FUN_00c43440 then free), tears down the config pieces (FUN_009b0fd0, FUN_009b42b0, FUN_009ae2b0, FUN_009b1110, FUN_009… |
| `009bc410` | `ClientMiniGameManager::ClientMiniGameManager` | ✅ | ctor | Constructor: chains GuiEventHandler and DelayedAssetLoader::StatusHandler vtables into ClientMiniGameManager::vftable, stores the owner client at +8, initialises the state HashList (009bbfc0), creates the mini-game UI s… |
| `009bca60` | `ClientMiniGameManager::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the client mini-game manager: calls the manager destructor 009bbba0, which resets the ClientMiniGameManager vtables and tears down its tables, sub-interfaces and state list. Then frees the… |
| `009bf5a0` | `UpdateMiniGameManagerState` | ✅ | virtual | Sets manager state to 3, runs state cleanup, and requests mini-game start when manager flag bit 3 is set. |
