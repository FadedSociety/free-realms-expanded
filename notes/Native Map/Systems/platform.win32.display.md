# System: platform.win32.display

37 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440569` | `_InitMultipleMonitorStubs` |  | Windows SDK multimon.h stub initialiser (decompiler labels it 'Library Function - Single Match', Visual Studio 2005) backing the x* wrappers in this shard. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004407b3` | `xGetMonitorInfoA` |  | Windows SDK multimon.h helper (the target of the GetMonitorInfoA stub pointer resolved in 00440569). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440931` | `xEnumDisplayDevicesA` |  | Windows SDK multimon.h helper (the target of the EnumDisplayDevicesA stub pointer resolved in 00440569). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004409bc` | `InitMonitorApiFromUser32` |  | App-specific monitor helper initialiser distinct from the multimon.h stubs; its caller 0044E41B uses the resolved pointers for monitor placement. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0045b7bd` | `LoadGetMonitorInfo` |  | Lazy import of GetMonitorInfoA used for monitor/desktop queries on the Windows platform layer (caller 00474642). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045bb27` | `EnsureDirectDraw7` |  | DirectDraw bootstrap of the Direct3D 9 renderer path (caller 0045bb76). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0046cef2` | `GDisplayD3D9Impl::~GDisplayD3D9Impl` | GDisplayD3D9Impl | Destructor of the Direct3D 9 graphics device object; the FreeLibrary call shows it unloads a dynamically loaded module (d3d9/ddraw) on teardown. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `008dd8f0` | `ApplyDisplayWindowedSetting` |  | Applies the stored Display/Windowed configuration value to the window mode; the exact strings identify the config section and value. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `00915a90` | `ResolveAndWriteDisplaySettings` |  | Chooses the fullscreen resolution (the probe result, or 1024x768 as the fallback) and writes the fullscreen and windowed size and refresh settings under Display. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `009173b0` | `ReadDisplayMaximizedConfig` |  | Reads the Display/Maximized config flag, the flag ApplyMaximizedDisplaySetting (0091e000) consumes. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `009179c0` | `StoreDisplayResolutionConfig` |  | Saves the chosen resolution into the fullscreen or windowed Display config keys, skipping the windowed size while the window is maximised. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `0091e000` | `ApplyMaximizedDisplaySetting` |  | Keeps the persisted Display/Maximized option in sync with the actual window state; driven by the client virtual 0091e710. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091e710` | `BaseClient::SyncMaximizedState` | BaseClient | Keeps the Display/Maximized setting aligned with the real maximized state of the game window. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `00922c10` | `InitDeviceWithFallbacks` |  | Graphics device bring-up run by PlayClient::Initialize (0092b170); it writes the InitDevice failure messages to FreeRealms.log. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440525` | `IsWin32NtPlatform` |  | Platform check used by the multimonitor stub initialiser 00440569 to record whether the NT platform is running. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440748` | `xMonitorFromWindow` |  | Windows SDK multimon.h helper (decompiler symbol _xMonitorFromWindow@8) used to place windows on the right monitor. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440846` | `xEnumDisplayMonitors` |  | Windows SDK multimon.h helper (decompiler symbol _xEnumDisplayMonitors@16). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440cf5` | `CopyDescRetainString` |  | Copies a display/device descriptor including its trailing string handle; used by 00445CFE and 0044FD43 (outside this shard). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0045bb06` | `DdrawModuleClose` |  | Shutdown of the DirectDraw module and interface (callers 0046cef2, 004783f1). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045bb76` | `QueryAvailableVideoMemory` |  | Video-memory query feeding the device block of the accepted ProbeDeviceCaps (0046340f), which stores the pairs next to its D3DCAPS9 data. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0046cf8c` | `GDisplayD3D9Impl::BlitImage` | GDisplayD3D9Impl | Blits a source surface into the display client area with clipping and dirty-rect maintenance; occupies the display vtable slot at 0x017651b8. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046d05e` | `GDisplayD3D9Impl::PresentTrackedRect` | GDisplayD3D9Impl | Presents the display pending dirty rectangle; occupies the display vtable slot at 0x017651c0. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046d0c1` | `GDisplayD3D9Impl::PresentRect` | GDisplayD3D9Impl | Presents one caller-supplied rectangle; occupies the display vtable slot at 0x017651bc. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046d9ed` | `GDisplayD3D9Impl::DtorThunk` | GDisplayD3D9Impl | MSVC scalar deleting destructor of the Direct3D 9 display object; it is the slot at 0x01765040, the vtable 00476b89 installs and FUN_0046cef2 restores. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `008ed400` | `DisplaySetWindowedMode` |  | Switches the display between windowed and fullscreen and records a pending-change flag at this+0x870. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008ed480` | `DisplayToggleWindowed` |  | Toggles windowed and fullscreen display mode. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |

## ⚪ unknown (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0044065e` | `— ⚠reject` |  | Windows SDK multimon.h helper (decompiler symbol _xGetSystemMetrics@4) used by the client's screen/rect helpers such as 00440C47. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004406a6` | `— ⚠reject` |  | Windows SDK multimon.h helper (decompiler symbol _xMonitorFromPoint@12). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004406fc` | `— ⚠reject` |  | Windows SDK multimon.h helper (decompiler symbol _xMonitorFromRect@8); also called by xMonitorFromWindow 00440748. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440c47` | `— ⚠reject` |  | Screen-bounds helper of the Win32 window layer (framing/window placement math). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0046cac7` | `— ⚠reject` | GDisplayD3D9Impl | Device-lost/reset handler of the Direct3D 9 display implementation; the return codes are the 'display status' values its caller reports. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046cb80` | `— ⚠reject` | GDisplayD3D9Impl | Resolution/backbuffer change of the D3D9 display: resizes the viewports, re-creates the render targets and re-binds dependent resources. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046cd73` | `— ⚠reject` | GDisplayD3D9Impl | Startup/focus diagnostic of the Direct3D 9 display: decides whether the display can render and produces the exact status text; occupies the display vtable slot at 0x017650a8 (the table the destructor 0046cef2 restores). | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046ce78` | `— ⚠reject` | GDisplayD3D9Impl | Releases one interface registered with the display and removes its registry entry; occupies the display vtable slot at 0x017650d8. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `009159e0` | `— ⚠reject` |  | Writes the Display mode (Windowed or Fullscreen, as reported by FUN_00f9a430) and clears the Display/Maximized flag. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00915a30` | `— ⚠reject` |  | Writes the fullscreen refresh (0), width and height settings under Display via FormatThenStore96d0. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00915b80` | `— ⚠reject` |  | Loads the Display settings (mode, plate and social display, hover, maximized, fullscreen and windowed size and refresh) and reports them to the log. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |

