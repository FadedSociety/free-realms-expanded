# System: platform.win32.window

28 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440b17` | `RegisterGWindowManagerWin32` |  | Registers the Win32 window-manager implementation class with the engine's class/type registry. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441141` | `RegisterGWindowWin32` |  | Registers the Win32 window implementation class (the class whose methods populate 004411AE .. 00441E0E) with the engine's class registry. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0091d9f0` | `CreateClientWindow` |  | Startup path that brings up the client window and display; called once from the client bootstrap 0092b170. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |

## 🟢 likely (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440997` | `PostOrSendMessage` |  | Used by the window code in this shard to post/send window messages from non-window-proc context (callers 00441E0E and 00449F5B). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440b84` | `DefWindowProcWithFilter` |  | Fallback window-procedure helper that offers selected message IDs to the installed filter, then forwards unclaimed messages to the configured A/W default procedure. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440bff` | `GetSetPair58_5C` |  | Small get/set property pair on a Win32 layer object (window style words or similar); no in-shard caller. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440d6a` | `ShowMessageBox` |  | Fatal/confirmation dialog helper of the Win32 platform layer (MB_TASKMODAL message box). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004411ae` | `GetSetCursor` |  | Cursor management of a GWindowWin32 object (the +0x34 field is the HWND used by all its methods). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441606` | `SetCursorPosition` |  | Cursor warping for the Win32 window object (relative wParam from a raw-input/UI path). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004416a1` | `CaptureMouse` |  | Acquire Win32 mouse capture for the window (drag/UI interaction support). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004416ca` | `ReleaseMouseCapture` |  | Release Win32 mouse capture held by the window. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441711` | `SetWindowText` |  | Caption/title setting on the Win32 window object, honouring the engine-wide unicode flag. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004417bb` | `GetWindowText` |  | Caption/title retrieval on the Win32 window object. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0044185d` | `InvalidateWindowRect` |  | Repaint request for the Win32 window object. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004418b8` | `ValidateWindowRect` |  | Validates (removes from the update region) part or all of the Win32 window. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441940` | `ClientToScreenPoints` |  | Coordinate conversion of a point pair (e.g. a rect) from client to screen space for the Win32 window. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0044199b` | `ScreenToClientPoints` |  | Coordinate conversion of a point pair from screen to client space for the Win32 window. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441a31` | `GetWindowBorderInsets` |  | Computes the window border/caption insets of the Win32 window (used when converting between client and frame rectangles). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441d7f` | `GetWindowFlags` |  | Returns the current window style/state flags of the Win32 window by decoding the live Win32 style words. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00441e0e` | `RestoreAndShowWindow` |  | Brings the Win32 window back to restored/visible state (de-iconify path). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004424b5` | `ComputeMessageWaitTimeout` |  | Produces the millisecond wait budget for the message loop when the queue is idle (companion of RunMessageLoop 00442335). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00445a6d` | `ShowWindowFromState` |  | State-driven ShowWindow path of the GFC Win32 window objects (minimise/maximise/show). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445af1` | `SetFocusIfNotActive` |  | Focus management of the GFC Win32 window objects, including a re-entrancy/ownership bit held in the owner's state dword around the SetFocus call. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445cfe` | `UpdateCachedWindowRect` |  | Keeps the cached geometry of the Win32 window object up to date after the window moved or resized. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `008b8f30` | `SetClientWindowEmbedMode` |  | Switches the client window between embedded and unparented display modes. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00916f90` | `ApplyWindowPlacement` |  | Applies the stored window placement to the client window and brings it to the foreground. The inputs come from the FUN_00915b80 state reader. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00442335` | `— ⚠reject` |  | Main Win32 message loop with IME awareness: it is the only core function here that links against IMM32 (ImmIsUIMessageA) and it drives the window-proc of all client windows. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `008a8ba0` | `— ⚠reject` |  | Updates the stored foreground state and emits a one-call edge when focus changes from absent to present; this+0x941 remains set while focused, while this+0x942 is cleared on the next focused call. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |

