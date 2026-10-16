# System: platform.win32.input

61 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440428` | `InitMouseDeviceEntry` |  | Descriptor initialiser for the mouse device; the sibling 004432C0 does the same with the name L'Keyboard'. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00442302` | `RegisterGIODeviceSourceWin32` |  | Registers the Win32 input-device source implementation class with the engine's class/type registry. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004426b5` | `WindowsKeyToKeysym` |  | The client's key representation mapper: the vault note 'Native Combat Controls' cites this exact address as the original Windows-to-keysym mapper behind the default keybinding table. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004432c0` | `InitKeyboardDeviceEntry` |  | Descriptor initialiser for the keyboard device; the sibling 00440428 does the same with the name L'Mouse'. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00445f20` | `DeviceKindRecordAndName` |  | Builds a device descriptor and its display name; the interned strings are the manager and device names (Input Manager, Window Manager, System/Virtual/Window-Manager Keyboard and Mouse). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0091e500` | `BaseClient::HandleDebugHotkey` | BaseClient | Debug/screenshot hotkeys used by the client shell (HideUI toggle, minimap toggle, screenshot). | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `009c0560` | `CaptureMousePointer` |  | Begins Windows mouse capture for the supplied window. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c0600` | `ReleaseMousePointerCapture` |  | Ends Windows mouse capture after all capture flags clear. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009cef40` | `PollKeyboardStateBuffers` |  | Maintains double-buffered Windows keyboard state. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cf1c0` | `InitializeKeyboardActionBindings` |  | Builds the keyboard action-state table. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |

## 🟢 likely (46)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004403d0` | `BuildDeviceObjectTable` |  | Fills the per-device object table used by the Win32 input device layer (device object enumeration); its caller 00445341 is an input-device setup function. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441255` | `GetKeysymModifierState` |  | Reports which modifier keys are currently held, expressed with X11-style keysym codes; the keysyms match the table used by the Windows-to-keysym mapper 004426B5. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441bd9` | `EncodeWindowFlags` |  | Converts the client's internal window/input flag mask into the two dwords consumed by the Win32 style setters; inverse direction of 00441CD3. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441eb9` | `ImeGetCompositionText` |  | Reads the active IME composition string (composition / reading / result variants) from the window's input context; the pointer table is the IMM32 API resolved at startup. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441fc3` | `ImeGetCursorPosition` |  | Computes the caret index of the active IME composition (used to position the composition/candidate UI); the byte scan over COMPATTR values 1/2 mirrors ATTR_TARGET_CONVERTED/NOTCONVERTED. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0044205e` | `ImeSelectCandidate` |  | Selects an entry in the IME candidate list (selector 0x12 matches NI_SELECTCANDIDATESTR of the notification interface used here). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004424f8` | `GetAsyncKeyFlags` |  | Reports down/transition state for a single key (used by the input layer's vtable +0x8C dispatch at 0044547C). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0044262d` | `ResetKeyStateTables` |  | Initial state reset of the input-device object; callers 0044F83C / 0044F8B9 / 0044F937 are input source setup paths. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00442659` | `GetModifierFlags` |  | Fast synchronous or asynchronous modifier-key mask of the Win32 input layer (caller 0044547C, the device code that also uses GetAsyncKeyFlags). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00442c28` | `KeysymToWindowsKey` |  | Conversion of internally stored keysyms back to Win32 virtual keys; it is the exact inverse of the note-cited mapper 004426B5 and pairs with the action-key dispatch path documented in 'Native Combat Controls'. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004432fd` | `InitKeyboardDeviceEntryEx` |  | Full keyboard-device record setup (the 0x81 field likely selects keyboard capabilities); caller 00445F20. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00443328` | `TranslateKeyIndexToKeysym` |  | Resolves a key index (probably the DirectInput/scan index) into the client's keysym space with the current Shift/CapsLock/NumLock state applied, for text/action input. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00445341` | `InitMouseStateKind5` |  | Initialises the Win32 mouse device state block (button count, wheel flag, cursor snapshot, device back-pointer) for the kind-5 initialiser path; the accepted map labels its three callers 0044fae8/0044fb65/0044fbe6 as Wi… | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004453c1` | `SetMouseActiveFlag` |  | Setter for the object's mouse-active byte; it also notifies the listener chain when the flag changes (callers include the window detach path 0044e1d3 and the mouse poll 0044547c). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044543a` | `InitMouseDeviceEntryEx` |  | Mouse counterpart of InitKeyboardDeviceEntryEx (004432fd): completes the device descriptor that the input manager publishes for the system mouse. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044547c` | `UpdateMouseState` |  | Per-frame update of the Win32 mouse state (buttons, cursor deltas, wheel, modifier mask) in the GFC input layer; no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445550` | `SetMouseActiveMode` |  | Mode-based activation wrapper around the flag setter 004453c1 (2 = set, 3 = clear then set, 4 = clear, anything else returns 0). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445695` | `FormatKeyDisplayName` |  | Formats a display name for a device key/axis entry: the axis-table name when the descriptor's type byte is 2, otherwise L'Button %d' with the entry's second byte. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004457c5` | `IteratorFactoryAt70_140` |  | Builds a checked iterator/end marker for the table owned by the object (owner field +0x70, marker at +0x74 + 0x140, the only literal in the body); no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445957` | `IteratorFactoryAt100_140` |  | Iterator factory for the table owned by the object (owner at +0x100, marker at +0x104 + 0x140), gated by the alive byte at +0x3c; no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044598d` | `BuildMouseButtonMask` |  | Packs the object's per-button state into a bitmask (0x10000 << index) for the caller; the query goes through virtual slot +0x88 and the button count through xGetSystemMetrics. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004459d7` | `GetCursorPosition` |  | Cursor-position query used by the Win32 window/input objects: the caller supplies the two-dword destination and chooses screen or client coordinates. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00446152` | `GetKeyStateModifierFlags` |  | Modifier mask read for the keyboard device object (the twin of GetModifierFlags at 00442659, which uses GetKeyState instead of the owned state table). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00446181` | `SetKeyboardActiveFlag` |  | Keyboard counterpart of the mouse-active flag setter (004453c1): it changes the device activation byte and notifies the event chain. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00446278` | `SetKeyboardActiveMode` |  | Keyboard family version of the mode-based activation wrapper around 00446181 (2 = set, 3 = clear then set, 4 = clear). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004462c2` | `IteratorFactoryAt150_81` |  | Iterator factory over the device table at +0x150 (0x81 entries) of the object; the +0x144 field gates it. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044645c` | `FormatKeyNameFromKeysym` |  | Key-name formatting through the virtual-key/keysym translation used by the input manager; no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044650d` | `GetKeyDownAsFloat` |  | Boolean-to-float key-state query over the device object (this+4 holds the device); no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `008a8cb0` | `Wrapper_009c05d0` |  | Enters the exclusive input mode through the sub-object at this+0x644; called from 00af8fc0 together with the exit twin 008a8cc0. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008a8cc0` | `Wrapper_009c0670` |  | Leaves the exclusive input mode through the sub-object at this+0x644. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008a8cd0` | `Wrapper_009c04f0_HideCursor` |  | Hides the OS cursor through the input sub-object at this+0x644; called from 0094fe10 next to the show twin. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008a8ce0` | `Wrapper_009c04f0_ShowCursor` |  | Shows the OS cursor again through the input sub-object at this+0x644. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `009c03d0` | `InitializeInputCaptureState` |  | Initializes state used by the Windows pointer-capture and mouse-input paths. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c04f0` | `SetCursorVisibilityAndRestorePosition` |  | Hides or restores the Windows cursor while input capture is active. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c05d0` | `CaptureMouseWithTemporaryOwner` |  | Transfers pointer capture while retaining the original owner value. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c0670` | `ClearTransferAndReleaseCapture` |  | Completes temporary capture transfer and releases mouse capture when eligible. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c0680` | `DecodeWindowsMouseMessage` |  | Translates Windows mouse messages into engine input events. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c0a40` | `DispatchWindowsInputMessage` |  | Converts selected Windows input messages into the engine's internal input queue. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009cefe0` | `ReleaseKeyboardActionStrings` |  | Releases cached key-action strings during keyboard state cleanup. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cf070` | `UpdateKeyboardActionStates` |  | Applies keyboard state transitions to registered action objects. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cf320` | `InitializeKeyboardActionsAndReturnArgument` |  | Wrapper used during PlayClient initialization. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00c168f0` | `SetKeyboardEnabled` |  | Enables or disables keyboard input for the client; the name is the table row 0x0182e060 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16960` | `SetMovementEnabled` |  | Enables or disables player movement input; the name is the table row 0x0182e050 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c169d0` | `GetDefaultKeyBinding` |  | Resolves a named action to its default key binding for the key-binding UI; the name is the table row 0x0182e0a0 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16b30` | `AddKeyBinding` |  | Adds a key binding for an action from the options UI (action name, key name, optional mode below 3, optional modifier flag); the name is the table row 0x0182e070 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16cc0` | `SetKeyboardIntercept` |  | Sets which UI element intercepts keyboard input; the name is the table row 0x0182e0a8 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00442096` | `ImeSetCandidatePageSize ? ⚠rename` |  | Sets the IME candidate page size (selector 0x16 matches NI_SETCANDIDATE_PAGESIZE); sibling of ImeSelectCandidate 0044205E. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `009cefd0` | `SetKeyboardStateCacheFlag ?` |  | Stores the supplied byte at this+0x308 and returns. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00441cd3` | `— ⚠reject` |  | Recovers the client's internal flag mask from GWL_STYLE/GWL_EXSTYLE; used by GetWindowFlags 00441D7F and by the inverse encoder 00441BD9. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441f59` | `— ⚠reject` |  | Sets the composition string/reading of the window's IME context (the counterpart of ImeGetCompositionText). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00902cc0` | `— ⚠reject` |  | Input message pump for the game window: opens the debug console from the 0x7e key and chat from the 0x3f key, and routes other input to the component message routers. Caller 0094a970 is unlabelled. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |

