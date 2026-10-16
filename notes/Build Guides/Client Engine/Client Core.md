---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Client Core

Part of [[Build Guides]] › Client Engine. Related: [[Command System]], [[UI Framework (GFC and XML)]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

2448 accepted functions: 566 confirmed, 889 likely, 709 speculative, 284 unknown. Network-linked (`wire`): 17.

Subsystems: `engine.serialization` (629), `engine.client` (534), `platform.win32` (174), `engine.valuedata` (145), `platform.win32.input` (125), `engine.log` (113), `debug` (104), `platform.win32.file` (74), `engine.service` (72), `debug.exception` (72), `engine.messaging` (58), `debug.freeform_actor` (47), `platform.win32.display` (43), `debug.console` (36), `platform.win32.window` (31), `engine.persistent` (25), `engine.steam` (19), `engine.statemachine` (17), `engine.event` (16), `debug.trace` (16), `debug.profiler` (15), `platform.time` (12), `platform.udp` (9), `platform.win32.clipboard` (8), `debug.userdata_command` (8), `platform` (6), `debug.watchdog` (6), `engine.factory` (6), `engine.loop` (4), `debug.metrics` (3), `platform.win32.shell` (3), `debug.assert` (3), `platform.win32.gdi` (2), `engine.ui` (2), `debug.itemlist` (2), `debug.actor` (2), `debug.physics` (2), `platform.win32.heap` (2), `debug.udp` (1), `debug.claim` (1), `debug.item` (1)

_Large or library guide: only confirmed and likely functions are listed (1455 of 2448)._

### `debug`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008efda0` | BaseClient::FatalErrorForModeCode | likely | Virtual method of PlayClient and BaseClient (ECX unused; two stack arguments: mode, message; ret 8). Maps mode 0 to code 0x14, mode 1 to 0x13, mode 2 to 0x15, … |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008de160` | PlayClient::DumpDiagnostics | confirmed | Virtual method (slot 1 of PlayClient/BaseClient): writes the exact line Version: %s with the build string 1.910.1.530630 from PTR_s_1_910_1_530630_01ba6fe8 int… |

**(no class)** (92)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dddf1` | DebugDumpFileLength | confirmed | Continues the module version dump: opens the file through 00703e10, on success fetches its size with 00702f60 and logs the exact line [version %s] Length=%lld … |
| `008dde8a` | DebugDumpFileCreationTime | confirmed | Builds a time record from the shared file handle state and calls 007026b0 to expand it, then logs the exact line [version %s] CreationTime %02d/%02d/%02d %02d:… |
| `008ddf2c` | DebugDumpFileModifiedTime | confirmed | Expands the modification timestamp of the shared file handle through 007026b0 and logs the exact line [version %s] ModifiedTime %02d/%02d/%02d %02d:%02d:%02d, … |
| `008ddfb5` | DebugDumpFileIsDirectory | confirmed | Final step of the module version dump: logs the exact line [version %s] IsDirectory %s and then closes the file through 007034d0, releases through 00703a30/007… |
| `008deaaa` | DebugDump_IsNonattackable | confirmed | Loads the object pointer at this+0xBC and tests bit 8 of its dword at +0x6F8; when set, emits Player IsNonattackable, then calls 008deb0f. |
| `008deb0f` | DebugDump_IsFalling | confirmed | Tests bit 0x40 at character+0x142; when set, formats Player IsFalling into the dump buffer and calls the next dump routine. |
| `008deb6f` | DebugDump_IsIgnoringCollision | confirmed | Tests bit 0 of the state byte at character+0x142; when set, formats Player IsIgnoringCollision and calls the next dump helper. |
| `008debcf` | DebugDump_IsJumping | confirmed | Tests bit 0x20 at character+0x142; when set, formats Player IsJumping and calls the next dump helper. |
| `008dec2f` | DebugDump_IsJuking | confirmed | Tests bit 8 at character+0x142; when set, formats Player IsJuking and calls the next dump helper. |
| `008dec8f` | DebugDump_IsInWater | confirmed | Tests bit 0x40 at character+0x143; when set, formats Player IsInWater and calls the next dump helper. |
| `008decef` | DebugDump_IsSwimming | confirmed | Tests bit 0x20 at character+0x143; when set, formats Player IsSwimming and calls the next dump helper. |
| `008ded4f` | DebugDump_IsMoving | confirmed | Calls the predicate at 008cfff0 and, when it returns nonzero, formats Player IsMoving before continuing the dump. |
| `008dedae` | DebugDump_MicrophoneAttached | confirmed | Tests the nonzero field at character+0x804; when set, formats MicrophoneAttached and continues the dump. |
| `008dee0d` | DebugDump_Gender | confirmed | Reads the dword at character+0x6e8, formats it with Gender: %d, and continues the dump. |
| `008dee6a` | DebugDump_ModelCustomizationName | confirmed | Reads the pointer at character+0x5d4, formats it with GetModelCustomizationName: %s, and continues the dump. |
| `008deeca` | DebugDump_HairModelName | confirmed | Reads the pointer at character+0x5a4, formats it with GetHairModelName: %s, and continues the dump. |
| `008def2a` | DebugDump_HeadModelName | confirmed | Reads the pointer at character+0x5b4, formats it with GetHeadModelName: %s, and continues the dump. |
| `008def86` | DebugDump_SkinToneName | confirmed | Calls a virtual getter at object+0x54 slot offset 0x30 using a temporary string, formats the returned string with GetSkinToneName, destroys the temporary, then… |
| `008df000` | DebugDump_FacePaintName | confirmed | Calls a virtual getter at object+0x54 slot offset 0x34 using a temporary string, formats the returned string with GetFacePaintName, destroys the temporary, the… |
| `008df07a` | DebugDump_TerrainObjectId | confirmed | Reads the dword at character+0x824, formats it as TerrainObjectId: %u, and continues the dump. |
| `008df0d9` | DebugDump_Heading | confirmed | Loads two orientation components at character offsets 0x610 and 0x618, computes fpatan, formats Heading: %f, then continues. |
| `008df158` | DebugDump_VerticalVelocity | confirmed | Reads the float at character+0x674, formats GetVerticalVelocity: %f, and continues the dump. |
| `008df1bb` | DebugDump_VerticalOffset | confirmed | Reads the float at character+0x670, formats GetVerticalOffset: %f, and continues the dump. |
| `008df21e` | DebugDump_DistanceToGround | confirmed | Calls 009584f0, formats its returned floating-point value with DistanceToGround: %f, and continues. |
| `008df281` | DebugDump_PlayerBlendTimeSec | confirmed | Reads the float at character+0x684, stores it in a stack temporary, formats GetPlayerBlendTimeSec, and continues. |
| `008df2ef` | DebugDump_WaterHeight | confirmed | Reads the float at character+0x424, formats GetWaterHeight: %f, and continues. |
| `008df35d` | DebugDump_WaterDepth | confirmed | Reads the float at character+0x428, formats GetWaterDepth: %f, and continues. |
| `008df3cb` | DebugDump_WaterDisplacement | confirmed | Calls 0095e120, formats its floating-point return with GetWaterDisplacement: %f, and continues. |
| `008df42e` | DebugDump_Position | confirmed | Calls the object virtual at vtable offset 0x1c for a float4 pointer and formats its four components as GetPosition, then continues. |
| `008df4af` | DebugDump_Facing | confirmed | Reads four floats at character+0x610..0x61c, formats them with the GetFacing label, and continues. |
| `008df52e` | DebugDump_LookDirection | confirmed | Reads four floats at character+0x630..0x63c, formats them with GetLookDirection, and continues. |
| `008df5ac` | DebugDump_Velocity | confirmed | Reads four floats at character+0x640..0x64c, formats them with GetVelocity, and continues. |
| `008df62b` | DebugDump_GroundNormal | confirmed | Reads four floats at character+0x400..0x40c, formats them with GetGroundNormal, and continues. |
| `008df6aa` | DebugDump_GroundPosition | confirmed | Reads four floats at character+0x410..0x41c, formats them with GetGroundPosition, and continues. |
| `008df7a0` | DebugDump_ZoneGeometry | confirmed | Formats the zone geometry pointer at object+0x178 into the dump buffer and continues to render diagnostics. |
| `008df7f8` | DebugDump_RenderLevel | confirmed | Checks the graphics wrapper pointer; emits a null warning and exits that branch or reads render level at wrapper+0x438, formats it, and continues. |
| `008df8be` | DebugDump_ShadowQuality | confirmed | Calls 007912f0 for a shadow-quality value, formats Shadow quality: %d, and continues the render dump. |
| `008df91b` | DebugDump_RenderLevelValue | confirmed | Reads the render-level field at wrapper+0x438, formats Render level: %d, and continues. |
| `008df979` | DebugDump_DisplayWidth | confirmed | Calls 00790d00 for display width, formats Width: %d, and continues. |
| `008df9d5` | DebugDump_DisplayHeight | confirmed | Calls 00790d10 for display height, formats Height: %d, and continues. |
| `008dfa32` | DebugDump_DisplayRefreshRate | confirmed | Calls 00790d20 for refresh rate, formats Hz: %d, and continues. |
| `008dfa8f` | DebugDump_Fullscreen | confirmed | Calls 00790d60; selects the literal true or false from its result, formats Fullscreen: %s, then continues. |
| `008dfaf9` | DebugDump_CameraPosition | confirmed | Calls 00791d20 for a float4 camera position, formats its four components with Camera position, and continues. |
| `008dfb77` | DebugDump_CameraOrientation | confirmed | Reads four orientation floats at wrapper+0x390..0x39c, formats them with Camera orientation, and continues. |
| `008dfc62` | DebugDump_DeviceCapabilities | confirmed | Checks the engine and device-interface pointers, reports null cases, or calls the device vtable at offset 0x4c to obtain MaxMrts and continues. |
| `008dfd7c` | DebugDump_MaxVertexShaderVersion | confirmed | Calls the device-interface vtable at offset 0x50, formats the result as MaxVertexShaderVersion, and continues. |
| `008dfdda` | DebugDump_MaxPixelShaderVersion | confirmed | Calls the device-interface vtable at offset 0x54, formats the result as MaxPixelShaderVersion, and continues. |
| `008dfe38` | DebugDump_TextureQuality | confirmed | Calls the device-interface vtable at offset 0x124, formats GetTextureQuality: %d, and continues. |
| `008dfe99` | DebugDump_AvailableVideoMemory | confirmed | Calls the device-interface vtable at offset 0x120, formats GetVideoMemoryAvailableAtStartup: %d, then continues. |
| `008dfef7` | DebugDump_Locale | confirmed | Reads the locale dword at object+0x6d0, formats Locale: %d, and restores the saved exception-list pointer. |
| `008ed4a0` | DumpBoneNamesToSink | confirmed | Fastcall on this. When the object at this+0xc8 is non-null and its vtable +0x24 returns nonzero, it reads a count from 0076bd00, then for each index formats "%… |
| `008efad0` | ReportCriticalError | confirmed | Stdcall-style with three stack arguments: error code (param_1), message (param_2, may be null) and exit-immediately flag (param_3). Runs one-time setup (007001… |
| `00942c80` | DebugStatsPageFormat | confirmed | Formats one debug-statistics page into a StringFixed<512> buffer, selected by the dword at this+0x838 (mode 2..0xe): frame timings (2), thread timings (3), ren… |
| `00a5c550` | InitializeDebugDataCollections | confirmed | Initializes an IString array, a ray-test-data array and a debug-line list, clearing their storage and counts. |
| `00a5c800` | CleanupDebugDataCollections | confirmed | Stores the debug-line-list vtable at +0x20 and drains that list through 00a58180; resets the ray-test array vtable at +0x10, frees its buffer at +0x14 with ali… |
| `00bf8680` | LogStandingOn | confirmed | Builds a description of the object type at +0x20: case 0 'Actor (name=%s)' (name from 0072f4a0 when +0x1c is set), case 0xe 'ActorId (name=%s, actorId=%d)' whe… |
| `00cda810` | FormatMovementDebugString | confirmed | Calls a status helper, encodes its result into an integer written through the third argument, and formats pointer, turn, speed, skid, and trailing text fields … |
| `00700dc0` | OutputDebugFormatted | likely | Formats its printf-style argument list into a StringFixed<char,512> through 00402d80 (IStringFormatV), converts the text to wide characters through 00701290 an… |
| `007e1470` | FormatHashPipeAndLog4 | likely | void, two args (a message object and a C string, which may be replaced by defaults when null or empty). Substitutes the default strings PTR_DAT_01b32b34 and PT… |
| `008de1ea` | DebugDump_PlayerGuid | likely | Writes the exact line Player GUID: %d through 00d7b7a0 using the two 32-bit halves at ESI+0x394 and ESI+0x398, then chains into 008de249. |
| `008de249` | DebugDump_ProxiedCharacter | likely | When the ProxiedCharacter pointer at client+0xbc is null it writes the exact line ProxiedCharacter is NULL! and chains to 008de2a6; otherwise it writes Charact… |
| `008de2a6` | DebugDump_CharacterNameRuntime | likely | Writes the exact line Character name: %s from client+0x3a0, calls 008ddc00 with the exact module name GFxWrap.dll to dump that DLL version, then either reports… |
| `008de360` | DebugDump_ActorDefinitionName | likely | Writes the exact line Actor definition name: %s from the string pointer at (character+0x834) through 00d7b7a0 and chains into 008de3bf. |
| `008de3bf` | DebugDump_SubTextName | likely | Writes the exact line Sub-text name: %s from the string pointer at (character+0x11c) through 00d7b7a0 and chains into 008de41f. |
| `008de41f` | DebugDump_CharacterGuid | likely | Calls the virtual method at slot +0x10 of the object at (character+0x54) to obtain the guid pair into a stack slot and writes the exact line Character GUID: %l… |
| `008de48b` | DebugDump_CharacterHashKey | likely | Loads the two dwords at character+0x78 and character+0x7c, XORs them and writes the exact line Character hash key: %d through 00d7b7a0, then chains into 008de4… |
| `008de4f0` | DebugDump_ModelId | likely | Writes the exact line Model ID: %d from the dword at character+0x6ec through 00d7b7a0 and chains into 008de54d. |
| `008de54d` | DebugDump_HairTint | likely | Calls 00958180 to obtain the current hair tint and writes the exact line Hair tint: %d through 00d7b7a0, then chains into 008de5aa. |
| `008de5aa` | DebugDump_EyeTint | likely | Calls 00958150 to obtain the current eye tint and writes the exact line Eye tint: %d through 00d7b7a0, then chains into 008de607. |
| `008de607` | DebugDump_MaxHitpoints | likely | Writes the exact line Max hitpoints: %d from the dword at character+0x15c through 00d7b7a0 and chains into 008de665. |
| `008de665` | DebugDump_CurrentHitpoints | likely | Writes the exact line Current hitpoints: %d from the dword at character+0x160 through 00d7b7a0 and chains into 008de6c3. |
| `008de6c3` | DebugDump_MaxMana | likely | Writes the exact line Max mana: %d from the dword at character+0x154 through 00d7b7a0 and chains into 008de721. |
| `008de721` | DebugDump_CurrentMana | likely | Writes the exact line Current mana: %d from the dword at character+0x158 through 00d7b7a0 and chains into 008de77f. |
| `008de77f` | DebugDump_MaxLives | likely | Writes the exact line Max lives remaining: %d from the dword at character+0x170 through 00d7b7a0 and chains into 008de7dd. |
| `008de7dd` | DebugDump_CurrentLives | likely | Writes the exact line Current lives remaining: %d from the dword at character+0x16c through 00d7b7a0 and chains into 008de83b. |
| `008de83b` | DebugDump_ExpectedSpeed | likely | Calls 008b3180 to obtain the expected movement speed as a float and writes the exact line Expected speed: %f through 00d7b7a0, then chains into 008de89c. |
| `008de89c` | DebugDump_AppRuntimeSeconds | likely | Builds a two-word time value from the fields at client+0x54/+0x58, converts milliseconds to seconds through 00707040 and writes the exact line AppRuntime Secon… |
| `008de919` | DebugDump_IsStunned | likely | Tests bit 6 of the dword at (character+0x6f8) and, when set, writes the exact line Player IsStunned through 00d7b7a0 before chaining into 008de97d. |
| `008de97d` | DebugDump_IsKnockedOut | likely | Tests bit 7 of the dword at (character+0x6f8) and, when set, writes the exact line Player IsKnockedOut through 00d7b7a0 before chaining into 008de9e2. |
| `008de9e2` | DebugDump_IsKnockedBack | likely | Tests bit 9 of the dword at (character+0x6f8) and, when set, writes the exact line Player IsKnockedBack through 00d7b7a0 before chaining into 008dea46. |
| `008dea46` | DebugDump_IsPoppedUp | likely | Tests bit 0x14 of the dword at (character+0x6f8) and, when set, writes the exact line Player IsPoppedUp through 00d7b7a0 before chaining into 008deaaa (Player … |
| `008dfbf5` | DebugDump_HorizontalViewAngle | likely | Reads the float at wrapper+0x434 and formats it as ViewAngleHorz: %d before continuing. |
| `00915660` | CleanupCall0090f980 | likely | No arguments. Called from four EH unwind funclets. Calls FUN_0090f980 (label accepted:confirmed PerformanceProfilerSampleMap::~PerformanceProfilerSampleMap: re… |
| `00922b70` | Wrapper_0091d980 | likely | Calls 0091d980 on the object in ECX (no arguments of its own) under an SEH frame and returns. |
| `00a0f640` | ConsoleLineRing_Reset | likely | Resets the text ring: stores 0 into the write index at this+0x10 and into the oldest index at this+0x14 and sets the dirty flag at this+0x18 to 1. Nothing else… |
| `00a0f650` | ConsoleLineRing_Dtor | likely | Destructor of the text ring: runs the EH vector destructor iterator over the 500 sixteen-byte entries at this+0x1c (element destructor FUN_006fad90), then rele… |
| `00a0f700` | ConsoleLineRing_Ctor | likely | Constructor: installs the SoeUtil::IString<char> vtable and empty-string buffer at this+0..+0xc, constructs the 500-element, 16-byte-stride vector at this+0x1c… |
| `00a0f7a0` | ConsoleLineRing_NewLine | likely | Advances the ring: increments the write index at this+0x10, wraps it at 500 (0x1f4), and when it catches the oldest index at this+0x14 advances that one too (a… |
| `00a0f7f0` | ConsoleLineRing_Append | likely | Appends the caller's wide string param_2 to the current ring entry (entry = this + 0x1c + writeindex*16) with FUN_0099f6e0 and sets the dirty flag at this+0x18… |
| `00a0f820` | ConsoleLineRing_Render | likely | Flush routine: when the dirty flag at this+0x18 is set it clears it, resolves the GuiEditCtrl through FUN_009f8dc0(this+4) (a dynamic_cast to GuiEditCtrl), the… |
| `00c21130` | EmitSwfFrameInfo | likely | Builds the temporary string list through 008424f0, passes each generated line to the global manager callback, then releases the list. |
| `00cd62f0` | ParseBattleMagesCameraCommand 🌐 | likely | Recognizes the /bm_camera debug command, parses its arguments, and routes accepted values into Battle Mages camera/input helpers. |

### `debug.actor`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0079c980` | ActorListDumpToLog | likely | Fastcall (ecx = holder). Walks the list at holder+0x38 (next link at entry+0x1d4). For each entry writes 'Actor: %s Loc: %.2f, %.2f, %.2f' to ActorDump.log thr… |

### `debug.assert`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f5a60` | WaAssertFail | confirmed | Assertion failure reporter: allocates a 100-byte block with 00d833a6, logs the text 'Reached a wa_assert' through 013ef5c0, then copies param_1 (C string) into… |
| `012f5c10` | SehErrorFail | confirmed | SEH error reporter with fixed text: allocates a 100-byte block with 00d833a6, logs 'Reached an SEH error, please submit a bug.' through 013ef5c0, logs empty st… |
| `012f5dc0` | WaErrorReportAndThrow | confirmed | Writes a wa_error report with a printf-style message (param_1; __vsnprintf into a 1000-byte buffer, the varargs following it), logs the header 'wa_error' and, … |

### `debug.claim`

**ClaimItem** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0105f7f0` | ClaimItem::TraceDump | confirmed | Virtual (vtable slot [0] of ClaimItem, not a destructor: no dtor call, no free). Writes a trace listing through LogV (00720c60) with its first two arguments pa… |

### `debug.console`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00901f10` | BaseClient::PrintConsoleLine | confirmed | Virtual with stack args (param_2 string, param_3 byte, param_4 optional wide string). If this+0x640 is set: when param_3 is zero, calls ConsoleLineRing_NewLine… |

**ClientLuaInternals** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009783c0` | ClientLuaInternals::RunRegisteredKeyChunk | likely | Takes a text argument (param_2, converted with FUN_0083d1e0) and a hashed key (param_4). Returns 1 at once when param_3 is nonzero. For each registered name st… |

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0081f080` | DataSourceConsole_ListMatching | confirmed | Thiscall (ecx=registry) with one char* filter arg. Walks the data-source list whose head is at registry+0xae98 (next at element[+0x10]). For each element, with… |
| `00975f00` | ClientLuaSinkForward_008a7b20 | likely | Lua C function (returns 0). Under the internals lock: when the sink at ClientLuaInternals+0x90 is non-null, calls 008a7b20 with no arguments; that callee tail-… |
| `009781c0` | ClientLuaReportEvalError | likely | Reads the Lua global evalStat: when its type is boolean (1) and its value is false, the error flag is set. It then reads the Lua global evalResult; when its ty… |
| `009788f0` | ClientLuaEvalConsoleText | likely | Takes a wide string param_2. When its first character is '=' it wraps the rest in 'evalStat, evalResult = pcall( function() return <text> end )'; otherwise it … |
| `00fc0d90` | BoolFromFirstChar | likely | Reads the string pointer at param_1+4, sets *param_2 to 0, then sets it to 1 when the first character is '1', 't' or 'T'; otherwise it stays 0. Returns nothing. |
| `00fc1740` | TaggedValueToString | likely | Dispatches on the type tag at *this: tag 0 formats the float at this[7] through 00402fd0 with the format at 017e4014, tag 1 calls 00402fd0 (its arguments are n… |
| `00fc1820` | TaggedValueFromString | likely | Dispatches on the type tag at *this: tag 0 stores _atof of the text at param_2+4 as a float at this[7]; tag 1 calls 0072d5d0 on the text and the output; tag 2 … |
| `00fc2030` | ClearCVarCommand | likely | Hashes the argument name with 00705b00 (case-folded), finds the entry in the singleton table from 00fc1fb0 by bucket and key, and reports not found through 007… |
| `00fc21b0` | RegisterIntVar | likely | Hashes the name with 00705b00, looks it up in the command table singleton (bucket at +0x14, key at node+0x40); when absent it creates an entry through 00fc1990… |
| `00fc2310` | RegisterFloatVar | likely | Same flow as 00fc21b0 for a float: entry tag 0, stores the storage pointer at +0x1c, writes the float default into the storage, assigns the name, stores the ch… |
| `00fc2470` | RegisterVectorVar | likely | Same flow for a four-float value: entry tag 4; copies the four dwords of param_4 into the storage at param_2, stores the changed-flag pointer and flag byte, as… |
| `00fc25f0` | RegisterBoolVar | likely | Same flow for a byte: entry tag 2; writes the byte default param_4 into the storage at param_2, assigns the name, stores the changed-flag pointer and flag byte… |
| `00fc2750` | RegisterStringVar | likely | Same flow for a string: entry tag 3; stores the IString storage pointer param_2 at +0x1c, assigns the name, and when param_5 is zero assigns the default param_… |
| `00fc28d0` | SetVarUpdateCallback | likely | When the callback argument param_2 is non-zero: looks up the name in the command table and, when found, appends the callback to the entry's array at +0x2c (cou… |
| `00fc2a00` | RegisterCommandFunction | likely | Hashes the name with 00705b00 and finds or creates its entry through 00fc1990 with tag 5. For a tag-5 entry it stores param_3 at entry+0x4, param_2 at entry+0x… |
| `00fc2b60` | RegisterAlias | likely | Hashes the name with 00705b00 and finds or creates its entry through 00fc1990 with tag 6. For a tag-6 entry it on first use stores a local IString (its vtable … |
| `00fc2cf0` | CollectCVarNameCompletions | likely | Given the prefix argument (param_1), optionally resets the array argument (0071b470), resets the output IString param_2 to the empty literal, then walks the co… |
| `00fc3000` | UnregisterCVarByName | likely | Hashes the name with 00705b00, searches its bucket for the node with that key, and when found removes it through 00fc1a40 and returns 1; returns 0 when no node… |
| `00fc30f0` | QueueCommandText | likely | When param_1 is non-null, under the guard appends the text param_1 to the pending command buffer (data at +0x53c, length at +0x540, capacity at +0x544, grown t… |
| `00fc3270` | DumpRegisteredCommands | likely | Logs the header 'Registered commands...' to CommandQueue.log, then walks the ordered entry list (head at +0x8, next at +0x38) under the guard and logs the name… |
| `00fc3480` | ExecuteCommandLine | likely | Runs one console command line: splits the first token with 00fc15c0 into the name and the argument text, resolves the entry through 00fc3340 and 00fc3390, and … |
| `00fc3d50` | SetVec4VarIfChanged | likely | Finds the entry for the flag-filtered name (00fc3340). When it exists and its tag equals 4 (vector, compared as the float bit pattern 5.60519e-45), it compares… |
| `00fc4340` | ExecCommandFile | likely | When the path argument is non-empty, opens the file through 00703e10 (mode 3), reads its contents through 00702200, and splits it into lines (006fc270). Each l… |

### `debug.exception`

**SceaSharedUtil::InternalCrashReporterWrapper** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7c470` | InternalCrashReporterWrapper::Construct | confirmed | Fastcall (ECX = this). Stores the InternalCrashReporterWrapper vftable at +0 and the List<LogEntry> vftable at +4 (the embedded list, with head, tail and count… |
| `00d7c530` | InternalCrashReporterWrapper::DtorBody | confirmed | Fastcall (ECX = this). Stores the InternalCrashReporterWrapper vftable at +0, calls 00d7ddc0 on the member block, then for each string member (IStringFixed<cha… |
| `00d7c730` | InternalCrashReporterWrapper::ScalarDeletingDtor | confirmed | Virtual slot [0] of the InternalCrashReporterWrapper vftable. Calls 00d7c530 (the non-deleting body) and calls _free(this) when bit 0 of the flag argument is s… |
| `00d7ca90` | InternalCrashReporterWrapper::Init | likely | Thiscall (ECX = wrapper, param_2 stored at +0x14, param_3 = config pointer whose [0] and [1] dwords are copied as strings with 00402ce0; its other dwords are c… |

**wws::CrashReport::Private::BufferLogger** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7e0a0` | wws::CrashReport::Private::BufferLogger::Construct | confirmed | Fastcall (ECX = object). Stores the BufferLogger vftable at +0, clears the byte at +4 and sets the dword at +0x204 to -1; returns nothing. |
| `00d7dfd0` | BufferLogger_VFormatTo512 | likely | Virtual slot [0] of the BufferLogger vftable (thiscall, a level dword, a format string and varargs). Formats the message with __vsnprintf into this+4 with a 0x… |
| `00d7e0c0` | wws::CrashReport::Private::BufferLogger::VirtualDeleteSelf | likely | Virtual slot [1] of the BufferLogger vftable (thiscall, one flag byte). Stores the Logger vftable at +0 and calls _free(this) when bit 0 of the flag is set; re… |

**wws::CrashReport::Private::Logger** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7cce0` | wws::CrashReport::Private::Logger::DtorBody | confirmed | Fastcall (ECX = logger). Stores the wws::CrashReport::Private::Logger vftable at this+0 and returns; a 7-byte non-deleting destructor body with no other effect. |
| `00d7d810` | wws::CrashReport::Private::Logger::DtorBody_B | confirmed | Fastcall (ECX = logger). Stores the wws::CrashReport::Private::Logger vftable at +0 and returns; byte-identical to 00d7cce0. |
| `00d7ccf0` | wws::CrashReport::Private::Logger::VirtualDeleteSelf | likely | Virtual slot [1] of the Logger vftable (thiscall, one flag byte). Stores the Logger vftable at this+0 and calls _free(this) through 00d82463 when bit 0 of the … |
| `00d7e050` | wws::CrashReport::Private::Logger::VirtualLog | likely | Virtual slot [0] of the Logger vftable (thiscall, level, format and varargs). When DAT_01bf328c is set and its threshold at +0x60 is below the level it returns… |

**wws::CrashReport::Private::NullLogger** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7d910` | wws::CrashReport::Private::NullLogger::VirtualDeleteSelf | likely | Virtual slot [1] of the NullLogger vftable (thiscall, one flag byte). Stores the Logger vftable at +0 and calls _free(this) through 00d82463 when bit 0 of the … |

**(no class)** (36)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7e890` | SessionDirEntry_NameMatches | confirmed | Takes a prefix string and a directory-entry record. Returns 0 unless the record kind at +0x110 is 2 (directory), the name is longer than 0x18 bytes, and the na… |
| `00d7eba0` | UploadMarker_RecordNow | confirmed | Builds the .upload path with 00d7e4a0 into a 260-byte stack buffer and calls 00d81630 with it, which writes the current time into that path's metadata file. |
| `00d7ebf0` | UploadMarker_CheckAllowed | confirmed | Builds the .upload path with 00d7e4a0 and returns the value of 00d815c0 for that path in AL. The security-cookie check runs after the call and leaves EAX uncha… |
| `00d7edb0` | SessionFolder_BuildPath | confirmed | Returns 0 when the 64-bit value at this is zero. Otherwise it builds the root folder through 00d812a0 (mode = dword at this+8) into the path buffer, logs and r… |
| `00d7eea0` | SessionFolder_FindMatch | confirmed | Method (ECX = find-state record, +0xc = root index). Loops over candidate roots: it first tries FindNext on the local find state (00d81ac0). When no entry is l… |
| `00d7f260` | SessionFolder_CreateLocked | confirmed | Method (ECX = session record at block+0x70). Stores the QueryPerformanceCounter value (seeds the 64-bit session id) at this+0 and a zero attempt count at this+… |
| `00d7f460` | CrashReport_WriteMiniDumpFile | confirmed | Loads dbghelp.dll and gets MiniDumpWriteDump with GetProcAddress (failures reported through the logger at param_6). Builds the file name from the directory arg… |
| `00d7f790` | SessionData_SaveToFolder | confirmed | Gets the executable file name (GetModuleFileNameA), takes the base name after the last slash, backslash or colon, and checks the 64-bit session value at block+… |
| `00d7f8a0` | CrashReport_SaveSessionAndDump | confirmed | Calls 00d7f790 to save the session data (on failure it returns). It then takes the executable base name (00d7cc90), writes a minidump with 00d7f460 using the d… |
| `00d7f9f0` | WriteMinidumpAPC_InvokeUserCallback | confirmed | If the function pointer stored at the global block (block+0) is nonzero, it builds the user_data path under the crash folder (00d7eca0 with size 0x104, then ap… |
| `00d7fdb0` | wwsCrashReportPlatformLaunchUploader | confirmed | Builds the uploader command line from the crash parameter block (param_1) and the exe path plus numeric fields in param_2: -h <+0x54>; -r <+0xc> or -s <param_2… |
| `00d80950` | wwsCrashReportPlatformInit | confirmed | Allocates the dump buffer (block+0x14 megabytes shifted by 20, or 32 MB when zero) into block+0x10, creates the minidump thread at LAB_00d7fb30 (stack 0x1000),… |
| `00d80e90` | MetadataPath_Build | confirmed | Takes the base path in EAX and the destination in ESI with its size in EDI. If the base path is non-null and nonempty it copies it with _strncpy and appends th… |
| `00d80ee0` | LoadMetadata | confirmed | Stored-argument form: ECX = output 8-byte buffer, the second stack argument = base path. If both are nonnull it builds the .metadata path (00d80e90), opens it … |
| `00d81010` | SaveMetadata | confirmed | Fastcall: ECX = base path, EDX = pointer to an 8-byte value. Builds the .metadata path (strncpy and strncat with 0x105 bytes), creates it with 00d7cd10 using w… |
| `00d812a0` | CrashRootFolder_Resolve | confirmed | Stack args: mode, output buffer, size. Mode 0 uses GetTempPathA, mode 1 uses SHGetFolderPathA with CSIDL 0x1c (local application data); each appends the folder… |
| `00d7c6a0` | XmlWriteNodeListIsList | likely | Cdecl, five stack arguments (writer config, a name string, a pointer whose +4 is a list head, a second string, and a flag). Calls 01017250 on the second argume… |
| `00d7cc50` | CrashReporter_CreateInstance | likely | Thiscall (ECX = destination pointer, param_2 forwarded). Allocates 0x9264 bytes through 00d833a6; if that fails it stores 0 in DAT_01bf3258, otherwise it const… |
| `00d7d0c0` | wwsCrashReportCheckForUploader | likely | Cdecl (crash report object, output buffer, output size). Logs 'wwsCrashReportCheckForUploader: no crash report object' through 00d7e0e0 when the object is null… |
| `00d7d280` | wwsCrashReportLaunchUploaderIfNeeded | likely | Cdecl-style (crash report object, two dwords, two bytes, a flag byte). Logs 'wwsCrashReportLaunchUploaderIfNeeded: no crash report object' when the object is n… |
| `00d7d4a0` | wwsCrashReportCreate | likely | Cdecl (crash report object, params pointer). Fails with 'wwsCrashReportCreate: invalid parameter' when a pointer is null, the string at params+8 is null or the… |
| `00d7d930` | CrashReport_BufferedInit | likely | Cdecl with one stack argument (a dword). Clears a 0x104-byte path buffer, takes the executable file name through GetModuleFileNameA and 00d7cc90 (or 'wws_crash… |
| `00d7ddc0` | CrashReport_ShutdownGlobal | likely | Cdecl with no arguments. When the global DAT_01bf328c is set it clears a 0x104-byte path buffer, takes the executable file name (GetModuleFileNameA with the 00… |
| `00d7dec0` | CrashReportEmitLine | likely | Cdecl-style (level dword, format string, varargs). If GetConsoleWindow returns null it formats into a 0x7ff-byte buffer with __vsnprintf and sends it to Output… |
| `00d7e0e0` | CrashReportLog | likely | Cdecl (level, message, varargs). Drops the message when DAT_01bf328c is set and its threshold at +0x60 is below the level; otherwise, when the callback at +0x6… |
| `00d7e140` | CrashReport_CreateDirectoryOrLogEsi | likely | The directory path is read from ESI (not in the decompiled signature). Calls CreateDirectoryA(path, NULL): success, or GetLastError() equal to 0xb7 (already ex… |
| `00d7e1c0` | CrashReport_MoveFileOrLog | likely | The source path is read from EDI and the destination from ESI (not in the decompiled signature). Calls MoveFileA(source, destination); on failure it logs 'Unab… |
| `00d7e280` | CrashReport_CreateDirectoryOrLogEcx | likely | Fastcall with the directory path in ECX. Calls CreateDirectoryA(path, NULL) and returns 1 on success. On failure, GetLastError() == 0xb7 (already exists) also … |
| `00d7e2d0` | CrashReport_RemoveDirectoryTree | likely | Cdecl-style (path buffer, buffer size). Appends '/' to the path with strncat, lists the folder entries through 00d81870 and 00d81900, removes each entry, and l… |
| `00d7fcf0` | CrashBlock_MoveOutOfPageAndRelease | likely | Copies the 0x90-byte global crash block (0x24 dwords) from its VirtualAlloc page into the heap copy at DAT_01bf3290 and makes that copy the global block. It th… |
| `00d802c0` | CrashBlock_EndAccess | likely | If the global crash block pointer is set, makes its 0x90 bytes read-only (VirtualProtect with PAGE_READONLY 2) and releases the mutex at block+0x2c. |
| `00d80300` | CrashBlock_BeginAccess | likely | If the global crash block pointer is set, waits on the mutex at block+0x2c (INFINITE) and then makes the 0x90-byte block read-write (VirtualProtect with PAGE_R… |
| `00d80840` | CrashBlock_MinidumpThreadBody | likely | Takes one argument, calls WriteMinidumpAPC on it inside an SEH frame, and then calls SetEvent on the handle at block+0x28. |
| `00d80a70` | CrashReport_SaveDumpWithStackLogger | likely | Stores the wws::CrashReport::Private::Logger vtable into a stack object, takes the current thread id (GetCurrentThreadId), calls 00d7f8a0 with the thread id an… |
| `00d815c0` | UploadThrottle_Allowed | likely | Takes the base path. It loads the saved 64-bit time with 00d80ee0; if that fails it returns 1. Otherwise it reads the current Unix nanosecond time (00d81bd0) a… |
| `00d81630` | Metadata_SaveCurrentTime | likely | Takes the base path on the stack. Gets the current Unix nanosecond time from 00d81bd0 and calls 00d81010 with the path in ECX and the address of the 8-byte tim… |

### `debug.freeform_actor`

**FreeformActorManipulationEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c95b0` | FreeformActorManipulationEventHandler::Dtor | confirmed | Replaces the object vtable with FreeformActorManipulationEventHandler::vftable and returns. |
| `009c95c0` | FreeformActorManipulationEventHandler::ScalarDeletingDtor | confirmed | Restores event-handler vtable, frees when flag bit 0 is set, otherwise returns the object pointer. |

**FreeformActorManipulationModeBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c9750` | FreeformActorManipulationModeBase::GetField0C | confirmed | Returns the dword stored at object offset +0x0c. |
| `009c9770` | FreeformActorManipulationModeBase::Dtor | confirmed | Restores the FreeformActorManipulationModeBase vtable at the object base. |
| `009c9c10` | FreeformActorManipulationModeBase::ScalarDeletingDtor | confirmed | Restores the base-mode vtable, frees when flag bit 0 is set, otherwise returns the object pointer. |

**FreeformActorManipulationModeManipulateActor** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c97f0` | FreeformActorManipulationModeManipulateActor::StoreVectorAn… | confirmed | Copies four dwords into +0x50..+0x5c and calls child vtable slot +0x18 when the child at +0x14 exists. |
| `009c9830` | FreeformActorManipulationModeManipulateActor::ForwardChildS… | confirmed | When child at +0x14 exists, tail-dispatches through its vtable slot +0x10; absent child returns after cleaning 0x14 stack bytes. |
| `009c9850` | FreeformActorManipulationModeManipulateActor::ForwardChildS… | confirmed | When child at +0x14 exists, tail-dispatches through its vtable slot +4; otherwise returns. |
| `009ca450` | FreeformActorManipulationModeManipulateActor::Dtor | confirmed | Restores derived mode and handler vtables, releases child event handler and three IString members, then restores the base-mode vtable. |
| `009ca590` | AdjustorThunk_009ca5a0 | confirmed | Subtracts 0x10 from ECX and jumps to 009ca5a0; this is an adjustor thunk, not the deleting-destructor body. |
| `009ca5a0` | FreeformActorManipulationModeManipulateActor::ScalarDeletin… | confirmed | Calls the full ManipulateActor destructor, frees when flag bit 0 is set, and otherwise returns the object pointer. |
| `009cb0b0` | FreeformActorManipulationModeManipulateActor::Ctor | confirmed | Initializes base and handler vtables, stores context values, initializes three IString members, builds the transform action, sets four vector fields, and selec… |
| `009c9870` | FreeformActorManipulationModeManipulateActor::ForwardAdjust… | likely | Calls a virtual method at this-0x10, passes both input values to 00777930, then calls slot +4 on the handler at this-8. |
| `009c98b0` | FreeformActorManipulationModeManipulateActor::ForwardAdjust… | likely | Passes the pointed value to the mode at this-0x10 and helper 00779ea0, then forwards the original pointer to handler at this-8 slot +8. |

**FreeformActorManipulationModeManipulateActorInternalBase** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c98f0` | FreeformActorManipulationModeManipulateActorInternalBase::H… | confirmed | Event 8 activates the mode and saves coordinates; event 0x40 deactivates it and clears two flags; bit 1 of final flags sets another byte. |
| `009ca000` | FreeformActorManipulationModeManipulateActorInternalBase::S… | confirmed | Calls InternalBase destructor, frees when flag bit 0 is set, and otherwise returns the object pointer. |
| `009ca8b0` | FreeformActorManipulationModeManipulateActorInternalBase::C… | confirmed | Stores context, actor and event values, clears two state bytes, creates the transform action, adjusts its option bit when set, toggles a global flag and initia… |
| `009c9d10` | FreeformActorManipulationModeManipulateActorInternalBase::D… | likely | Restores InternalBase vtable, clears action pointer through a nested virtual call when set, then restores the manipulation base vtable. |

**FreeformActorManipulationModePlaceActor** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c9cc0` | FreeformActorManipulationModePlaceActor::HandlePlacementEve… | confirmed | Reads vector through virtual slot +0x14; kind 1 passes vector offsets +400/+0x2f0 to child callback, kind 0x1000 calls 00777610. |
| `009c9fe0` | FreeformActorManipulationModePlaceActor::ScalarDeletingDtor | confirmed | Calls PlaceActor destructor, frees when flag bit 0 is set, and otherwise returns the object pointer. |
| `009ca7d0` | FreeformActorManipulationModePlaceActor::Ctor | confirmed | Stores context and actor pointers, installs base then PlaceActor vtables, builds the initial vector and transform action, stores its pointer, and toggles a glo… |
| `009c9790` | FreeformActorManipulationModePlaceActor::SetPlacementVector | likely | Builds a four-float vector, sets its first component from nested field +0x390 plus a constant, passes it to own slot +0x14, then calls 007776b0. |
| `009c9c30` | FreeformActorManipulationModePlaceActor::Dtor | likely | Restores PlaceActor vtable, clears its current action through a nested virtual call when set, then restores the base-mode vtable. |

**FreeformActorManipulationModeRotation** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009caad0` | FreeformActorManipulationModeRotation::Ctor | confirmed | Constructs InternalBase state through 009ca8b0, then installs the Rotation mode vtable. |
| `009caba0` | FreeformActorManipulationModeRotation::ScalarDeletingDtor | confirmed | Runs InternalBase destructor, frees when flag bit 0 is set, otherwise returns the object pointer. |

**FreeformActorManipulationModeScale** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c9a90` | FreeformActorManipulationModeScale::PassScaleVectorToChild | confirmed | Loads child at +8 and calls its vtable slot +8 with a pointer to this object's four floats at +0x20. |
| `009cac00` | FreeformActorManipulationModeScale::Ctor | confirmed | Constructs InternalBase state, initializes three floats from a shared constant, installs Scale vtable and clears field +0x2c. |
| `009cacc0` | FreeformActorManipulationModeScale::StoreScaleVector | confirmed | Copies four dwords from the supplied vector into object fields +0x20 through +0x2c. |
| `009cad30` | FreeformActorManipulationModeScale::ScalarDeletingDtor | confirmed | Runs InternalBase destructor, frees when flag bit 0 is set, otherwise returns the object pointer. |

**FreeformActorManipulationModeTranslation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ca9a0` | FreeformActorManipulationModeTranslation::Ctor | confirmed | Constructs InternalBase state through 009ca8b0, then installs the Translation mode vtable. |
| `009caa70` | FreeformActorManipulationModeTranslation::ScalarDeletingDtor | confirmed | Runs InternalBase destructor, frees when flag bit 0 is set, otherwise returns the object pointer. |
| `009c9da0` | FreeformActorManipulationModeTranslation::DrawAxisHandles | likely | When active, reads two object vectors, adds a fixed fourth-component offset, builds handle geometry through 0075a000, and submits it through 007590e0 when the … |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ca350` | CreateFreeformTargetEventHandlers | likely | Initializes context fields, allocates two handler objects, constructs actor and terrain target handlers, and stores pointers at +0x0c and +0x10. |
| `009ca5c0` | BuildTransformHandleArguments | likely | Searches a keyed property range, prepares local string and fixed fields from supplied objects, then calls a nested object's virtual slot +4 with the data. |
| `009caa20` | DestroyInternalModeTemporary | likely | Sets up SEH cleanup state, calls 009c9d10 for cleanup, then restores the previous exception-list pointer. |
| `009cab50` | DestroyInternalModeTemporary | likely | Sets up SEH cleanup state, calls 009c9d10 for cleanup, then restores the previous exception-list pointer. |
| `009cace0` | DestroyInternalModeTemporary | likely | Sets up SEH cleanup state, calls 009c9d10 for cleanup, then restores the previous exception-list pointer. |
| `009cad90` | ReplaceModeWithPlaceActor | likely | Unregisters two descriptor links, releases the current mode object, creates a PlaceActor mode, stores it, and calls its virtual slot +0x18 with owner fields. |
| `009cb230` | ReplaceActorManipulationMode | likely | Unregisters two descriptors, releases current mode, registers the descriptors again, constructs a replacement ManipulateActor mode, stores it, and updates owne… |

### `debug.item`

**GameItemEntry** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01061090` | GameItemEntry::TraceDump | confirmed | Virtual (vtable slot [0] of GameItemEntry; not a destructor: no free, no dtor call). Writes trace lines through LogV (00720c60) with its first two arguments pa… |

### `debug.itemlist`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c72230` | ItemListDumpToString | confirmed | Appends the literal 'ItemList:' with 00402ce0, then for each node of the list headed at +0x14 (next at +0x18) appends ' Id=%d,Qty=%d,Cost=%d' with the node fie… |

### `debug.metrics`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b76420` | ReportSoccerContactMetrics | confirmed | Telemetry pass over the match: for the two player slots (walking the list at +0x304 and splitting by the 0x6e4 flag) it drives an animation query (FUN_0078b390… |
| `007b0e00` | SoundDebugStatsCollect | likely | Thiscall: clears the count at this+0x8508, then for each active voice record of the owner at (this+8)+0xa0 (count at (this+8)+0x9c, active when the dword at +0… |
| `00fc8500` | SystemMetricsSnapshot | likely | Plain function with one pointer argument (an int array of about 26 dwords). Stores a 64-bit tick from 007027f0 at [0..1]; with GetPerformanceInfo fills 64-bit … |

### `debug.physics`

**AppPhysics::GhostObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fddaa0` | GhostObject::DebugDraw | likely | Virtual slot 9, inherited by WaterBody, GhostTrigger and CameraFrustumSensor. Lazily allocates a 0x28-byte record (00fdda00) and a 0x38-byte record (00fdd850) … |

### `debug.profiler`

**BaseProfile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01089dc0` | BaseProfileScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ret 4 with flag byte): calls 01089be0 (BaseProfile destructor body), then calls _free on this when bit 0 of the flag is set. Returns… |
| `01089be0` | BaseProfileDtor | likely | Under an SEH frame: stores the BaseProfile vftable at this, then calls 01089b70 (HashListMap destructor body) with ECX = this+0x100, destroying the embedded ha… |

**SoeGems::PerformanceProfiler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d057e0` | PerformanceProfilerPopSample | confirmed | Virtual slot [2] of SoeGems::PerformanceProfiler: gets the thread context via 006fb860, decrements its counter, and when the enabled flag at context+0x10 is se… |

**SoeUtil::List<SoeGems::PerformanceProfiler::ThreadContext,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d05a20` | ThreadListBufferRealloc | confirmed | Virtual slot [2] of SoeUtil::List<SoeGems::PerformanceProfiler::ThreadContext,-1>: calls 006fb9c0, then vtable slot +4 of the object from 006fb860 (when non-nu… |
| `00d05a60` | ThreadListFreeBlock8 | confirmed | Virtual slot [3] of the ThreadContext list (one stack argument, ret 4): frees the pointer argument with 00401c00(ptr, 8). |

**SoeUtil::ProfilerHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d05090` | ProfilerHandlerBaseDtorBody | confirmed | Sets the SoeUtil::ProfilerHandler vtable at offset 0 and returns (7 bytes: mov [ecx],vtable; ret). Does not free the object. |
| `00d050a0` | ProfilerHandlerScalarDtor | confirmed | Scalar deleting destructor: stores the SoeUtil::ProfilerHandler vtable and calls _free on the object when bit 0 of the flag argument is set. |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d05150` | ProfileAccumulateSample | likely | Increments the 64-bit count at +0x0/+0x4, stores the 64-bit sample (param_2 low, param_3 high) as the last value at +0x20/+0x24, adds it to the 64-bit total at… |
| `00d062b0` | ProfilerResetAllContexts | likely | Stores the current 64-bit tick from 007029b0 at this+0x22258/+0x2225c, locks through 00fc45f0, calls 00d060f0 for each record of the chain at this+0x10 (next l… |
| `00d064f0` | AppendSamplePathSegments | likely | Recursively appends the parent chain (this+0x34, parent first) into the IString at param_3: each level writes '/' (0x2f) at the end of the buffer, then appends… |
| `00d06820` | ClearSampleChainLocked | likely | Locks through 00fc45f0, then repeatedly removes the head of the chain at this+0xa8 through 008d55b0 until it is empty, and unlocks through 00fc4610 unless this… |

### `debug.trace`

**EvaluationEnvironment** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01384af0` | EvaluationEnvironment::TraceCardRef | confirmed | Thiscall with a label (second argument) and a card pointer (third argument). Under the trace guard (this+4 zero or +0x22c set, and this+0x78 set): if the card … |
| `01385610` | EvaluationEnvironment::TraceFormatDepth | confirmed | Thiscall with a format string and varargs. When the depth counter at this+0x74 is above 1000 it emits 'INFINITE RECURSION DETECTED! > 1000 Depth! abort! abort!… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f5d80` | LogWaErrorLine | confirmed | Varargs wrapper (cdecl-style, first stack argument = format in EDI): calls 012f5920 (AppendLogLine) with ECX = the 'wa_error.log' string at 0x18d65a8 and the a… |
| `00d7b7a0` | SinkAppendFormatIfSet | likely | Cdecl-style (plain ret, two stack arguments): reads the dword at *param_1; if it is non-null it pushes the variadic argument pointer and the format string and … |
| `012f5920` | AppendLogLine | likely | Appends one formatted line to a log file: the file path is the C string in ECX (copied into a std::string, 01241650), the format is in EDI (an unaffected regis… |
| `012f5da0` | AppendLogLineToPath | likely | Varargs wrapper: ECX = the first stack argument (a C string path), EDI = the second stack argument (format), and calls 012f5920 (AppendLogLine) with the addres… |

### `debug.udp`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010db970` | DumpPacketRingToFile | confirmed | Under the lock at this+0x2b4 (010da6d0 enter, 010da6e0 leave) it opens the file named by param_2 with fopen mode wt. For each non-empty entry of a ring of poin… |

### `debug.userdata_command`

**UserDataCommand_RespondDBAvatars** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01523f50` | UserDataCommand_RespondDBAvatars::DebugDump | confirmed | Virtual slot [19]. Appends the header UserData Command Respond DB Avatars and the parallel lists labelled mAvatarIDs, mTitles, mIcons and mDescriptions (int an… |

**UserDataCommand_RespondDBAwards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015234e0` | UserDataCommand_RespondDBAwards::DebugDump | confirmed | Virtual slot [19]. Appends the header UserData Command Respond DB Awards and the parallel lists labelled mAwardIDs, mTitles, mIcons and mDescriptions (int and … |

**UserDataCommand_RespondDBHonors** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01522a70` | UserDataCommand_RespondDBHonors::DebugDump | confirmed | Virtual slot [19]. Appends the header UserData Command Respond DB Honors and the parallel lists labelled mHonorIDs, mTitles, mIcons and mRequirements (string a… |

**UserDataCommand_RespondNewsMessage** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01524990` | UserDataCommand_RespondNewsMessage::DebugDump | confirmed | Virtual slot [19]. Appends the header UserData Command Respond News Message and the parallel lists labelled mHeadlines, mCallbacks, mParams and mValues (string… |

**UserDataCommand_RespondPlayerAvatars** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01522260` | UserDataCommand_RespondPlayerAvatars::DebugDump | confirmed | Virtual slot [19]. Appends the header UserData Command Respond Player Avatars and the four parallel lists with labels mAvatarIDs, mEnabled, mSelected and mDate… |

**UserDataCommand_RespondPlayerAwards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01521b50` | UserDataCommand_RespondPlayerAwards::DebugDump | confirmed | Virtual slot [19]. Appends the header UserData Command Respond DB Awards, then mAwardIDs: Size(n) with the count of the list at this+0x10/+0x14, then one -----… |

**UserDataCommand_RespondPlayerInfo** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015206f0` | UserDataCommand_RespondPlayerInfo::DebugDump | confirmed | Virtual slot [19]. Calls FUN_0140c290 on the output argument (a stub), then appends the header UserData Command Respond Player Info and sprintf lines mStatus (… |

### `debug.watchdog`

**SoeGems::InfiniteLoopMonitor** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d04710` | InfiniteLoopMonitorPoll | confirmed | Virtual slot [2] of SoeGems::InfiniteLoopMonitor: loops until 00fc4ac0 returns non-zero; each pass, when the threshold at this+0xc4 is positive, takes a 64-bit… |
| `00d04830` | InfiniteLoopMonitorCtor | confirmed | Stores the InfiniteLoopMonitor vtable at offset 0, param_2 at +0xc4 (threshold) and param_3 at +0xec, installs the SoeUtil::IString<char> vtable at +0xd0 with … |
| `00d048d0` | InfiniteLoopMonitorDtorBody | confirmed | Non-deleting destructor body: resets the InfiniteLoopMonitor vtable and the IString vtable, releases the string buffer at +0xd4 when its count at +0xdc is posi… |
| `00d04920` | InfiniteLoopMonitorScalarDtor | confirmed | Scalar deleting destructor: resets the InfiniteLoopMonitor and IString vtables, releases the string buffer as in 00d048d0, calls 00fc4c30, and calls _free on t… |
| `00d049c0` | InfiniteLoopMonitorCtorNamed | confirmed | Calls 00fc4cc0 with the literal "InfiniteLoopMonitor" (argument form 0,2,name), installs the InfiniteLoopMonitor and IString vtables, sets the threshold at +0x… |
| `00d04a90` | InfiniteLoopMonitorCtorAssign | confirmed | Calls 00fc4cc0(0,2,0), installs the InfiniteLoopMonitor and IString vtables, stores param_2 at +0xc4, assigns param_3 to the string at +0xd4 through 00402ce0, … |

### `engine.client`

**ActionBarSlot** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008bf9f0` | ActionBarSlot::ScalarDeletingDestructor | likely | Calls 0x0101cd70 with this in ECX and no stack arguments, then frees this when deleting-flag bit 0 is set; otherwise returns this. |

**BaseClient** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a67e0` | BaseClient::AllocAndStore64C | confirmed | Creates a 4-byte object with the allocator FUN_00d833a6 (returning 0 on failure), fills it by calling FUN_00979280(this), stores the result at this+0x64c and i… |
| `008a7820` | BaseClient::CallVtbl6C | confirmed | Calls vtable slot +0x6c on this with param_3 and param_4, then returns with ret 0xc; param_2 is unused. This is a normal virtual call. |
| `008a79c0` | BaseClient::CallVtbl10 | confirmed | Virtual forwarder: tail-calls slot +0x10 of the object vtable with no arguments; no direct callers. |
| `008a79d0` | BaseClient::CallVtblC | confirmed | Calls vtable slot +0xc on this with (param_2,param_3,0), then returns with ret 8; this is a normal call, not a tail jump. |
| `008a7a10` | BaseClient::Wrapper_00a9e3c0 | confirmed | Adjustor thunk loads ECX from this+0xf4 and jumps to 00a9e3c0; no stack arguments are changed. |
| `008a7a20` | BaseClient::Wrapper_00aa19e0 | confirmed | Adjustor thunk loads ECX from this+0xf4 and jumps to 00aa19e0; no stack arguments are changed. |
| `008a7a30` | BaseClient::CallVtbl8OnF4 | confirmed | Loads ECX from this+0xf4, reads the target from that object vtable slot +8 and jumps to it; no stack arguments are changed. |
| `008a7f70` | BaseClient::Call00704dd0WithArgs | confirmed | Makes a normal call to 00704dd0 with (0,&DAT_017ed2ec,param_1), then returns with ret 4; no direct callers are listed. |
| `00917050` | BaseClient::ResetDevice | confirmed | Reads window state through FUN_00915b80; when the first flag byte is zero and the bytes at this+0x870 or +0x871 are set, runs FUN_00823c50 (flag zero) or FUN_0… |
| `00917910` | BaseClient::ResetDisplay | confirmed | Virtual slot [1] shared by PlayClient and BaseClient. Calls BaseClient::ResetDevice (00917050). On failure it runs a fallback ladder, each step followed by ano… |
| `0091de50` | BaseClient::InitLogging | confirmed | Startup logging setup: sets the log directory ./Logs through FUN_0070bd80, reads Logging/Level (and, when this+0x6d8 is set, Logging/Directory with the ./Logs … |
| `009359e0` | BaseClient::Shutdown | confirmed | Sets the global shutdown flag, logs 'Starting Client Shutdown.', tears down client managers and optional GuiOnSave state, destroys owned objects, clears stored… |
| `00942c00` | BaseClient::ScalarDeletingDtor | confirmed | Bytes subtract 0x0c from ECX and jump to 00947180, adjusting a BaseClient subobject pointer before shared scalar deletion. |
| `00942c10` | BaseClient::ScalarDeletingDtor | confirmed | Bytes subtract 0x10 from ECX and jump to 00947180, adjusting a BaseClient subobject pointer before shared scalar deletion. |
| `00942c20` | BaseClient::ScalarDeletingDtor | confirmed | Bytes subtract 0x14 from ECX and jump to 00947180, adjusting a BaseClient subobject pointer before shared scalar deletion. |
| `00942c30` | BaseClient::ScalarDeletingDtor | confirmed | Bytes subtract 0x18 from ECX and jump to 00947180, adjusting a BaseClient subobject pointer before shared scalar deletion. |
| `00942c40` | BaseClient::ScalarDeletingDtor | confirmed | Bytes subtract 4 from ECX and jump to 00947180, adjusting a BaseClient subobject pointer before shared scalar deletion. |
| `00942c50` | BaseClient::ScalarDeletingDtor | confirmed | Bytes subtract 8 from ECX and jump to 00947180, adjusting a BaseClient subobject pointer before shared scalar deletion. |
| `009464c0` | BaseClient::~BaseClient | confirmed | Destructor body: writes BaseClient::vftable into this+0..+0x1c, then the handler interface vtables (StringProvider, PrintOutput, CrashReporterHandler, GuiReset… |
| `00946a80` | BaseClient::InitAppServices | confirmed | Virtual slot 62 of BaseClient (shared with PlayClient). Allocates (00d833a6) and constructs the client's service managers in order: sound loader 008a2530 and 0… |
| `007938c0` | BaseClient::InitStringMember4f0 | likely | Fastcall (ECX = this). Calls 007001c0 twice with an EH frame, calls 00703540 (speculative FixedString32Constructor), then stores the SoeUtil::IString<char> vta… |
| `008d2760` | BaseClient::Slot1_StringDispatch | likely | Virtual slot [1] shared by BaseClient and PlayClient: builds a temporary SoeUtil::IString<char>, calls the object's own virtual slot +8 with the incoming argum… |
| `008d2dd0` | BaseClient::Slot4_WideStringToSlotC | likely | Virtual slot [4] shared by BaseClient and PlayClient: converts the incoming UTF-16 (short*) argument into a temporary SoeUtil::IString<char> through FUN_0083d0… |
| `008d2ea0` | BaseClient::Slot6_WideStringToSlot14 | likely | Virtual slot [6] shared by BaseClient and PlayClient: converts the incoming UTF-16 argument into a temporary SoeUtil::IString<char> through FUN_0083d070, calls… |
| `008d2f80` | BaseClient::Slot32_WideStringToSlot84 | likely | Virtual slot [32] shared by BaseClient and PlayClient: builds a temporary string from the incoming argument through FUN_0083d1e0, calls the object's own virtua… |
| `008d3020` | BaseClient::Slot65_GuidPairHandler | likely | Virtual slot [65] shared by BaseClient and PlayClient: resolves the local actor through FUN_00984960, and when the param_6 flag is set and one of the two incom… |
| `008ec090` | BaseClient::ConfigureClientEndpoints | likely | Stores two endpoint values, assigns strings with empty defaults, creates service object, configures URL and updates shared endpoint record. |
| `008ec1b0` | BaseClient::FormatVersionString | likely | Passes the caller-provided output-string pointer, format "FreeRealms v%s", and embedded version string 1.910.1.530630 to 00402fd0; returns the formatter charac… |
| `008ed8d0` | BaseClient::CreateClientCommandProcessor | likely | Virtual slot [59] of PlayClient and BaseClient. Allocates 0x228 bytes with 00d833a6; a null result returns 0. Otherwise it assigns param_2 into a local IString… |
| `00941ba0` | BaseClient::Ctor | likely | Installs BaseClient and handler-interface vtables; initializes client flags, strings, collections, timers, display/player state, gateway handler, GarageManager… |
| `00947180` | BaseClient::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: calls the BaseClient destructor body 009464c0 on this, then calls _free(this) when bit 0 of the flag argument is set; returns … |
| `0094a410` | BaseClient::Run 🌐 | likely | Initializes client timing and services, waits for world readiness when not in the alternate startup path, logs startup and system information, then runs the vi… |
| `0094a970` | BaseClient::FrameTick | likely | Virtual (vtable slot 15 of BaseClient, shared with PlayClient; about 7 KB, fastcall with the object pointer): per-frame client tick. Re-applies window focus wh… |

**BaseItemStatDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c0480` | BaseItemStatDefinition::ScalarDeletingDestructor | confirmed | Installs BaseItemStatDefinition::vftable at object offset 0 and frees the object when deleting-flag bit 0 is set. |

**CameraFrustumSensor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af3a50` | CameraFrustumSensor::RemoveEventKey | likely | Virtual slot 15. Derives the same key as slot 14 (event type 0 uses (object at +0x1c)+0x268 after a check that its +0x568 is nonzero; type 0xe uses the object … |

**ClientApplication** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012d4270` | ClientApplication::~ClientApplication | confirmed | Stores ClientApplication::vftable into *this, calls 012d1ea0 (list walk at +0x14), deletes the NetworkService at +8 through 012f0560 after 012f1d90 and clears … |
| `012d4710` | ClientApplication::ScalarDeletingDtor | confirmed | Calls the ClientApplication destructor body 012d4270 on this, then frees this with _free when bit 0 of the flag argument is set; returns this (ret 4). |
| `012d0950` | ClientApplication::SetCurrentLobby | likely | Asserts that its argument is non-null (ClientApplication.cpp line 0x267, 'lobby' assert) and stores it at this+0xac; returns nothing. |
| `012d0980` | ClientApplication::ClearCurrentLobby | likely | Asserts its argument is non-null (ClientApplication.cpp line 0x26d, 'lobby' assert), then clears this+0xac to 0 when it equals the argument. |
| `012d2260` | ClientApplication::RemoveId_0xA0 | likely | Asserts its argument is non-null (ClientApplication.cpp line 0x24f, 'lobby' assert), then removes the first int equal to the argument from the int vector at th… |
| `012d3550` | ClientApplication::PushEvent | likely | Thiscall (ECX = object) with one stack argument (the pointer to push): reads the list proxy at this+0x14, allocates a 0xc-byte list node holding the argument v… |

**ClientAreaDataManager** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090bd50` | ClientAreaDataManager::ClientAreaDataManager | confirmed | Calls 008d1970, installs the manager vtables, clears fields +0x2c..+0x38 and a 0x200-byte region, stores param_2 at +0x23c, then calls 00906880. |
| `0090be80` | ClientAreaDataManager::ScalarDeletingDtorAdjustor | confirmed | Subtracts four from this and jumps to 0090be30, adapting the secondary-vtable this pointer to the deleting destructor. |
| `0090be90` | ClientAreaDataManager::HasEntries | confirmed | Returns whether the signed dword at this+0x38 is greater than zero. |
| `0090bea0` | ClientAreaDataManager::GetField30 | confirmed | Returns the dword stored at this+0x30. |
| `0090beb0` | ClientAreaDataManager::GetField164 | confirmed | Returns the dword stored at this+0x164. |
| `0090bec0` | ClientAreaDataManager::GetCount | confirmed | Returns the dword stored at this+0x38. |
| `0090c190` | ClientAreaDataManager::AddStringEntry | confirmed | Hashes and inserts the supplied string key, updates the maximum numeric index, stores the index in the new entry, then calls virtual slot 6 with the index and … |

**ClientCommandProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9f410` | ClientCommandProcessor::ExecuteCommand | confirmed | Virtual command dispatcher taking an integer kind: kind 0 calls FUN_00a9e790, kind 1 calls FUN_00a9e8f0, kind 2 calls FUN_00a9ea40; each handled case returns 1… |
| `00aa06a0` | ClientCommandProcessor::ClientCommandProcessor | confirmed | Constructor: installs CommandProcessor::CommandEventHandler::vftable then ClientCommandProcessor::vftable, stores the second argument at +4 (the console contex… |
| `00aa07b0` | ClientCommandProcessor::~ClientCommandProcessor | confirmed | Destructor body: rewrites the ClientCommandProcessor vtable, calls FUN_00986c10, saves the command history with FUN_00a9c420 (writes CommandHistory.txt), rewri… |
| `00aa12c0` | ClientCommandProcessor::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00aa07b0 (which saves the command history) and, when bit 0 of the flags argument is set, calls _… |

**ClientDisplay** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b16e10` | ClientDisplay::ClientDisplay | confirmed | ClientDisplay constructor: installs ClientDisplay::vftable into *this, stores the owner argument at +8, zeroes the loading-screen cache pointer at +0xc, and ca… |
| `00b17060` | ClientDisplay::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor of ClientDisplay::vftable: calls the destructor body 00b16fe0 and frees the object with _free when bit 0 of the flag argume… |
| `00b16ea0` | ClientDisplay::ShowLoadingScreen | likely | Creates or refreshes the loading screen cached at this+0xc: when a cached object exists and its +0x14/+0x10 fields differ from the two size arguments it is des… |
| `00b16f90` | ClientDisplay::DropLoadingScreen | likely | Drops the loading screen: when the owner object at this+8 has a non-null field at +0x24c it calls 009f7900 with that object and the float constant at 0x01762a3… |

**ClientInterpolatedMovementController** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b16b80` | ClientInterpolatedMovementController::ClientInterpolatedMov… | confirmed | Constructor: calls the base constructor 01077a70 (which runs the MovementController chain and installs InterpolatedMovementController::vftable) and then instal… |
| `00b16c90` | ClientInterpolatedMovementController::ScalarDeletingDtor | confirmed | Slot [0] scalar deleting destructor: installs ClientInterpolatedMovementController::vftable, calls the base destructor 01077940 and then frees the object with … |
| `00b16c40` | ClientInterpolatedMovementController::Destruct | likely | Destructor body: installs ClientInterpolatedMovementController::vftable, then calls the base destructor 01077940, which walks the chain by writing Interpolated… |

**ClientMovementManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b16cf0` | ClientMovementManager::CreateController | confirmed | Slot [0] virtual factory of ClientMovementManager::vftable: when the kind argument equals 1 it allocates 0x50 bytes through the allocator 00d833a6 and, if the … |

**ClientPcData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a10a80` | ClientPcData::SetField_0x34f80 | confirmed | Stores param_2 into the dword field at this+0x34f80 and returns. |
| `00a10a90` | ClientPcData::SetField_0x34f7c | confirmed | Stores param_2 into the dword field at this+0x34f7c and returns. |

**ClientServerCore::BaseApi** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00897390` | ClientServerCore::BaseApi::CallSlot50With1 | confirmed | Adjustor thunk (bytes 8b 54 24 0c 8b 81 3c ff ff ff 8b 40 50 6a 01 81 c1 3c ff ff ff 52 8b 54 24 10 52 ff d0 c2 0c 00, ret 0xc): subtracts 0xc4 from this and c… |

**ControllerCloneWarsDialog** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aebb40` | ControllerCloneWarsDialog::DestroyBody | confirmed | Destructor body: stores ControllerCloneWarsDialog::vftable and calls the base teardown 0094fab0. It releases no members of its own. |
| `00aebd60` | ControllerCloneWarsDialog::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 7): stores the CloneWars vtable, calls the base teardown 0094fab0, and frees the object through _free when bit 0 of the… |

**ControllerDressingRoom** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00985790` | ControllerDressingRoom_DestructBody | confirmed | Destructor body of ControllerDressingRoom under SEH frame LAB_01599ec8: stores the ControllerDressingRoom vftable into [param_1], parks the object on the stack… |
| `009858d0` | ControllerDressingRoom::ControllerDressingRoom | confirmed | Constructor under SEH frame LAB_01599ef8: calls the base constructor 0094f6c0(param_2,param_3,param_4), installs the ControllerDressingRoom vftable, calls 0094… |
| `00985990` | ControllerDressingRoom_ScalarDeletingDtor | confirmed | Scalar deleting destructor registered in slot [7] of ControllerDressingRoom::vftable: under an SEH frame it stores the class vftable into [param_1], calls the … |
| `00985ce0` | ControllerDressingRoom_CollectEventFlags | confirmed | Virtual method in slot [50] of ControllerDressingRoom::vftable: walks the list at [this+0xc8] through the node link at +4, and for every event node whose [node… |
| `00985ed0` | ControllerDressingRoom_UpdateMove | confirmed | Per-frame continuation of the move started by 00985a10 (virtual slot [51] of the dressing-room vtable): when the accumulated time at +0x138 is below the durati… |

**ControllerFlyOver** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aec9a0` | ControllerFlyOver::Ctor | confirmed | Constructs the FlyOver controller: calls the base constructor 00950530 first, stores the vtable, initializes two Curve-pointer arrays with their vtables at +0x… |
| `00aecbf0` | ControllerFlyOver::DestroyBody | confirmed | Destructor body: stores ControllerFlyOver::vftable, for each element of the two Curve-pointer arrays calls 007d69e0 and frees the element, releases each array … |
| `00aed040` | ControllerFlyOver::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 7): calls 00aecbf0 (destructor body) and frees the object through _free when bit 0 of the flag argument is set. |

**ControllerFreeFly** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae85f0` | ControllerFreeFly::DestroyControllerBaseState | confirmed | Installs the ControllerFreeFly vtable and invokes the Controller base destructor to release its inherited key and controller state. |
| `00ae8650` | ControllerFreeFly::GetFieldE4 | confirmed | Returns the float stored at this+0xe4. |
| `00ae8660` | ControllerFreeFly::SetFieldE4 | confirmed | Stores the supplied float at this+0xe4 and returns. |
| `00ae8680` | ControllerFreeFly::GetFieldE0 | confirmed | Returns the float stored at this+0xe0. |
| `00ae8690` | ControllerFreeFly::Ctor | confirmed | Constructs the Controller base, installs the ControllerFreeFly vtable, clears state flags, initializes basis vectors from constants, and registers several inpu… |
| `00ae87b0` | ControllerFreeFly::ScalarDeletingDtor | confirmed | Calls the Controller base destructor, frees the object when flag bit 0 is set, and returns its pointer otherwise. |
| `00ae8810` | ControllerFreeFly::GetHorizontalBasisAngle | confirmed | Reads two horizontal basis components from the associated controller transform, returns atan2 of them when their squared magnitude exceeds the threshold, and o… |
| `00ae8c60` | ControllerFreeFly::UpdateTransform | confirmed | Reads the current transform, applies input-derived rotations and movement, normalizes and adjusts the resulting basis, invokes mode-dependent controller action… |
| `00ae9460` | ControllerFreeFly::MoveFromInput | confirmed | Reads registered movement inputs, derives forward/side/up axes and speed, applies transform basis vectors, clamps the target position to per-axis bounds, optio… |

**ControllerGarage** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af6560` | ControllerGarage::DtorBody | confirmed | Stores the ControllerGarage vtable, then calls ControllerRoot::DtorBody (009509a0). |
| `00af65d0` | ControllerGarage::Ctor | confirmed | Calls ControllerRoot::ControllerRoot (00950530) with the forwarded arguments, stores the ControllerGarage vtable, zeroes the flag bytes at +0xf4..+0xf7 region … |
| `00af66f0` | ControllerGarage::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: stores the vtable, calls ControllerRoot::DtorBody (009509a0), then calls _free when bit 0 of the argument is … |

**ControllerInspect** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af6150` | ControllerInspect::DtorBody | confirmed | Stores the ControllerInspect vtable at +0, then calls ControllerRoot::DtorBody (009509a0) on the object under an SEH frame. |
| `00af61e0` | ControllerInspect::Ctor | confirmed | Calls ControllerRoot::ControllerRoot (00950530) with the forwarded arguments, stores four float constants from globals at +0x15c, +0x164, +0x174 and +0x170, st… |
| `00af62d0` | ControllerInspect::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: stores the vtable, calls ControllerRoot::DtorBody (009509a0), then calls _free on the object when bit 0 of th… |

**ControllerMarketplace** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae9840` | ControllerMarketplace::SetFields174And178 | confirmed | Stores the two supplied values at this+0x174 and this+0x178. |
| `00ae9860` | ControllerMarketplace::SetAdjustedFields174And178 | confirmed | Subtracts viewport integer fields +0x1d0 and +0x1d4, scaled by a constant, from the two supplied floats and stores the results at this+0x174 and this+0x178. |
| `00ae98c0` | ControllerMarketplace::SetField17C | confirmed | Stores the supplied value at this+0x17c and returns. |
| `00aeb760` | ControllerMarketplace::Ctor | confirmed | Constructs the marketplace controller: stores ControllerMarketplace::vftable, sets the key pairs at +0xe0..0xec to the sentinel pair, constructs the HiddenActo… |
| `00aeba00` | ControllerMarketplace::DestroyBody | confirmed | Destructor body: stores ControllerMarketplace::vftable, looks up the object for the key pair at +0xe0/+0xe4 (00984960) and sets its +0x226 flag to 1 when its v… |
| `00aebb20` | ControllerMarketplace::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 7): calls 00aeba00 (destructor body) and frees the object through _free when bit 0 of the flag argument is set. |
| `00aea8d0` | ControllerMarketplace::Update | likely | Virtual slot 51, per frame. Recomputes the camera direction and position from the fields at +0x100..0x118, +0x158..0x170 and the target (00769710, 0094cf60), c… |

**ControllerNull** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af63e0` | ControllerNull::Ctor | confirmed | Calls Controller::Controller (0094f6c0) with the forwarded arguments, then stores the ControllerNull vtable. |
| `00af6470` | ControllerNull::DtorBody | confirmed | Stores the ControllerNull vtable, then calls Controller::DtorBody (0094fab0). |
| `00af6500` | ControllerNull::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: stores the vtable, calls Controller::DtorBody (0094fab0), then calls _free when bit 0 of the argument is set. |

**ControllerOverheadCamera** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aec080` | ControllerOverheadCamera::DestroyBody | confirmed | Destructor body: stores ControllerOverheadCamera::vftable, calls 0094c8a0(1), sets the byte at this+0xf4 to 1, updates the owner obtained from 0094cf60 (007052… |
| `00aec2e0` | ControllerOverheadCamera::Ctor | confirmed | Constructs the overhead camera controller: calls the base constructor 00af06f0 first, stores the vtable, clears the bytes at +0x200 and +0xf4, sets the dwords … |
| `00aec430` | ControllerOverheadCamera::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 7): calls 00aec080 (destructor body) and frees the object through _free when bit 0 of the flag argument is set. |

**ControllerPanZoom** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af05b0` | ControllerPanZoom::DtorBody | confirmed | Destructor body (ECX = this, EH state set). Stores ControllerPanZoom::vftable at [this] and calls ControllerRoot::DtorBody (009509a0). It is called from Contro… |
| `00af06f0` | ControllerPanZoom::Ctor | confirmed | Constructor (ECX = this, three stack args forwarded). Calls ControllerRoot::ControllerRoot (00950530), stores ControllerPanZoom::vftable, zeroes the state at +… |
| `00af08c0` | ControllerPanZoom::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): stores ControllerPanZoom::vftable at [this], calls ControllerRoot::DtorBody (009509a0), calls _free(thi… |

**ControllerPhotographer** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af3170` | ControllerPhotographer::DtorBody | confirmed | Stores the ControllerPhotographer vtable, then releases four embedded object pointers at +0x370..+0x37c (vtable slot +0x28 with 0, then a deleting call with 1 … |
| `00af3ca0` | ControllerPhotographer::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: calls the destructor body 00af3170, then calls _free on the object when bit 0 of the argument is set; returns… |
| `00af5af0` | ControllerPhotographer::Ctor | confirmed | Constructor body: calls Controller::Controller (0094f6c0) with the forwarded arguments, stores the ControllerPhotographer vtable and initialises the photo fiel… |
| `00af2620` | ControllerPhotographer::Update | likely | Virtual slot 51. Takes a float time step, multiplies it by a constant and clamps it to 1. Eases the 4-float pose at +0x110..0x11c toward the pose at +0x120..0x… |
| `00af3380` | ControllerPhotographer::GetFloatE8 | likely | Virtual slot 16. Returns the float stored at this+0xe8. |
| `00af3390` | ControllerPhotographer::SetFloatE8 | likely | Virtual slot 17. Stores the float argument at this+0xe8 and returns (ret 4). |
| `00af33b0` | ControllerPhotographer::GetFloatE4 | likely | Virtual slot 27. Returns the float stored at this+0xe4. |
| `00af33f0` | ControllerPhotographer::RecomputeFrame | likely | Clamps the float at this+0xec to a floor constant. Reads the renderer width and height dwords at (the object at +0x74) +0xc78 and +0xc7c and, when both are non… |
| `00af4590` | ControllerPhotographer::ProcessActions | likely | Virtual slot 50. Returns early unless the byte at +0x88 is set. Walks the action list at +0xc8: axis actions 1,2,3,4,7,8 and 0xe set local axis values; button … |

**ControllerRail** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aed860` | ControllerRail::Ctor | confirmed | Constructs the rail camera controller: calls the base constructor 0094f6c0 first, stores ControllerRail::vftable, sets default vectors at +0x160..0x17c and +0x… |
| `00af0200` | ControllerRail::DtorBody | confirmed | Destructor body (ECX = this, EH state set). Stores ControllerRail::vftable at [this]. Frees the buffers at this+0x220 and this+0x224 (007d69e0, then _free when… |
| `00af0590` | ControllerRail::~ControllerRail | confirmed | Scalar deleting destructor shape (ECX = this, flag argument): calls ControllerRail::DtorBody (00af0200), then calls _free(this) when bit 0 of the flag argument… |
| `00aef410` | ControllerRail::ProcessActions | likely | Thiscall with one stack argument (ECX = this). Walks the action list at this+0xc8 (node: entry pointer +0, next +4; entry: type dword +0x18, flag byte +0xc or … |
| `00aefc90` | ControllerRail::RebuildFromCamera | likely | Thiscall (ECX = this). Takes the four-float orientation from the object returned by 0094cf60 (vtable +0x1c), and searches the list at [this+0x74] for the neare… |
| `00af0290` | ControllerRail::Activate | likely | Thiscall (ECX = this, kept in edi). Clears the list at this+0xc8 (0094e210). Sets default constants: this+0x240 from the object at this+0x74+0xe8+0x434, tangen… |

**ControllerWebBrowser** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af6750` | ControllerWebBrowser::Ctor | confirmed | Calls Controller::Controller (0094f6c0) with the forwarded arguments, then stores the ControllerWebBrowser vtable. |
| `00af67e0` | ControllerWebBrowser::DtorBody | confirmed | Stores the ControllerWebBrowser vtable, then calls Controller::DtorBody (0094fab0). |
| `00af6870` | ControllerWebBrowser::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: stores the vtable, calls Controller::DtorBody (0094fab0), then calls _free when bit 0 of the argument is set. |

**Deep::EngineCriticalErrorHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ab00` | Deep::EngineCriticalErrorHandlerScalarDtor | confirmed | Scalar deleting destructor of Deep::EngineCriticalErrorHandler: stores the class vtable at +0 and calls _free when bit 0 of param_2 is set. |
| `0089aaf0` | Deep::EngineCriticalErrorHandler::~EngineCriticalErrorHandl… | likely | Trivial destructor body: writes the Deep::EngineCriticalErrorHandler vtable pointer at +0 and returns; reached only from EH unwind funclets. |

**Engine::AccountServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013be260` | Engine::AccountServiceBase::~AccountServiceBase | confirmed | Destructor body: stores Engine::AccountServiceBase::vftable at +0, frees the block at +0x20 when non-null and zeroes +0x20, +0x24 and +0x28, then runs the base… |
| `013be480` | Engine::AccountServiceBase::ScalarDeletingDtor | confirmed | Virtual slot [0] of Engine::AccountServiceBase: calls the destructor body 013be260 on this, then calls _free(this) only when bit 0 of the argument is set; retu… |
| `013c04b0` | Engine::AccountServiceBase::AccountServiceBase | confirmed | Constructor body for Engine::AccountServiceBase: runs FUN_01230d10 (SEH frame setup) and stores the Engine::AccountServiceBase vtable at +0, calls FUN_0143d050… |

**Engine::FRTWAccountService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b9640` | Engine::FRTWAccountService::~FRTWAccountService | confirmed | Stores Engine::FRTWAccountService::vftable at +0, then calls the member tidy helpers on this+0x50, +0x44, +0x38 and +0x2c (013b95d0, 013b9560, 013b94f0, 013b94… |
| `013b9890` | Engine::FRTWAccountService::FRTWAccountService | confirmed | Constructor body: calls 013c04b0 (AccountServiceBase constructor) on this, stores Engine::FRTWAccountService::vftable at +0, then constructs the member objects… |
| `013b9920` | Engine::FRTWAccountService::ScalarDeletingDtor | confirmed | Virtual slot [0] of the class: calls the destructor body 013b9640 on this, then calls _free(this) only when bit 0 of the argument is set; returns this. |

**Engine::PageFilter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c1a70` | Engine::PageFilter::~PageFilter | confirmed | Destructor body (fastcall): re-stores the Engine::PageFilter vtable at +0 and returns. Nothing is freed here. |
| `013c1bb0` | Engine::PageFilter::ScalarDeletingDtor | confirmed | Virtual slot [0] of Engine::PageFilter (bytes f6 44 24 04 01 / 56 / 8b f1 / c7 06 ... / 74 09 / 56 / e8 ... / c3): stores the Engine::PageFilter vtable at +0, … |
| `013c1a50` | Engine::PageFilter::PageFilter | likely | Constructor (fastcall): stores the Engine::PageFilter vtable at +0, zeroes the words at +4 and +8, sets the word at +0xc to 1 and returns this (bytes 8b c1 / 3… |
| `013c2140` | Engine::PageFilter::CopyPage | likely | Virtual slot [1] of Engine::PageFilter: validates the source vector at param_3 (end not below begin, FUN_00d83c2d on failure), builds an iterator with FUN_012c… |

**GameEventSequencer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c4680` | GameEventSequencer::Destructor | confirmed | Restores GameEventSequencer vtable, walks the event chain at +0x24 and calls 0x007e7ea0(event,1,1) for each. Then invokes the owned member at +4 through virtua… |
| `008c4710` | GameEventSequencer::ScalarDeletingDestructor | confirmed | Calls GameEventSequencer cleanup, then frees this when deleting-flag bit 0 is set; otherwise returns this. |

**GameProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9efd0` | GameProcessor::GameProcessor | confirmed | Constructs the GameProcessor base object: stores param_2 at +4, stores GameProcessor::vftable at *this, clears the byte at +8, sets the dwords at +0xc and +0x1… |
| `00b9f340` | GameProcessor::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls 00b9f050(this) (the GameProcessor destructor) and, when the low byte of param_2 is set, calls _free(this); returns this. |

**GameSceneEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b8690` | GameSceneEvent::GameSceneEvent | confirmed | Installs GuiEventHandler then GameSceneEvent vtables, stores param_2, and registers the object with event id 0xe20548a9. |
| `008b8710` | GameSceneEvent::DtorBody | confirmed | Restores GameSceneEvent vtable, unregisters event id 0xe20548a9 for this object, then restores GuiEventHandler vtable. |
| `008b87c0` | GameSceneEvent::ScalarDeletingDtor | confirmed | Calls GameSceneEvent cleanup and frees the object when the deleting flag bit is set. |
| `008b8780` | GameSceneEvent::Handle | likely | When the global scene state at +0xddc0 is zero, calls FUN_009c04b0 with an event-owned field and FUN_009c0a40 with param_3, then returns 1. |

**LoadingScreenThread** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca8530` | LoadingScreenThread::Ctor | confirmed | Calls the thread base constructor 00fc4cc0 with the name string LoadingScreenThread and argument 2, installs LoadingScreenThread::vftable, stores the two argum… |
| `00ca8660` | LoadingScreenThread::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of LoadingScreenThread): calls base destructor 00fc4c30 on this, then frees this through _free when bit 0 of the flag… |

**MovementController** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b16c00` | MovementController::SetField3A | confirmed | One-instruction virtual setter shared by the MovementController, InterpolatedMovementController and ClientInterpolatedMovementController vtables at slot [6]: r… |
| `00b16c10` | MovementController::GetField39 | confirmed | One-instruction virtual getter shared by the three movement controller vtables at slot [7]: raw bytes 8a 41 39 c3 return the byte field at this+0x39. |
| `00b16c20` | MovementController::SetFlag39 | confirmed | One-instruction virtual setter shared by the three movement controller vtables at slot [10]: raw bytes c6 41 39 01 c3 store the constant 1 into the byte field … |

**PlayClient** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00401040` | PlayClient::PlayClient | confirmed | Constructs the outer client object: calls the BaseClient constructor cluster (FUN_00941ba0) under an SEH frame, then stores the PlayClient vtable pointer into … |
| `00401190` | PlayClient::~PlayClient | confirmed | Destructor body: rewrites the eight PlayClient interface vtable pointers at +0x00..+0x1C inside an SEH frame, then calls FUN_009464c0 (the client subsystem tea… |
| `008ba700` | PlayClient::LogOnConnectReceived | confirmed | Writes a NetInfo.log entry with the OnConnect receive label and the two dwords at object offsets +0x3a0 and +0x3a4. |
| `008ba890` | PlayClient::ForwardResolvedValueToSlot80 | confirmed | Calls FUN_00a9e030 and, if it returns a non-null object, invokes this object's virtual method at offset +0x80 with the dword from the returned object at +4. |
| `008ba8c0` | PlayClient::ForwardSecondResolvedValueToSlot80 | confirmed | Calls FUN_00a9e0a0 and, if it returns a non-null object, invokes this object's virtual method at offset +0x80 with the dword from the returned object at +4. |
| `008dd860` | PlayClient::UnbindUiModules | confirmed | Virtual method (slot 23 of PlayClient/BaseClient): when the UI module registry DAT_01be1090 exists it calls 008b1f30 for the exact names App.UI.SelectedControl… |
| `008dda10` | PlayClient::WaitForPlayerData | confirmed | Polls the connection object at this+0x280 through virtual slots +0x28/+0x24, sleeping 25ms per iteration through 006ffcb0 until disconnect, player-data arrival… |
| `008ec1d0` | PlayClient::BuildScriptsBasePath | confirmed | Creates temporary IString, assigns base path, appends ScriptsBase.bin, passes result to helper and releases it. |
| `0090c970` | PlayClient::Update | confirmed | Advances client timers and frame statistics, updates gameplay subsystems and UI state, enforces single-threaded rendering, runs rendering and screenshot work, … |
| `0092b170` | PlayClient::Initialize | confirmed | When param_2 is nonzero, initializes PlayClient: configures crash reporting, reads client settings, initializes asset delivery and display/UI services, constru… |
| `00401490` | PlayClient::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the PlayClient destructor body FUN_00401190, and when the flag argument's low bit is set frees the object with _free; returns… |
| `008c5f60` | PlayClient_CountdownThenDispatch | likely | If signed counter this+0xc8c is <=0, clears byte this+0xc88, calls FUN_008c5ee0 then FUN_008c5e00, and returns the byte from FUN_008c5ee0. If the counter is po… |

**PlayerEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b8830` | PlayerEventHandler::PlayerEventHandler | confirmed | Calls TargetProcessorPlayerEventHandler construction, installs PlayerEventHandler::vftable, and stores param_2 at +0x0c. |

**RealTimeGameEventSequencer** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c4730` | RealTimeGameEventSequencer::Constructor | confirmed | Calls base constructor 0x008b3040 with this in ECX and no stack arguments, installs RealTimeGameEventSequencer vtable, allocates 0x548 bytes, calls 0x00872850 … |
| `008c47c0` | RealTimeGameEventSequencer::InvokeMemberSlot1 | confirmed | Loads the child pointer at this+4 and invokes its virtual slot 1 with the supplied argument and zero. |
| `008c47e0` | RealTimeGameEventSequencer::Destructor | confirmed | Restores RealTimeGameEventSequencer vtable, then calls base cleanup 0x008c4680 with this in ECX and no stack arguments. |
| `008c4830` | RealTimeGameEventSequencer::ScalarDeletingDestructor | confirmed | Installs RealTimeGameEventSequencer vtable, calls base cleanup 0x008c4680 with this in ECX and no stack arguments, then frees this when deleting-flag bit 0 is … |

**SocialShareManager::SocialShareManagerHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c520` | SocialShareManager::SocialShareManagerHandler::~SocialShare… | confirmed | Trivial destructor body: writes the handler vtable pointer at +0 and returns; reached only from EH unwind funclets. |

**SoeGems::PerformanceProfiler** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d08d30` | SoeGems::PerformanceProfiler::RegisterCall | confirmed | Finds or creates thread context and callstack sample entry, initializes counters, and links entry into profiler sample maps. |
| `00d08f10` | SoeGems::PerformanceProfiler::RecordTiming | confirmed | Finds or creates keyed sample, increments 64-bit call count and accumulated timings, and updates maximum and minimum timing pairs. |
| `00d0a350` | SoeGems::PerformanceProfiler::PerformanceProfiler | confirmed | Installs profiler vtable, initializes context list and embedded StringFixed hash list, resets flags, allocates two 4000-byte buffers, and initializes helper. |
| `00d0a410` | SoeGems::PerformanceProfiler::~PerformanceProfiler | confirmed | Clears global profiler pointer when it refers to this object, destroys profiler lists and buffers, then restores base vtables. |
| `00d0a4c0` | SoeGems::PerformanceProfiler::ScalarDeletingDtor | confirmed | Calls profiler destructor, frees object when deleting flag is set, and otherwise returns original pointer. |

**SoeUtil::HashListMap<int,BaseClient::PhotoInfo,4,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009105b0` | BaseClientPhotoInfoMap::Cleanup | confirmed | Restores the PhotoInfo hash-list-map vtable and repeatedly removes entries through 008e6e80 until the map is empty. |

**SoeUtil::HashListMap<int,SoeGems::PerformanceProfiler::SampleEntry,2048,512>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090f980` | PerformanceProfilerSampleMap::~PerformanceProfilerSampleMap | confirmed | Repeatedly removes map entries, frees both tree subtrees and the root allocation, resets size fields, then restores the unbounded HashListMap vtable and clears… |
| `00910590` | PerformanceProfilerSampleMap::ScalarDeletingDtor | confirmed | Calls 0090f980 to destroy the profiler sample map, frees the object when flag bit 0 is set, and returns its pointer. |

**TargetProcessorActorEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c49a0` | TargetProcessorActorEventHandler::Constructor | confirmed | Installs TargetProcessorEventHandler vtable, zeros fields +4 and +8, then installs TargetProcessorActorEventHandler vtable. |
| `008c4a10` | TargetProcessorActorEventHandler::ScalarDeletingDestructor | confirmed | Restores TargetProcessorEventHandler vtable, then frees this when deleting-flag bit 0 is set; otherwise returns this. |

**TargetProcessorEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b8890` | TargetProcessorEventHandler::TargetProcessorEventHandler | confirmed | Writes TargetProcessorEventHandler::vftable to the object and returns it. |
| `008b8940` | TargetProcessorEventHandler::DtorBody | likely | Writes TargetProcessorEventHandler::vftable to the object and returns. |

**TileEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b88e0` | TileEventHandler::TileEventHandler | confirmed | Calls TargetProcessorTileEventHandler construction, installs TileEventHandler::vftable, and stores param_2 at +0x0c. |

**UIProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a8d500` | UIProcessorUpdateHashedEntry | confirmed | Looks up param_3 in a 128-bucket table, retains and assigns the supplied object on a match, notifies the processor for noncurrent keys, and inserts keys above … |

**(no class)** (133)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00403340` | WinMain | confirmed | Program entry invoked directly by the CRT startup: runs the client init pipeline (FUN_006ffc00, FUN_007003b0, config objects FUN_006febe0/FUN_006fef80/FUN_006f… |
| `008a9010` | ReloadOptionsManager | confirmed | Releases the old manager at this+0x668 through its destructor and _free. Allocates 0x38 bytes with FUN_00d833a6 (the sole explicit argument is the size); on su… |
| `008d2730` | LeaveMiniGameOrEncounter 🌐 | confirmed | Calls the encounter-exit request routine FUN_009bd240 and, when it reports success (nonzero return), clears the client dword at +0x704 and calls FUN_008c5b20; … |
| `008dd790` | ProcessDownloadStatsFile | confirmed | If .downloadstats.txt exists, calls 00779090 to construct a StringFixed<1024>, opens/reads the source through 00702440 into that buffer, then calls 00704dd0 wi… |
| `008ed620` | SetClientConfigFileName | confirmed | Takes one stack argument. When it is null, substitutes the literal "ClientConfig.ini"; stores the argument into the IString at this+0x66c by adding 0x66c to EC… |
| `008ed640` | SetUserOptionsFileName | confirmed | Takes one stack argument. When it is null, substitutes the literal "UserOptions.ini"; stores the argument into the IString at this+0x6a4 by adding 0x6a4 to ECX… |
| `00900d40` | ShowLoadingScreenAndNotify | confirmed | Fastcall on this. If this+0x654 is set, calls FUN_009f6890 and FUN_009f5a70, frees the buffer and clears this+0x654. If byte this+0x258 (+600) is set and this+… |
| `009178e0` | SetCameraAutoAdjustment | confirmed | Thiscall with one stack argument (byte). Stores the low byte at this+0x6e4, then calls FUN_00910930("Controls","CameraAutoAdjustment",arg,-1,-1) to write the c… |
| `00a40990` | AnnouncementsSubsystem_Ctor | confirmed | Constructor of the client's announcements subsystem object: stores the owner at +0, constructs two Array<WhatsNewInfo,8,1> members through 00a3ff00, allocates … |
| `00a9e100` | Console_LoadCommandHistory | confirmed | Loads the saved console history file: opens CommandHistory.txt for reading through FUN_00702440 with the SoeUtil::IStringFixed<char,256> wrapper at local_144, … |
| `00a9e2d0` | Console_ReportUnsupportedCommand | confirmed | When the console-context object at this+4 is non-null it formats the wide message Command not supported: %s with the command name argument (FUN_0082ecd0) and h… |
| `00a9ea40` | Console_ReportCommand | confirmed | Console report command: with no argument it writes the usage string 'Incorrect syntax. Usage: /report username' to the console object at [this+4] through the o… |
| `00a9f460` | SoeUtil_Deque256_IString_Dtor | confirmed | Destructor body of the 256-entry deque of IString<unsigned short>: rewrites its own vtable at +0, frees the page array through FUN_00a9cab0(param_1[10]), calls… |
| `00aa01e0` | SoeUtil_Deque256_IString_ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor for the 256-entry deque of IString<unsigned short>: calls the deque destructor body FUN_00a9f460 and, when bit 0 of the flags a… |
| `00caacc0` | LoadingScreen_DtorBody | confirmed | Destructor body of the loading-screen object: releases the GFx movie (00ca8180) and the cached movie at +0xc, truncates and frees the entry and text arrays (00… |
| `00cab4e0` | LoadingScreen_Ctor | confirmed | Constructs the loading-screen object: stores param_3 at +0, param_2 at +4, param_6 at +0x10 and param_5 at +0x14, installs the IString member (vtable at +0x18)… |
| `0076a1e0` | GlobalObjectDtorBody | likely | fastcall with ECX = the global object (0x1f0 bytes, created by LazyCreateGlobalObject). Releases each owned field: vtable slot [0] with 1 for the +0x1e0, +0x16… |
| `0076a5d0` | GlobalObjectScalarDeletingDtor | likely | thiscall: calls 0076a1e0 (GlobalObjectDtorBody) on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |
| `00823930` | SetDependentVideoMode | likely | fastcall (ECX = this). If the global DAT_01be11f8 is non-NULL, calls its table slot +0x90 with (this+0x3b8, this+0x3b0, 0, 1). When that returns zero it logs F… |
| `0089f190` | TickTwoMembers | likely | Branches on two embedded objects: calls virtual slot 1 (offset +4) of the object whose pointer sits at this+4 with param_2, then, if the pointer at this+8 is n… |
| `008a8d40` | CallVtable8OfField3e0 | likely | When this+0x3e0 holds an object pointer it calls that object's vtable slot +8 and returns the result; otherwise returns 0. The call is emitted as an indirect j… |
| `008a9160` | HasEntry_3e0_Field214b8 | likely | Reads the object at this+0x3e0; when it is non-null and not the sentinel -0x1d3d0 it calls FUN_01032950 on the dword at object+0x214b8 and returns 1 when that … |
| `008b98b0` | UpdateClientStateLayout | likely | Stores two input values at +0xc78/+0xc7c, sets +0xc80 to 0x140, computes +0xc84 from renderer dimensions, and calls FUN_00af33f0 when state id 0x17 is active. |
| `008b99d0` | GetState17Byte104 | likely | If an active implementation exists and its virtual method returns 0x17, returns the byte at implementation+0x104 in EAX's low byte; otherwise returns zero. |
| `008b9d80` | ForwardState21TargetRequest | likely | Ensures state id 0x15 is active, resolves an object by the two input ids, builds a Target payload from either its +0x1c value or -1, and invokes virtual +0xd4. |
| `008bab70` | CopyConditionalIdPair | likely | Uses DAT_01be4850/4854 when flag +0x83d is clear; otherwise asks FUN_00a2e7d0 for an id pair, then copies the selected pair to param_2. |
| `008bf930` | DestroyMemberAt18ThenDelete | likely | Calls 0x0103d010 with ECX=this+0x18 and no stack arguments, then frees this when deleting-flag bit 0 is set; otherwise returns this. |
| `008c5d90` | StateObj_CallSlot9IfFlag | likely | Calls FUN_008b8ff0(param_2) (stack arg). When it returns nonzero, calls vslot 9 (+0x24) of the object at this+0xec with no arguments. Returns with ret 4. |
| `008c5dc0` | StateObj_State17GateCallSlot9 | likely | If the object at this+0xec is set and its vslot 0 returns 0x17, returns. Otherwise when FUN_008b8ff0(0x17) returns nonzero, calls vslot 9 (+0x24) of the object… |
| `008c5e00` | StateObj_DispatchState17 | likely | If the object at this+0xec returns state 0x17 from vslot 0, reads flag bytes at this+0x11c+0x190 and this+0x130+8. When both are zero it tests bytes at this+0x… |
| `008c6090` | StateObj_GatedVSlot9_0xfThenForward | likely | Calls FUN_008b8ff0(0xf) with a stack arg; when it returns nonzero calls vslot 9 (+0x24) of the object at this+0xec. Then calls FUN_00a70790 with receiver this+… |
| `008c60c0` | StateObj_GatedVSlot9_State0xf | likely | If the object at this+0xec returns 0xf from vslot 0, sets param_2 to 1 when it is 0, calls FUN_008b8ff0(param_2) and calls vslot 9 (+0x24) of the object when n… |
| `008c6110` | StateObj_GatedVSlot9_ForwardReturn1 | likely | If the object at this+0xec returns 0xf from vslot 0, jumps past the gate. Otherwise calls FUN_008b8ff0(0xf) and vslot 9 when true, then calls FUN_00a70790(para… |
| `008c6170` | StateObj_GatedVSlot9_Fwd3Return1 | likely | If the object at this+0xec returns 0xf from vslot 0 it skips the gate. Otherwise calls FUN_008b8ff0(0xf) and vslot 9 (+0x24) of the object when nonzero, then c… |
| `008c6290` | StateObj_GatedVSlot9_Fwd5 | likely | If the object at this+0xec returns 0xf from vslot 0 it skips the gate. Otherwise calls FUN_008b8ff0(0xf) and vslot 9 (+0x24) when nonzero, then calls FUN_00a70… |
| `008c6350` | StateObj_GatedVSlot9_Fwd6 | likely | If the object at this+0xec returns 0xf from vslot 0 it skips the gate. Otherwise calls FUN_008b8ff0(0xf) and vslot 9 (+0x24) when nonzero, then calls FUN_00a70… |
| `008d0d90` | Wrapper_008d0d90 | likely | Thunk decoded from the raw bytes (8b 54 24 08 / 83 ec 08 / 8b c4 / 89 10 / 8b 54 24 14 / 89 50 04 / 8b 44 24 0c / 50 / 81 c1 48 57 03 00 / e8 de 62 ff ff / c2 … |
| `008d2500` | ApplyCombatSettingsChange | likely | Compares a copied encounter-data record against the live one and applies the differences: copies param_1[0x1de] through the accepted EncounterDataCommon copy c… |
| `008d2870` | CallSlot24IfStateMatch | likely | Reads the current state id from virtual slot 0 of the object at +0xec (or 0 when absent) and compares it with param_2; on a match it inspects the flags of the … |
| `008d29c0` | CallSlotD4Slot5CThenSlot24 | likely | Queries the state predicate FUN_008b8ff0(0x27), then calls virtual slots of the object at +0xec with the caller's arguments (slot +0xd4(param_2,param_3,param_4… |
| `008d2d00` | StringLookupThenForward | likely | Builds a temporary SoeUtil::IString<char> and passes it to virtual slot +0x10 of the sub-object at +4 together with param_2 (an out-string query), then calls v… |
| `008ebf70` | ClientAccountManagerStartupFromConfig | likely | Creates and stores the account manager, then returns 0 if its readiness byte at manager +0x22046 is clear. Otherwise it calls 0070f490 with Resources/. Only a … |
| `008f11e0` | CachedStringComparison | likely | Returns the cached dword at this+0xc58 when this+0xc64 is set; otherwise copies the string object at this+0x5ac, compares it, and returns whether the compariso… |
| `008f12a0` | StringComparisonIsOne | likely | Calls 008f11e0 and returns a boolean indicating whether its result equals one. |
| `008f12b0` | GetStringStateFlag | likely | Calls 008f11e0; if the result is not one or this+0xc64 is set, returns the byte at this+0xc61, otherwise returns one. |
| `00917be0` | DrainClientEventQueue | likely | Reads the object at this+0x20 and its queue count at +0x67024. While the count is positive and FUN_008b8d10() returns nonzero: takes the head node at +0x6701c,… |
| `0092e550` | RefreshClientTimingState | likely | Reads a two-part clock value, copies it into offsets +0x66fe0/+0x66fe4 of the object at param_1+0x20, invokes two methods through the interface at param_1+0xec… |
| `009417c0` | InitializeEmbeddedTargetAndControllers | likely | Calls 0089b800 with ECX=this+8 to clear a 0x98-byte structure and set its +0x60 dword to -1; constructs embedded update packets, installs Target at +0x268, ini… |
| `009419d0` | DestroyClientStateMembers | likely | Calls zero-argument singleton getter 00fbe740 and compares its returned pointer with *this; on equality, resets the singleton through 00fbe7c0(0) and destroys … |
| `00942c60` | ClientStateScalarDeletingDtor | likely | Calls 009419d0 to destroy the state block, then frees it only when the deleting-destructor flag is set. |
| `0094a910` | CheckThirtySecondInterval 🌐 | likely | Reads the current 64-bit clock and the stored timestamp at +0x38/+0x3c; when elapsed time exceeds 30,000 ms it calls 0094a110, refreshes the timestamp, and ret… |
| `00958220` | CallHelpersWithPushed79 | likely | Pushes 0x79, calls 00769710 without stack arguments, moves each result into ECX for calls to 007671f0 and 007aa6f0, and leaves 0x79 as the final helper stack a… |
| `0095df40` | ResolveField420ThroughService734 | likely | Returns zero unless this+0x420 and this+0x734 are nonzero; otherwise calls virtual slot +0x10 on the object at this+0x734, passing the value at this+0x420, and… |
| `00960000` | Call0095df70WithZeroOnResolvedMatch | likely | Follows this+0x738 to an object at +200 and requires that object?s +0x420 and +0x734 fields to be nonzero. Resolves through the +0x734 object?s virtual slot +0… |
| `00985800` | DressingRoom_ResetFlagBytes | likely | 39-byte body that stores 0 into the six consecutive bytes at object+0x13c, +0x13d, +0x13e, +0x13f, +0x140 and +0x141 and returns nothing. |
| `00987000` | CommandProcessor_RegisterHandler | likely | Hashes the supplied command name and removes any existing matching entry, inserts a new handler entry with the supplied two values, copies the command name int… |
| `00987180` | CommandProcessor_DispatchNamedString | likely | Builds a temporary IString from param_1 and invokes 00986c90 with that string and param_2, then releases the temporary reference-counted buffer. |
| `00987480` | CommandStringTables_Cleanup | likely | Calls the 128-bucket IString map cleanup and the 1024-bucket command-handler map cleanup, then restores the exception-list pointer. |
| `00987730` | CommandProcessor_InitializeRegistries | likely | Constructs the 1024-bucket handler map and 128-bucket IString map using 009869d0 and 00987690, then returns param_1. |
| `009c36a0` | InvokeConditionalObjectCallback | likely | Checks an object predicate, then may call virtual slot 3 with argument 1 depending on a second predicate and the supplied flag. |
| `009de0f0` | Reader_ReadTwoU32 | likely | Bound-checked read of two consecutive dwords from a byte-reader struct at param_2 (cursor at +8, end at +0xc, error flag at +0x10): each read returns 0 and set… |
| `009f8470` | InitializeLoadingScreenLists | likely | Initializes list subobjects at +0, +0x10, and +0x20; creates two helper objects through allocator size paths 0x3c and 0x38; stores loading-screen state and wri… |
| `00a10730` | MethodThunk_00bd5980 | likely | Eleven-byte MSVC adjustor thunk: 'add ecx,0x354dc' then 'jmp 00bd5980'; Ghidra recovers it as a bare call. The sub-object at this+0x354dc receives the call. |
| `00a10740` | MethodThunk_00bd5e90 | likely | Eleven-byte adjustor thunk ('add ecx,0x354dc; jmp 00bd5e90'), recovered as the single call FUN_00bd5e90(). |
| `00a10760` | MethodThunk_00bda400 | likely | Eleven-byte adjustor thunk ('add ecx,0x354dc; jmp 00bda400'), recovered as the single call FUN_00bda400(). |
| `00a10770` | MethodThunk_00bd8a00_minus1 | likely | Fourteen-byte thunk: pushes -1, adjusts ecx by +0x354dc and calls FUN_00bd8a00, then returns; recovered as FUN_00bd8a00(0xffffffff). |
| `00a10780` | MethodThunk_00bd5970 | likely | Eleven-byte adjustor thunk ('add ecx,0x354dc; jmp 00bd5970'), recovered as the single call FUN_00bd5970(). |
| `00a10790` | MethodThunk_0098fa40 | likely | Eleven-byte adjustor thunk ('add ecx,0x35748; jmp 0098fa40'), recovered as the single call FUN_0098fa40(). |
| `00a107a0` | MethodThunk_0098faa0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35748; jmp 0098faa0'), recovered as the single call FUN_0098faa0(). |
| `00a107b0` | MethodThunk_0098fab0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35748; jmp 0098fab0'), recovered as the single call FUN_0098fab0(). |
| `00a107f0` | MethodThunk_00c7cd40 | likely | Eleven-byte adjustor thunk adds 0x1add0 to ECX and jumps to 00c7cd40; no target body is present here. |
| `00a10850` | MethodThunk_00c7a610 | likely | Eleven-byte adjustor thunk ('add ecx,0x1add0; jmp 00c7a610'), recovered as the single call FUN_00c7a610(). |
| `00a10860` | GetClientField_0x1bec4 | likely | Returns the dword field at this+0x1bec4; nothing else is touched. |
| `00a108b0` | MethodThunk_00c814d0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35620; jmp 00c814d0'), recovered as the single call FUN_00c814d0(). |
| `00a108c0` | SnapshotThunk_00a88f70 | likely | Calls the virtual method at vtable slot +0x10 of the object at this+0x54 into an 8-byte stack value, then forwards that value to FUN_00a88f70; used as a snapsh… |
| `00a10910` | MethodThunk_00c7e1e0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35620; jmp 00c7e1e0'), recovered as the single call FUN_00c7e1e0(). |
| `00a10920` | MethodThunk_00c81250 | likely | Eleven-byte adjustor thunk ('add ecx,0x35620; jmp 00c81250'), recovered as the single call FUN_00c81250(). |
| `00a10930` | MethodThunk_00c7e250 | likely | Eleven-byte adjustor thunk ('add ecx,0x35620; jmp 00c7e250'), recovered as the single call FUN_00c7e250(). |
| `00a10950` | MethodThunk_00b5ce60 | likely | Eleven-byte adjustor thunk ('add ecx,0x19a60; jmp 00b5ce60'), recovered as the single call FUN_00b5ce60(). |
| `00a10960` | MethodThunk_00c869e0 | likely | Eleven-byte adjustor thunk ('add ecx,0x2c340; jmp 00c869e0'), recovered as the single call FUN_00c869e0(). |
| `00a10970` | GetClientField_0x19ae4 | likely | Returns the dword at this+0x19ae4, a field of the +0x19a60 sub-object region; called from the PlayerData Lua table builder 009fad80. |
| `00a10980` | MethodThunk_00b06100 | likely | Eleven-byte adjustor thunk ('add ecx,0x2c2c8; jmp 00b06100'), recovered as the single call FUN_00b06100(). |
| `00a109a0` | MethodThunk_00c896f0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35448; jmp 00c896f0'), recovered as the single call FUN_00c896f0(). |
| `00a109b0` | MethodThunk_00c884b0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35448; jmp 00c884b0'), recovered as the single call FUN_00c884b0(). |
| `00a109c0` | MethodThunk_00c897b0 | likely | Eleven-byte adjustor thunk ('add ecx,0x35448; jmp 00c897b0'), recovered as the single call FUN_00c897b0(). |
| `00a10a10` | MethodThunk_00c791d0 | likely | Eleven-byte thunk ('mov ecx,[ecx+0x2c2c0]; jmp 00c791d0'): it loads the member pointer at +0x2c2c0 and tail-calls FUN_00c791d0; recovered as a bare forwarding … |
| `00a10a20` | MethodThunk_00b1ee10 | likely | Eleven-byte adjustor thunk ('add ecx,0x1d0b0; jmp 00b1ee10'), recovered as the single call FUN_00b1ee10(). |
| `00a10aa0` | ClientFlagBitTest_0x34f7c | likely | Returns bit param_2 of the 32-bit flag word at this+0x34f7c: for param_2 < 0x20 it returns (((byte*)(this+0x34f7c))[param_2>>3] >> (param_2 & 7)) & 1; for para… |
| `00a10ad0` | MethodThunk_00baa450 | likely | Eleven-byte adjustor thunk ('add ecx,0x1bee8; jmp 00baa450'), recovered as the single call FUN_00baa450(). |
| `00a10ae0` | MethodThunk_00baa4b0 | likely | Eleven-byte adjustor thunk ('add ecx,0x1bee8; jmp 00baa4b0'), recovered as the single call FUN_00baa4b0(). |
| `00a12400` | ResetClientSubsystems_35428 | likely | Calls FUN_00be13d0 and FUN_00b056a0, then walks the list at this+0x35428 following +0x3a4 links and invokes vtable slot +8 on each node, passing *(*(this+0x3bc… |
| `00a12470` | SetClientFlag_0x464 | likely | Stores param_2 into the field at this+0x464, calls FUN_00c8a110(param_2 != 0) with the boolean form of the value and then FUN_00bfcf30(). |
| `00a124b0` | GetClientField_0x6d0 | likely | Returns the dword at this+0x6d0; called from the PlayerData Lua table builder. |
| `00a124c0` | GetClientField_FromArg_0xc | likely | Free function taking an object pointer: returns the dword at (+0xc) of the object passed on the stack. |
| `00a12510` | SetClientFlag_34f54 | likely | Stores param_2 into the field at this+0x34f54 and calls FUN_00bfcdd0(). |
| `00a12530` | ApplyClientFlagOnce_34f68 | likely | Calls FUN_00bfce80(param_2) and then, only when the byte at this+0x34f68 is still 0, calls FUN_00ade730(param_2, *(this+0x34f54)) and sets that byte to 1. |
| `00a12580` | LookupKeyInSingleton_Mod10 | likely | Takes the singleton field at FUN_00769fe0() and walks the bucket at singleton+0x10+(key % 10)*4 following the first-dword links, returning 1 when a node's seco… |
| `00a12740` | MemberThunk_LoadThis35634_00c8d170 | likely | Loads ECX from [ECX+0x35634], then jumps to 00c8d170; this is a member-pointer forwarding thunk, not an add-ECX adjustor. |
| `00a12760` | MethodThunk_00c8cd70 | likely | Eleven-byte adjustor thunk forwarding to FUN_00c8cd70; recovered as a bare call. |
| `00a12770` | MethodThunk_00c8cde0 | likely | Eleven-byte adjustor thunk forwarding to FUN_00c8cde0; recovered as a bare call. |
| `00a12780` | MethodThunk_00c8bdf0 | likely | Eleven-byte adjustor thunk forwarding to FUN_00c8bdf0; recovered as a bare call. |
| `00a12800` | GetClientFieldPair_0x19ae8 | likely | Copies the two dwords at this+0x19ae8 and this+0x19aec into param_2[0] and param_2[1]. |
| `00a12830` | MethodThunk_00c8d700 | likely | Eleven-byte adjustor thunk forwarding to FUN_00c8d700; recovered as a bare call. |
| `00a12a80` | MethodThunk_00c8e440_Guarded | likely | Guarded thunk: when the member at this+0x35648 is non-null it tail-calls FUN_00c8e440; the recovered body is that conditional forwarding call. |
| `00a12b80` | MethodThunk_00bfcd30 | likely | Eleven-byte adjustor thunk forwarding to FUN_00bfcd30; recovered as a bare call. |
| `00a3fe30` | ReleaseTwoRefsAndDestroyTwoArrays | likely | For each holder at +0x570 then +0x56c, releases it only when the pointer is non-null and DAT_01be1090 is nonzero: calls the held object's vtable byte offset +0… |
| `00a41250` | AnnouncementRequest_Dispatch 🌐 | likely | Request dispatcher for the announcements subsystem: when the byte at param_1[0x15d] equals 1 and a virtual slot 0x28 call on the owning client object returns n… |
| `00a42010` | NudgesSubsystem_Ctor | likely | Constructor of the client's nudge subsystem object: stores the owner at +0, allocates 600 bytes and constructs a NudgeDataSource with 00a41f60 (owner plus the … |
| `00a9e030` | ConsoleHistoryPrev | likely | Decrements the cursor at +0x7c of the history object, wrapping it to count(+0x3c)-1 when it drops below zero, then returns the entry for that cursor: index ari… |
| `00a9e0a0` | ConsoleHistoryNext | likely | Increments the cursor at +0x7c and wraps it to 0 when it reaches count(+0x3c), then returns the entry for the new cursor from the page table at +0x1c (same (cu… |
| `00a9e3c0` | ConsoleHistory_AppendText | likely | Builds an IString<unsigned short> from the caller's text (FUN_00703620 into the local IString at local_1c, which starts from the empty static at DAT_01be7acc) … |
| `00a9e480` | Console_ExecuteCommandLine | likely | Console line executor: resets the history cursor at +0x7c, appends the line to the history ring via FUN_00a9c990 (trimming the ring when the count exceeds 100 … |
| `00a9e790` | Console_AttackCommand | likely | Console command body for a target name: with a null name it calls FUN_00a29b50, otherwise it resolves the name with FUN_00983dc0; on success it calls FUN_00a2c… |
| `00a9e8f0` | Console_HitCommand | likely | Same shape as the index-0 console command: returns early on a null name, resolves the name with FUN_00983dc0, calls FUN_00a2c880 (the second BaseCombatPacket b… |
| `00a9eb90` | ScalarDeletingDtor_Via00703350 | likely | Deleting destructor under an SEH frame: calls the destructor body FUN_00703350 (which the frame protects), and when bit 0 of the flags argument is set calls _f… |
| `00a9f2e0` | Console_ExecuteQueuedLines | likely | Walks the paged ring of pending console lines (count +0x70, start offset +0x74, page table +0x50, page count +0x54): for each slot it may grow the ring (FUN_00… |
| `00aa19e0` | PrintSlashCommandMatches | likely | When the argument is a wide string beginning with the character 0x2f it splits the text after the slash into an Array of IString through FUN_009874e0 and print… |
| `00ad3250` | CallSubobjectVirtual38OrZero | likely | If this+0x28 is nonzero, calls that subobject's vtable slot +0x38 and returns its result; otherwise returns zero. |
| `00ae3750` | StoreFieldAndDispatch | likely | Stores argument at index 0xa032, then loads the object's first-field vtable and tail-jumps through slot 1. |
| `00aed140` | CurveBuildFromKeyPairs | likely | Thiscall with three stack arguments. Returns 0 when the two keyed lists at param_2 and param_3 differ in length. Otherwise it resets this+0x1a8 and for each in… |
| `00b16e80` | DeletingDtor_00b16e80 | likely | Deleting destructor for an object whose destructor body is 00caacc0: raw bytes 56 8b f1 e8 (call 00caacc0) then test bit 0 of the flag byte at [esp+8] and, whe… |
| `00b16fc0` | ForwardToCachedScreen | likely | Forwarder with one stack argument: raw bytes 8b 49 0c 85 c9 74 05 e9 (jmp 00ca8200) c2 04 00 load the cached object from this+0xc, and when it is non-null tail… |
| `00b17090` | CopyCachedTextOrDefault | likely | Text accessor: loads this+0xc; when that cached object is non-null it copies the string at cached+0x18 into the destination passed as the stack argument throug… |
| `00b6f780` | FindObjectWithTypeId17 | likely | Two-step type lookup: reads the object at this+8, takes its +0xec member and calls that object's vtable slot 0 (the type id); when the id equals 0x11 (17) the … |
| `00c85010` | InitNestedObjects | likely | Stores the supplied vtable pointer, initializes two nested objects, allocates 0x260-byte blocks, and stores the resulting pointers. |
| `00cab260` | LoadingScreenSelectAndMovie | likely | Refreshes the active entries through 00caa940; when no entry id is pinned (+0x70c negative and +0x708 below 2) it draws a weighted random entry with 0071acb0, … |
| `00cab450` | LoadingScreen_EnsureWorkerAndSelect | likely | Show step of the loading-screen owner: when +0xc holds no selection it runs 00cab260; clears the flag at +0x2c of the client object from 00769710; when +8 is e… |
| `00cc6fa0` | CallIfContextReady | likely | Returns zero for a null argument or absent context; when context field +0x16c is nonzero, calls FUN_007dc240 with the argument and returns one. |
| `00d07eb0` | ProfilerThreadContext_Init | likely | Initializes an embedded sample hash list, stores two supplied values and a flag, and clears context counters and bucket storage. |
| `00d08c40` | CreateProfilerThreadContext | likely | Allocates and initializes a profiler context from supplied state, links it into context list, and increments list count. |
| `00d09100` | CopyProfilerContextsAndSamples | likely | Creates a context copy for each source thread, clones each sample node and counters, and rebuilds copied hash chains. |
| `00d095b0` | MergeProfilerSnapshotSamples | likely | Walks source contexts and samples, creates destination nodes, copies counters, and subtracts matching baseline counters when available. |
| `012d3690` | ReconnectStatusPost | likely | Thiscall with one byte flag: stores the flag at this+0x60; if it is 1 it builds a 0x14 event with type 0x5c and adds the text Please wait while you are reconne… |

### `engine.event`

**ClientTimerEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4fc30` | ClientTimerEvent::Dtor | confirmed | Restores the ClientTimerEvent vtable and calls the ExecuteLuaEvent base cleanup routine. |
| `00c4fc90` | ClientTimerEvent::Run | confirmed | Runs the Lua event and, when the supplied flag has bit 0 set, notifies the timer manager using this+0x3c; returns the result with its low byte cleared. |

**CommandProcessor::CommandEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009758c0` | CommandProcessor::CommandEventHandler::DtorBody | confirmed | Thiscall (ECX = this). Stores the CommandProcessor::CommandEventHandler vtable (immediate 0x180b24c) at this+0 and returns. It destroys no members. Its callers… |
| `009758d0` | CommandProcessor::CommandEventHandler::ScalarDeletingDtor | confirmed | Thiscall with one flag argument (ret 4). Stores the CommandEventHandler vtable at this+0, then calls _free (thunk 00d82463) on this when bit 0 of the flag is s… |

**Engine::EventSession** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144a100` | Engine::EventSession::EventSession | confirmed | Default constructor (ECX=this): stores Engine::EventSession::vftable at +0; zeroes the byte at +0x08 and the words at +0x0c..+0x1c, +0x30 and +0x34; sets the s… |
| `0144a520` | Engine::EventSession::ScalarDeletingDtor | confirmed | Vtable slot [0] of Engine::EventSession: calls the destructor body 0144a460 with ECX=this; when bit 0 of the flag argument is set it frees the object through _… |
| `01449420` | Engine::EventSession::ToXml | likely | Serialises one EventSession into XML text: builds a local Utility::XmlBuilder (0130e6c0), writes the scalar fields as name/value elements (sessionID +0xc, even… |

**GameEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e78e0` | GameEvent::GameEvent | confirmed | Thiscall (ECX = object) with one stack arg (param_2, stored at +0x3c). Stores DeepParticle::Timeline::Event::vftable, zeroes +4, +8, +0x10 and +0x14 and stores… |

**GameEventSequencer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3040` | GameEventSequencer::GameEventSequencer | confirmed | Installs GameEventSequencer::vftable, zeroes +4, +8, +0xc, +0x10, +0x14 and the 0x80-byte block starting at +0x18 via _memset, then zeroes +0x98 and +0x9c. Ret… |
| `007e8020` | GameEventSequencer::QueueEvent | likely | Thiscall virtual, slot [2] of GameEventSequencer::vftable (also slot [2] of RealTimeGameEventSequencer::vftable). Args: a float delay (param_2), the event (par… |

**GameTimeGameEventSequencer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e8110` | GameTimeGameEventSequencer::QueueEventAtGameTime | likely | Thiscall virtual, slot [2] of GameTimeGameEventSequencer::vftable. Args: float time (param_2), event (param_3), dword flags (param_4), float (param_5). Returns… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b56b80` | BuildSpeakerListenerEvent | likely | Builds speaker and listener event arguments, looks up the proxied actor by id, packages actor state and caller data into a Lua parameter array, dispatches it, … |

### `engine.factory`

**ComponentFactory** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013cd6e0` | ComponentFactory::~ComponentFactory | confirmed | fastcall(ECX = this): stores ComponentFactory::vftable at this, then calls DestroyListAndBuffer (013cd2e0) on the embedded member at this+4 and returns; no fre… |
| `013cd9e0` | ComponentFactory::DtorAndMaybeFree | confirmed | thiscall(ECX = object; one stack arg = flags byte): calls ComponentFactory::~ComponentFactory (013cd6e0) on the object, then frees the object with _free when f… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013cd0f0` | FactoryCreateByClassId | confirmed | thiscall(ECX = registry object; one stack arg = class id; ret 4): looks up the class id with the hash find 013ccca0 in the registry map at object+4 (container … |
| `013ce230` | RegistryMapStoreInt | likely | thiscall(ECX = registry object; stack args: key, creator value; ret 8): calls the int-key subscript 013ce1a0 on the map at registry+4 with the key, and stores … |

### `engine.log`

**GameUtil::Logging** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dc940` | GameUtil::Logging::AppendField18IfPresent | confirmed | Virtual method (shared vtable slot 3 of GameUtil::Logging, RewardBundleBase, ClientRewardManager and ClientRewardBundle): when the dword at this+0x20 is set it… |
| `008dc970` | GameUtil::Logging::AppendField28IfPresent | confirmed | Virtual method (shared vtable slot 4 of GameUtil::Logging, RewardBundleBase, ClientRewardManager and ClientRewardBundle): when the dword at this+0x30 is set it… |

**Loggable** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0104cd30` | Loggable_ResetAt10_ScalarDeletingDtor | confirmed | Stores Loggable::vftable at this+0x10 and calls _free (00d82463) on this only when bit 0 of the flag argument is set; returns this. |
| `0104edf0` | Loggable_ResetAt0c_ScalarDeletingDtor | confirmed | Stores Loggable::vftable at this+0xc and calls _free (00d82463) on this only when bit 0 of the flag argument is set; returns this. |
| `01061220` | Loggable::~Loggable | confirmed | Fastcall with the owner in ECX. Writes Loggable::vftable into the word at owner+0xc and returns. The stack-cookie frame bookkeeping is the only other effect; n… |
| `0104c7a0` | Loggable_ResetAt10 | likely | Stores Loggable::vftable into the object's +0x10 sub-object; returns void. |
| `0104c970` | Loggable_ResetAt10_Dup | likely | Stores Loggable::vftable at this+0x10; returns void. |
| `0104e9b0` | Loggable_ResetAt0c | likely | Stores Loggable::vftable at this+0xc; returns void. |
| `0104eb30` | Loggable_ResetAt0c_Dup | likely | Stores Loggable::vftable at this+0xc; returns void. |

**Logging::LoggingApi** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00893f20` | LoggingApiBuildRecord | confirmed | Virtual slot [16]: writes a record into a 1012-byte stack buffer: byte 1 first, then NUL-terminated copies of the strings at this+0xbf8 and this+0xd0c and this… |
| `00897050` | Logging::LoggingApiCtor | confirmed | Constructs the Logging::LoggingApi singleton: registers with the SOE logging shim FUN_010d7650("LOGAPI_100",...), installs the LogHandler base vtable at +0xbe8… |
| `008973e0` | Logging::LoggingApi::~LoggingApi | confirmed | Destructor: reinstalls the LoggingApi vtables at +0/+0xc0/+0xc4/+0xbe8, releases the owned log-file manager object at +0xd270 through FUN_0089a630 plus _free, … |
| `008963c0` | LogApiLeveledFormatThunk | likely | Adjustor thunk (11 bytes): add ecx,-0xbe8 then jmp 008961e0; it is not a body. Entered through the LoggingApi secondary vftable at +0xbe8, slot 1. |
| `008964a0` | LogApiControlMessage | likely | Virtual slot [18]: when the length argument is positive and the first byte of the message is 1, stores the second byte as the pending flag at this+0xbf0, calls… |
| `008964f0` | LogApiPeriodicTick | likely | Virtual slot [9]: under the guard at this+0x1058 and the TLS scope of this+0x1050, forwards its argument to 010d6950, runs the log tick 00899860 when the file … |
| `008977c0` | Logging::LoggingApiScalarDtor | likely | Scalar deleting destructor of Logging::LoggingApi: runs the full destructor FUN_008973e0 and releases the object with _free when bit 0 of param_2 is set. |

**Smart::ReportHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8eb30` | Smart::ReportHandler::DtorBody | confirmed | Destructor body (does not free this): stores the Smart::ReportHandler vftable at this. |
| `00f8ebc0` | Smart::ReportHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the Smart::ReportHandler vftable at this and calls _free(this) only when bit 0 of the flag argument is set. |
| `00f91260` | Smart::ReportHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the Smart::ReportHandler vftable at this and calls _free(this) only when bit 0 of the flag argument is set. |

**SoeGems::IRateTracker** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008977f0` | SoeGems::IRateTrackerScalarDtor | confirmed | Scalar deleting destructor of SoeGems::IRateTracker: stores the IRateTracker vtable at +0 and releases the object with _free when bit 0 of param_2 is set. |
| `008978b0` | SoeGems::IRateTracker::~IRateTracker | confirmed | Trivial destructor body of SoeGems::IRateTracker: writes the IRateTracker vtable pointer at +0 and returns; reached only from EH unwind funclets (12 of them). |
| `00897ce0` | SoeGems::IRateTrackerScalarDtorB | confirmed | Second emission of the IRateTracker scalar deleting destructor: stores the IRateTracker vtable at +0 and releases the object with _free when bit 0 of param_2 i… |

**SoeGems::LoggingHeader** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d0a4e0` | SoeGems::LoggingHeader::LoggingHeader | confirmed | Installs LoggingHeader and IString<char> vtables, sets shared empty-string pointer, and zeros string metadata and flags. |
| `00d0a510` | SoeGems::LoggingHeader::~LoggingHeader | confirmed | Restores LoggingHeader and IString vtables, decrements shared string reference count, and frees buffer when count reaches zero. |
| `00d0a550` | SoeGems::LoggingHeader::ScalarDeletingDtor | confirmed | Runs LoggingHeader string cleanup, frees object when deleting flag is set, and otherwise returns it. |

**SoeGems::TRateTracker<128>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00897c70` | SoeGems::TRateTracker128Ctor | confirmed | Constructor of SoeGems::TRateTracker<128>: installs the class vtable, stores the default window length 1000 at +0x410, zeroes the two 128-entry int arrays at +… |
| `00897cd0` | RateTracker128GetWindowMs | confirmed | Returns the dword at object+0x410, the rate-window field initialized by FUN_00897c70 and set by FUN_00897910. |
| `00897df0` | RateTracker128AddSample | confirmed | Records a sample: advances the window (FUN_00897d00), adds param_2 to the current bucket's sum at +4 + (index & 0x7f)*4 and increments the matching count slot … |
| `00897e50` | RateTracker128ScaledSum | confirmed | Advances the window (FUN_00897d00) and then returns runningSum(+0x408) * param_2 / window(+0x410), a 64-bit signed division, i.e. the accumulated total rescale… |
| `00897e80` | RateTracker128ScaledCount | confirmed | Advances the window (FUN_00897d00) and returns sampleCount(+0x40c) * param_2 / window(+0x410) via __alldiv, i.e. the per-interval event count. |
| `00897eb0` | RateTracker128Average | confirmed | Advances the window (FUN_00897d00) and returns runningSum(+0x408) divided by sampleCount(+0x40c) when the count is positive, otherwise 0. |

**T4ErrorHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8e7f0` | T4ErrorHandler::ScalarDeletingDtor | confirmed | Sets the vptr to T4ErrorHandler::vftable; when bit 0 of the flag argument is set it calls _free (00d82463) on this, then returns this (ret 4). |

**T4GlobalErrorHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8e820` | T4GlobalErrorHandler::Ctor | confirmed | Constructor: clears the byte at this+4, sets the vptr to T4GlobalErrorHandler::vftable, zeroes the dword at this+8, calls 00e68e60(0,0,1), then sets the member… |
| `00e8e890` | T4GlobalErrorHandler::DtorBody | confirmed | Sets the vptr to T4GlobalErrorHandler::vftable, calls 00e6f540 (member cleanup), then sets the vptr to T4ErrorHandler::vftable (base). Returns nothing. |
| `00e8e9a0` | T4GlobalErrorHandler::SetTarget | likely | Virtual slot [0] of T4GlobalErrorHandler::vftable. If param_2 is null or the target at *param_2 has a null +0x18 field, it calls vtable slot [1] of this and re… |

**(no class)** (49)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008989c0` | LogFileWriteQueueWithFailover | confirmed | Core log write path of a log-file object: optionally expands the name pattern (FUN_00898400), takes timestamps, purges buffered text through the sink reached a… |
| `00899050` | LogApiUpdateDateStrings | confirmed | Rolls the log manager's date/time strings: formats the time with "%02d:%02d:%02d" into +0x828 and compares its day component with the stored day string at +0x9… |
| `00899380` | LogFileWriteLine | confirmed | Formats and queues one log line: takes the file timestamp, builds a tab-separated header with FUN_006fc570 using the literal "%s\t%s\t%s\t%d\t%d\t%d\t%lld\t" a… |
| `00899950` | LogApiWriteToChannel | confirmed | Writes one log line to a named channel: copies the channel name (param_4) into a local SoeUtil::StringFixed<1024> via FUN_00402ce0 and FUN_00703b20, looks the … |
| `0089a630` | LogFileManagerDtor | confirmed | Destructor of the heap-allocated log-file manager: runs the map destructor FUN_00899b50 first, then destroys the six SoeUtil::IStringFixed<char,64> members (re… |
| `0089a8b0` | LogFileManagerCtor | confirmed | Constructor of the heap-allocated log-file manager: runs FUN_00897c70 twice to build the two embedded SoeGems::IRateTracker<128> objects at +0 and +0x414, init… |
| `00d07500` | PrintProfilerReport | confirmed | Formats per-thread profiler timing rows, callstack and flat summaries, and writes them through string append routines. |
| `00f99fd0` | LogFormatted | confirmed | Formats a message with __vsnprintf_s into a 0x800-byte stack buffer (level 2 prefixes 'ERROR: ' through _strncpy_s), appends a newline through _strncpy_s, and … |
| `00f9a100` | LogWin32Error | confirmed | Calls FormatMessageA (0x1000 flags, 0x400 language, 0x40 buffer size) on the given error code into a 64-byte stack buffer, terminates it at the returned length… |
| `01350760` | AppendFormattedLogFile | confirmed | Thiscall logger (ECX = object, stack args: name, format, va_list): builds the file name from a directory string, the name argument and the int at [this+0x64] w… |
| `013539c0` | PlaybackLogUnsupported | confirmed | Calls 013508b0 with this, the format We can't play back log files (%s) on Windows yet., and the stack argument; then returns 0 (low byte of the callee result c… |
| `013541f0` | AppendGameLogLine | confirmed | When the gate flags at +0x14d, +0x318, +0x12c and +0x389 allow it, formats wa_game_%d.log with the game id at +0x64, opens it in append mode, optionally writes… |
| `00700f90` | LogSinkForward | likely | Forwards a log call to the global sink when the severity argument is at or below the threshold DAT_01b29294 and a sink is set: a NULL message is replaced by th… |
| `008953f0` | QueueListUnlinkAndFree | likely | Unlinks the entry param_2 from the doubly linked list (next at +0x170, prev at +0x174, head at this+4, tail at this+8), decrements the count at this+0xc, destr… |
| `008955c0` | QueueListClear | likely | Loops while the head at this+4 is non-null: unlinks the head (as in 008953f0), decrements the count, destroys its strings with 00894be0 and releases it through… |
| `00895fa0` | LogApiWriteMultiline | likely | Under the TLS scope of this+0x1050, when the reentrancy flag at this+0x104c is clear: sets it, falls back to the default directory string at this+0xd160 when t… |
| `008963d0` | LogApiFlushQueueEntries | likely | Under the guard at this+0x1058: when the state code from 00893340 is 1, walks the queue from the head at this+0x10e8 through the next links at +0x170 and seria… |
| `008973b0` | LoggingApiDtorThunkC0 | likely | Adjustor thunk (bytes 81 e9 c0 00 00 00 e9 05 04 00 00): subtracts 0xc0 from this and jumps to the LoggingApi scalar deleting destructor FUN_008977c0, providin… |
| `008973c0` | LoggingApiDtorThunkBE8 | likely | Adjustor thunk (bytes 81 e9 e8 0b 00 00 e9 f5 03 00 00): subtracts 0xbe8 from this and jumps to the LoggingApi scalar deleting destructor FUN_008977c0, providi… |
| `008973d0` | LoggingApiDtorThunkC4 | likely | Adjustor thunk (bytes 81 e9 c4 00 00 00 e9 e5 03 00 00): subtracts 0xc4 from this and jumps to the LoggingApi scalar deleting destructor FUN_008977c0, providin… |
| `00897910` | TRateTracker128Reset | likely | Resets a 128-slot rate tracker: stores the new window length (ms) at +0x410, zeroes the two 128-entry int arrays at +4 and +0x204 with _memset(...,0x200), and … |
| `00897d00` | RateTrackerAdvanceWindow | likely | Advances a 128-slot rate tracker to the current time: reads the 64-bit time from FUN_007027f0, computes bucket = (time*128)/window(+0x410) & 0x7fffffff, and wh… |
| `008980e0` | HashListFindByName | likely | Finds a node in the name-keyed 8192-bucket hash list at container+0xa60: hashes the C string with FUN_00705bc0, walks the bucket chain at +0x8dc while the stor… |
| `00898400` | ExpandLogNamePattern | likely | Expands the log-file name pattern at param_1[0xb] into a 2048-byte stack buffer: ## escapes to #, #D/#H/#M/#Y substitute the pre-formatted digit strings held i… |
| `00898e20` | LogFileCtor | likely | Constructor of one log-file object: initialises two SoeUtil::StringFixed<1024> members (name at +0x28/+0x440), an embedded byte-array list at +0x8b8, stores th… |
| `00898ff0` | LogApiFlushAllFiles | likely | Walks the log-file list at +0xa54 through the +0x8d0 next links and flushes each file: refreshes its timestamp at +0x18, and when the text buffer holds data or… |
| `00899220` | LogFileCloseAndFree | likely | Refreshes the timestamp at +0x18, writes buffered or queued data through FUN_008989c0, and calls _free for the text buffer at +4; raw bytes after the decompile… |
| `00899550` | LogFileNeedsFlush | likely | Predicate for the periodic log-service tick: refreshes the write timestamp and returns 1 when the elapsed time exceeds the flush interval (*api+0xa2c) times 10… |
| `008996f0` | LogApiAddFile | likely | Creates and registers one log file: allocates the 0x8e0-byte node through the owning map's virtual allocator (vtable slot 1 at offset 4), constructs the file o… |
| `00899780` | LogFileScalarDtor | likely | Scalar deleting destructor of a log-file object: runs the close-and-free routine FUN_00899220 and additionally _free's the object when bit 0 of param_2 is set. |
| `008997a0` | LogApiAddFileByName | likely | Hashes the log-file name with FUN_00705bc0 and registers a new file keyed by that hash through FUN_008996f0, passing the name pointer in a local pair. |
| `008997e0` | LogApiRemoveFile | likely | Removes and destroys one log-file node: unlinks it from the hash chain (FUN_00897af0), patches the ordered-list neighbours at node+0x8d0/+0x8d4 and the map hea… |
| `00899860` | LogApiTick | likely | Periodic log-service tick: when 200 timer units have elapsed it refreshes the date strings (FUN_00899050), resets the counters at +0xa40/+0xa44/+0xa48 and then… |
| `00899ac0` | RemoveAllLogFiles | likely | Drains a log-file map by repeatedly taking the head node at +8 and closing/removing it with FUN_008997e0 until the list is empty; the loop count is decremented… |
| `009124c0` | LoggableBaseResetCleanup | likely | thiscall with no arguments, under an SEH frame with a cookie check: stores Loggable vftable 0x017e8c54 at this+0x148; calls thiscall FUN_00905ca0 with ECX = th… |
| `00bab600` | ScalarDeletingDtor_00bab600 | likely | Calls FUN_009124c0, frees the supplied object when flag bit 0 is set, and otherwise returns its pointer. |
| `00d0a5a0` | LogFormattedToSink_00704db0 | likely | Formats variadic arguments into a temporary string, invokes object's virtual formatter, and forwards result with the fixed logging tag. |
| `00d0a680` | LogFormattedToSink_00704dd0 | likely | Formats variadic arguments into temporary string, calls object's virtual formatter, and forwards message to second logging sink. |
| `00d0a760` | LogFormattedToSink_00702ab0 | likely | Formats variadic arguments into temporary string, calls object's virtual formatter, and forwards result to selected sink. |
| `00d0a840` | LogFormattedToSink_006faad0 | likely | Formats variadic arguments into temporary string, calls object's virtual formatter, and forwards result to selected sink. |
| `00d0a920` | LogFormattedToSink_0089aa90 | likely | Formats variadic arguments into temporary string and routes it through a forwarding logger call with a leading flag. |
| `00e8eaf0` | GlobalErrorHandlerInstance | likely | Returns the address of the static object at DAT_01bfde98. On the first call it sets the flag bit in DAT_01bfdebc, constructs the object through 00e8e820 (T4Glo… |
| `00e91950` | ErrorQueueFlush | likely | Gets the GlobalErrorHandler instance (00e8eaf0). When its byte at +4 is clear it sets the count at this+0xcc to 0. Otherwise it calls vtable slot [2] of the ha… |
| `012d28c0` | LogToWaClientLog | likely | Appends one formatted line to wa_client.log: opens the file in append mode, asserts the open succeeded (ClientApplication.cpp line 0x203), formats param_2 with… |
| `01350850` | LogWaActionIfEnabled | likely | Cdecl helper (arg1 = object, moved to ECX; arg2 = format; the variadic arguments follow): when the dword at [object+0x12c] is non-zero, calls 01350760 with ECX… |
| `013508b0` | LogWaGeneralIfEnabled | likely | Cdecl helper (arg1 = object, moved to ECX; arg2 = format; the variadic arguments follow): when the dword at [object+0x12c] and the byte at [object+0x22c] are b… |
| `01353a90` | FormatToSinkIfEnabled | likely | Formats a varargs string into a 1000-byte stack buffer with __vsnprintf (varargs at stack+0xc) and passes the buffer to 013508b0 with this, but only when the d… |
| `0135a930` | CleanupWaLogFiles | likely | No arguments. Deletes wa_xtra.log, wa_properties.log and wa_network.log through 00d8e520. It then takes the global directory string from 013aef60; when 013aef6… |
| `0136b570` | FormatTimestampedLog | likely | When the flag byte at this+0x14d is not 1, formats its varargs with __vsnprintf into a 1000-byte stack buffer, copies the text into a std::string, stamps it wi… |

### `engine.loop`

**Messaging::EngineLoopBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ee9a0` | Messaging::EngineLoopBase::ctor | confirmed | Fastcall (this in ECX, returns this): builds the std::string "EngineLoop" (10 chars), passes it to base init 012eaa90, frees the string buffer when its capacit… |
| `012eea80` | Messaging::EngineLoopBase::ScalarDeletingDtor | confirmed | Thiscall with a flag byte (ret 4 shape not needed): calls 012e4590 (the destructor body, fastcall with ECX=this) and frees this only when flag bit 0 is set, th… |
| `012ee390` | Messaging::EngineLoopBase::DispatchKeyed | likely | Thiscall with one stack argument msg (ret 4). Key = first dword of msg (FUN_012f8c60 reads *ECX with ECX = msg). Looks up the map at this+0x5c with 012ee2d0. I… |

### `engine.messaging`

**Engine::ChatServiceBase** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013dbc40` | Engine::ChatServiceBase::~ChatServiceBase | confirmed | Stores Engine::ChatServiceBase vtable 018e20d0 into *this, calls 00f98150 (member cleanup), then calls 01230d30 (base cleanup). Does not free the object (94 by… |
| `013dbca0` | Engine::ChatServiceBase::Ctor | confirmed | Constructor (2952 bytes, fastcall, returns this). Stores vtable 018e20d0 into *this, calls 00f987e0 (base setup), then registers 44 chat command entries: for e… |
| `013dc830` | Engine::ChatServiceBase::ScalarDeletingDtor | confirmed | Vtable slot [0] of Engine::ChatServiceBase (30 bytes, ret 4): calls the destructor body 013dbc40 on the object, then if bit 0 of the argument is set calls _fre… |
| `013dbb60` | Engine::ChatServiceBase::LookupStringOrEmpty | likely | Vtable slot [3] shared by Engine::ChatService and Engine::ChatServiceBase (211 bytes). Looks up the key argument in the tree at receiver+8 through 00f95b90, ta… |

**Messaging::AudioLoop** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e55b0` | Messaging::AudioLoop::ScalarDeletingDtor | confirmed | Virtual slot [0] of AudioLoop: thiscall with a flag byte. Calls the LoopBase dtor body 012ea910 on this (it does not call the FRTWAudioLoop dtor), then calls _… |
| `012e5500` | Messaging::AudioLoop::Ctor | likely | __fastcall (ECX = this), returns this. Calls the FRTWAudioLoop constructor 012e53f0 on this, then stores the Messaging::AudioLoop vftable at this. |

**Messaging::AudioLoopBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013da300` | Messaging::AudioLoopBase::Ctor | confirmed | Builds a temporary std::string from the literal 'AudioLoop' (9 chars) with 0042131b, passes it to 012eaa90, frees the temporary when its capacity exceeds 0xf, … |
| `013da3d0` | Messaging::AudioLoopBase::ScalarDeletingDtor | confirmed | Vtable slot [0] of Messaging::AudioLoopBase. Calls 012ea910 (destructor body), then if bit 0 of the second argument is set calls _free on this and returns this… |

**Messaging::EngineLoop** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e47d0` | Messaging::EngineLoop::ScalarDeletingDtor | confirmed | Virtual slot [0] of EngineLoop: thiscall with a flag byte. Calls the EngineLoopBase dtor body 012e4590 on this, then calls _free(this) only when bit 0 of the f… |
| `012e4720` | Messaging::EngineLoop::Ctor | likely | __fastcall (ECX = this), returns this. Calls the FRTWEngineLoop constructor 012e4610 on this, then stores the Messaging::EngineLoop vftable at this. |

**Messaging::EngineLoopBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e4590` | Messaging::EngineLoopBase::DtorBody | likely | __fastcall (ECX = this). Frees the buffer at this+0x80 when it is non-null, zeroes the three words at +0x80, +0x84 and +0x88 (begin, end, capacity), then calls… |

**Messaging::FRTWAudioLoop** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e54a0` | Messaging::FRTWAudioLoop::ScalarDeletingDtor | confirmed | Virtual slot [0] of FRTWAudioLoop: thiscall with a flag byte. Calls the LoopBase dtor body 012ea910 on this, then calls _free(this) only when bit 0 of the flag… |
| `012e53f0` | Messaging::FRTWAudioLoop::Ctor | likely | __fastcall (ECX = this), returns this. Calls 013da300 on this first, then stores the Messaging::FRTWAudioLoop vftable at this. |

**Messaging::FRTWEngineLoop** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e4610` | Messaging::FRTWEngineLoop::Ctor | confirmed | __fastcall (ECX = this), returns this. Calls the EngineLoopBase constructor 012ee9a0 on this, then stores the Messaging::FRTWEngineLoop vftable at this. |
| `012e46c0` | Messaging::FRTWEngineLoop::ScalarDeletingDtor | confirmed | Virtual slot [0] of FRTWEngineLoop: thiscall with a flag byte. Calls the EngineLoopBase dtor body 012e4590 on this, then calls _free(this) only when bit 0 of t… |

**Messaging::FRTWUILoop** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013da4a0` | Messaging::FRTWUILoop::~FRTWUILoop | confirmed | Stores Messaging::FRTWUILoop::vftable 018e208c into *this, stores this in its local, and calls 012ea910 (the shared messaging-loop destructor body). It does no… |
| `013da4f0` | Messaging::FRTWUILoop::Ctor | confirmed | Calls 013da5e0 (base-level constructor of the UI loop, next in this shard) on the object, then stores vtable 018e208c into *this. Returns this. |
| `013da550` | Messaging::FRTWUILoop::ScalarDeletingDtor | confirmed | Vtable slot [0] of Messaging::FRTWUILoop. Stores vtable 018e208c, calls 012ea910 (shared destructor body), then if bit 0 of the second argument is set calls _f… |

**Messaging::LoopBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e5840` | Messaging::LoopBase::VirtualSlot2 | likely | Stdcall-style (ECX not used) with two stack arguments (mode dword, object pointer), ret 8. When mode is 0 or 1 and the object is non-null, calls the object's v… |
| `012e6550` | Messaging::LoopBase::VirtualSlot3 | likely | Stdcall-style body with two stack arguments (mode dword, pointer); ECX is not used (the ret count is not decoded). When mode is 0 or 1 and the pointer is non-n… |
| `012e7f80` | Messaging::LoopBase::VirtualSlot1 | likely | Thiscall queue pump (ECX = loop, 0x74 count at this+0x74 clamped by FUN_013daae0). Each iteration reads an item (013db180) and tests 013db9f0: when zero and 01… |

**Messaging::UILoop** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e57a0` | Messaging::UILoop::ScalarDeletingDtor | confirmed | Virtual slot [0] of UILoop: thiscall with a flag byte. Calls 013da4a0 on this, then calls _free(this) only when bit 0 of the flag is set. Returns this. |
| `012e5680` | Messaging::UILoop::Ctor | likely | __fastcall (ECX = this), returns this. Calls 013da4f0 on this first, then stores the Messaging::UILoop vftable at this. |
| `012e5730` | Messaging::UILoop::GetOrCreate | likely | No arguments. If the global DAT_01cbd7e8 is null: allocates 0x78 bytes with operator new (00d833a6); when non-null it constructs a UILoop there through 012e568… |

**Messaging::UILoopBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013da5e0` | Messaging::UILoopBase::Ctor | confirmed | Builds a temporary std::string from the literal 'UILoop' (6 chars) with 0042131b, calls 012eab60 with that string and the constant 0x80, frees the temporary wh… |
| `013da690` | Messaging::UILoopBase::ScalarDeletingDtor | confirmed | Vtable slot [0] of Messaging::UILoopBase. Calls 012ea910 (destructor body), then if bit 0 of the second argument is set calls _free on this and returns this. |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e4670` | EngineLoopDtorCleanup | likely | Plain wrapper with no arguments: calls the EngineLoopBase destructor body 012e4590 with ECX from a stack slot; returns void. |
| `012e5800` | MessagingLoopPostThunkA | likely | Thunk: adds 0x20 to ECX and jumps to 013db6d0 (the accepted MessagingLoopPost), so the callee receives the queue at this+0x20. |
| `012e5810` | MessagingLoopPostThunkB | likely | Same thunk as 012e5800: adds 0x20 to ECX and jumps to 013db6d0 (MessagingLoopPost). |
| `012e5820` | MessagingLoopPostSecondThunk | likely | Thunk: adds 0x20 to ECX and jumps to 013db7f0 (accepted MessagingLoopPost_PayloadSecond). Callers push the action first, then call 012e5730 for the singleton a… |
| `013db030` | ReleaseHandleCsAndVectors | likely | Fastcall (receiver in ECX): CloseHandle on the dword at receiver+0x38 and zeroes it; DeleteCriticalSection on receiver+0x20; frees the buffer at receiver+0x14 … |
| `013db180` | CsPopBackDword | likely | Fastcall (receiver in ECX, 128 bytes): EnterCriticalSection on receiver+0x20; if the begin pointer at +4 is non-null and the count (end-begin)>>2 is non-zero, … |
| `013db650` | InitVectorsCsAndEvent | likely | Fastcall (receiver in ECX): zeroes the dwords at +4, +8, +0xc, +0x14, +0x18 and +0x1c, calls InitializeCriticalSection on receiver+0x20, then creates an unname… |
| `013db6d0` | MessagingLoopPost | likely | Thiscall (receiver = loop, ECX; one stack argument, the item pointer). If the item pointer is non-zero it takes the loop critical section (receiver+0x20). If t… |
| `013db7f0` | MessagingLoopPost_PayloadSecond | likely | Same body as 013db6d0 (282 bytes, thiscall, one stack argument) with the payload passed in the second position: calls 013db9b0(0, payload) to build a new node … |
| `013db9b0` | PairNodeInit_ret8 | likely | Thiscall (receiver in ECX, two stack arguments, ret 8): stores the first argument at [ecx] and the second at [ecx+4], and returns the receiver in EAX (18 bytes… |
| `013db9e0` | PairNodeInit_noret | likely | Thiscall (receiver in ECX, two stack arguments, ret 8): stores the first argument at [ecx] and the second at [ecx+4] (16 bytes); EAX is the first argument, not… |

### `engine.persistent`

**MessageText** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01401df0` | MessageText::MessageText | confirmed | Constructor for MessageText: stores PersistentBase::vftable, PersistentComponent::vftable and MessageText::vftable into the object, zeroes the three words at +… |
| `01401e90` | MessageText::TypeName | confirmed | Virtual slot [11] of MessageText. Returns the constant C string 'MessageText'. |
| `01401ed0` | MessageText::DtorBody | confirmed | Non-deleting destructor body: stores MessageText::vftable into *arg1, calls 012ce9d0 on the object, then stores PersistentComponent::vftable and PersistentBase… |
| `01402190` | MessageText::~MessageText | confirmed | Virtual slot [0] of MessageText (scalar deleting destructor): calls 01401ed0 (MessageText::DtorBody) on the object and frees it through _free when bit 0 of the… |
| `01401ea0` | MessageText::GetStaticDword | likely | Virtual slot [12] of MessageText. Returns the dword at global DAT_01cbe220. |
| `01401eb0` | MessageText::SetStaticDword | likely | Virtual slot [13] of MessageText. Stores its stack argument into global DAT_01cbe220 and returns (ret 4). |
| `01402110` | MessageText::Create | likely | Allocates 0x1c bytes with operator new (00d833a6); if the allocation is non-null, calls the MessageText constructor 01401df0 on it and returns the EAX left aft… |
| `014024f0` | MessageCopyFields | likely | Thiscall with one argument (source object): if the source is null it calls 012f5a60 with the assertion text 'message' and the Message.cpp file, line 0x82; then… |

**PersistentBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279330` | PersistentBase::DtorBody | confirmed | Destructor body: stores PersistentBase::vftable into *this and returns. Called from 98 MSVC EH unwind funclets. |
| `01279370` | PersistentBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores PersistentBase::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |

**PersistentComponent** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279390` | PersistentComponent::DtorBody | confirmed | Destructor body: stores PersistentComponent::vftable then PersistentBase::vftable into *this and returns. Called from 87 MSVC EH unwind funclets. |
| `01279400` | PersistentComponent::GetClassName | confirmed | Returns the constant C string PersistentComponent (6 bytes: mov eax, imm32; ret). Virtual slot [11] of PersistentComponent::vftable. |
| `01279440` | PersistentComponent::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores PersistentComponent::vftable then PersistentBase::vftable into *this, then calls _free(this) when bit 0 of the flag argument… |
| `0127a250` | PersistentComponent::SetBaseVftables | confirmed | Stores PersistentBase::vftable then PersistentComponent::vftable into *this and returns. Two-step base-phase vtable store. |
| `012fa6c0` | PersistentComponent::Ctor | confirmed | Fastcall with ECX = object: stores PersistentBase::vftable into *this and then overwrites it with PersistentComponent::vftable; returns nothing. |

**PersistentRoot** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013475d0` | PersistentRoot::PersistentRoot | confirmed | Constructor (fastcall, ECX=this). Stores PersistentBase::vftable and then PersistentRoot::vftable at this+0. |
| `01347630` | PersistentRoot::DtorBody | confirmed | Destructor body (fastcall). Stores PersistentRoot::vftable then PersistentBase::vftable at this+0 and returns. |
| `013476b0` | PersistentRoot::ScalarDeletingDtor | confirmed | Thiscall with a flag argument (ret 4). Stores PersistentRoot::vftable then PersistentBase::vftable, then calls _free on this when bit 0 of the flag is set. Ret… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0132d830` | PersistentReadPolymorphic | likely | Stdcall-style (param_1 = reader state, param_2 = pointer slot for the result): reads one integer with 013c24d0; returns 0 when that read fails; releases a prev… |
| `01386b50` | RegisterFactory_0xaa | likely | Calls 013ce130 with key 0xaa and the function 01386ae0, then calls 013ce230 with the same key and function. It registers the factory in the persistent-class re… |
| `01401bf0` | MessageSetField8Checked | likely | Thiscall with one stack argument (ret 4): if the argument equals -1 it calls 012f5a60 with the assertion text 'v != -1', the file '..\common\common\messages\Me… |

### `engine.serialization`

**FormatSpecificData** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01434380` | FormatSpecificData::GetClassName | confirmed | Returns the address of the literal FormatSpecificData (mov eax,imm32; ret). No arguments and no side effects. |
| `01434490` | FormatSpecificData::~FormatSpecificData | confirmed | Scalar deleting destructor: calls the destructor body FUN_014342b0 on this, then if bit 0 of the flag argument is set it calls _free on this; returns this. |
| `014342b0` | FormatSpecificData::DtorBody | likely | Stores the FormatSpecificData vtables at +0x0 and +0x4, runs member teardown (FUN_00521a9c, FUN_01432fc0 which destroys the tree member, FUN_01325780) and FUN_… |
| `01434390` | FormatSpecificData::GetGlobal_01cbe2b0 | likely | Returns the dword at the absolute address DAT_01cbe2b0 (mov eax,[abs]; ret). It does not read this. |
| `014343a0` | FormatSpecificData::SetGlobal_01cbe2b0 | likely | Stores its stack argument into the absolute address DAT_01cbe2b0 (mov eax,[esp+4]; mov [abs],eax; ret 4). It does not touch this. |
| `014343d0` | FormatSpecificData::FormatSpecificData | likely | Stores the PersistentBase and PersistentComponent vtables at +0 after calling FUN_014240f0 on this, then stores the FormatSpecificData vtables, calls FUN_01325… |
| `014344b0` | FormatSpecificData::Create | likely | Allocates 0x44 bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_014343d0 (constructor) and returns its result, otherwise returns 0. |

**PersistentBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013aede0` | PersistentBase::WriteClassHeader | confirmed | Sets the stream state (arg1, the writer) to 2 via 013c2250, writes this->vtbl[6]() through 013c23f0 (SerialAppendInteger), then this->vtbl[8]() the same way, a… |
| `013aee40` | PersistentBase::ReadClassHeader | confirmed | Sets the stream arg1 to state 1 via 013c2250, reads one int with 013c24d0 and compares it with this->vtbl[6]() and then this->vtbl[7](). If both differ it call… |
| `013aed70` | PersistentBase::WriteToNewStream | likely | Virtual slot 1: allocates a 0x14-byte block with operator new (00d833a6), zeroes it through 013c2210 and calls this->vtbl[2] with that block (the class-header … |

**PropertySet** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01312720` | PropertySet::TypeName | confirmed | Virtual slot [11] of PropertySet::vftable: returns the address of the C string "PropertySet" (mov eax, imm32; ret). |
| `01312730` | PropertySet::GetStaticDword | confirmed | Virtual slot [12] of PropertySet::vftable: returns the dword at DAT_01cbda78 (mov eax,[01cbda78]; ret). |
| `01312740` | PropertySet::SetStaticDword | confirmed | Virtual slot [13] of PropertySet::vftable: stores its one argument into DAT_01cbda78. |
| `01312930` | PropertySet::PropertySet | confirmed | Fastcall constructor. Stores PersistentBase::vftable, then PersistentComponent::vftable, then PropertySet::vftable into this, sets the unwind state, calls 0131… |
| `013129a0` | PropertySet::ScalarDeletingDtor | confirmed | Virtual slot [0] of PropertySet::vftable, scalar deleting destructor shape: calls the destructor body 01312690, frees this with _free when bit 0 of the flag ar… |
| `01312690` | PropertySet::~PropertySet | likely | Fastcall destructor body. Stores PropertySet::vftable at this, clears the entry map at this+4 through 01311020 (PtrMapDeleteValuesAndClear), calls 01312620, th… |

**SCEA::BinXML::BadStream** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9ed60` | BadStream::Ctor | confirmed | Fastcall constructor: calls 00d837f0 (base exception constructor) with an EH cookie, then stores SCEA::BinXML::BadStream::vftable at +0 and returns this. |
| `00f9ef00` | BadStream::GetName | confirmed | Virtual slot 1 of SCEA::BinXML::BadStream: returns the address of the string literal 'Bad Stream'. |
| `00f9f3e0` | BadStream::ScalarDeletingDtor | confirmed | Virtual slot 0 of SCEA::BinXML::BadStream: calls 00d83915 (base destructor body) on this, frees this through _free when bit 0 of the byte argument is set, and … |
| `00f9f740` | BadStream::CopyCtor | confirmed | Thiscall copy constructor: calls std::exception copy constructor 00d83867 with the source, then stores SCEA::BinXML::BadStream::vftable at +0. |

**SCEA::BinXML::Binary** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9ee30` | Binary::DefaultCtor | confirmed | Fastcall constructor: stores the Streamable vtable, clears byte +4, byte +5 and the words at +8, +0xc and +0x10, then stores SCEA::BinXML::Binary::vftable at +… |
| `00f9f020` | Binary::CopyCtor | confirmed | Thiscall constructor from a source handle: calls 00f9e670 on the source, then stores SCEA::BinXML::Binary::vftable at +0. |
| `00f9f440` | Binary::ScalarDeletingDtor | confirmed | Virtual slot 0 of SCEA::BinXML::Binary: calls 00f9e6e0 (Streamable dtor body) on this, frees this through _free when bit 0 of the byte argument is set, and ret… |

**SCEA::BinXML::Streamable** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9e670` | Streamable::Ctor | confirmed | Thiscall constructor with a source handle: stores SCEA::BinXML::Streamable::vftable at +0, clears byte +4, byte +5, the pointer at +8, and the words at +0xc an… |
| `00f9e6e0` | Streamable::DtorBody | confirmed | Stores SCEA::BinXML::Streamable::vftable at +0, decrements the refcount at +8 and frees that block and the buffer at +0xc when the count reaches zero. Called f… |

**SCEA::BinXML::String** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9e940` | String::Ctor | confirmed | Thiscall constructor: calls 00f9e670 (Streamable constructor) on the source argument, then stores SCEA::BinXML::String::vftable at +0 and returns this. |
| `00f9e9e0` | String::BumpCachedLength | confirmed | Virtual slot 7 of SCEA::BinXML::String: when the cached length is non-zero and the byte before the terminator is non-zero and the byte at the cached end is NUL… |
| `00f9efc0` | String::Ctor2 | confirmed | Thiscall constructor identical to 00f9e940: calls 00f9e670 on the source argument, then stores SCEA::BinXML::String::vftable at +0. |

**T4BaseStream** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e73ac0` | T4BaseStream::T4BaseStream | confirmed | thiscall, one stack argument: stores T4BaseStream::vftable at [this], zeroes the dwords at +4, +8 and +0xc, stores the argument at +0x10, and zeroes the bytes … |
| `00e73cd0` | T4BaseStream::~T4BaseStream | confirmed | fastcall (ECX = this): stores T4BaseStream::vftable; when the owned sub-object pointer at +4 is non-NULL and the bytes at +0x14 and +0x15 are zero, it calls vi… |
| `00e748c0` | T4BaseStream::~T4BaseStream_Deleting | confirmed | thiscall, byte flag in stack arg 1: stores T4BaseStream::vftable; when the owned buffer (+4) is non-NULL and bytes +0x15 and +0x14 are both zero it calls the b… |

**T4ContextList** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e75be0` | T4ContextList::Resize | confirmed | thiscall, new count in stack arg 1: when the current 16-bit count at +0 is below the new count, it (re)allocates the pointer array at +4 (size count*4 plus a h… |
| `00e75e00` | T4ContextList::Append | likely | fastcall (ECX = list): reads the 16-bit capacity at +0 and the 16-bit used count at +2; when capacity is not above used it grows the list through T4ContextList… |

**T4StringBuffer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e73af0` | T4StringBuffer::T4StringBuffer | confirmed | thiscall, C string in stack arg 1 and length in arg 2 (-1 = strlen): stores T4BaseStream::vftable, zeroes +4 and +8, computes the length (strlen+0 or the given… |
| `00e73c20` | T4StringBuffer::T4StringBuffer | confirmed | thiscall, source object pointer in stack arg 1: stores T4BaseStream::vftable, zeroes +4, copies the source's dwords at +8, +0xc, +0x10 and bytes at +0x14, +0x1… |

**(no class)** (290)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008e2410` | ReadFloat4 | confirmed | Reads four consecutive 32-bit values into the destination float4; on exhaustion writes zero, marks the reader error byte, and clamps its cursor to the end; con… |
| `00f9f840` | BinXmlReadU16BE | confirmed | Fastcall reader: asserts two bytes or fewer remain, throws the BadStream exception otherwise, reads two bytes through the source's vtable slot +4, advances the… |
| `00f9f8e0` | BinXmlReadU32BE | confirmed | Fastcall reader: asserts four bytes or fewer remain, throws the BadStream exception otherwise, reads four bytes through the source's vtable slot +4, advances t… |
| `00f9f9a0` | BinXmlReadFloatBE | confirmed | Fastcall reader: asserts four bytes or fewer remain, throws the BadStream exception otherwise, reads four bytes, and returns them byte-swapped interpreted as a… |
| `00f9fb50` | BinXmlReadStringSized | confirmed | Thiscall with a string object: reads the expected length, reads or allocates the bytes (via 00d833a6 and the source's slot +4 or slot +0x10), asserts that the … |
| `00f9fe40` | BinXmlReadU64BE | confirmed | Fastcall reader: asserts eight bytes or fewer remain, throws the BadStream exception otherwise, reads eight bytes, and returns them byte-swapped through 00f9f3… |
| `0101a450` | UnSerializeBufferXml | confirmed | Stores the byte argument at this+2 and allocates a 0x10048-byte block (FUN_00d833a6) whose header dwords are initialised, storing it at this+4. Grows the strin… |
| `0101ac50` | XmlStackBuffer_Ctor | confirmed | Constructor (thiscall): clears this+0, this+0xc, this+0x10, this+0x14, stores a List<SerializeXml::SerializeStackElement,32> vftable at this+8, calls 010169c0 … |
| `013b74f0` | EmitMatchesList | confirmed | Writes a 'Matches' element; for each match record in the range [arg+0x2c, arg+0x30] (record pointer = pbVar1) it emits a 'MatchEntry' with string attributes op… |
| `013b88e0` | EmitPlayerInfo | confirmed | Writes one Player element for the record at arg (param_2). AccountID = dword at the record (arg). GroupID = constant 0. IsBuddy = 012ce190 of the id after 012c… |
| `004c574a` | StreamWrite8Bytes | likely | Thiscall (this in ECX, source pointer param_2 to an 8-byte value): copies the 8-byte value to a local and calls the stream object at this+0x14 through its vtab… |
| `004c8488` | WriteVarUint6 | likely | Cdecl-shaped, two stack args (stream, value); ECX is passed through unused. Encodes value as one header byte holding value&0x3f, with bits 6-7 = number of foll… |
| `004c84c8` | ReadVarUint6 | likely | Cdecl, two stack args (stream, out*); ECX unused; reads one byte through stream->vtable[0x18](buf,1), takes the count from bits 6-7 and reads that many more by… |
| `004c853a` | WriteVarUintPrefixN | likely | Cdecl, four stack args (stream, value, n, flag); ECX unused. Writes a header byte: (value & mask[n]) \| (flag << (6-n)) \| (count << 6), where mask[n] is a dwo… |
| `004c85a0` | ReadVarUintPrefixN | likely | Cdecl, four stack args (stream, out value*, n, out flag*); ECX unused. Reads a header byte and the count bytes it announces through stream->vtable[0x18]. Store… |
| `004c8653` | WriteVarUint6Plus2 | likely | Cdecl, two stack args (stream, value). Calls 004c8488(stream, value+2): writes value+2 with the 6-bit variable-length encoding. Returns nothing. |
| `004c8667` | ReadVarUint6Minus2 | likely | Cdecl-shaped, two stack args (stream, out*). Calls 004c84c8(stream, &local) to read a 6-bit variable-length unsigned integer, and on success stores local-2 int… |
| `004c8690` | WriteVarUintPrefix1Flag | likely | Cdecl, three stack args (stream, value, flag). Calls 004c853a(stream, value, 1, flag) and returns nothing. The callee writes the value with a 5-bit payload and… |
| `004c86a7` | ReadVarUintPrefix1FlagBool | likely | Cdecl-shaped, three stack args (stream, value*, bool*). Calls 004c85a0(stream, value*, 1, &flag). On success stores (flag == 1) into *bool and returns 1; other… |
| `004c86d7` | BitReaderReadBits | likely | Thiscall on a bit-reader object (fields: [0] buffer pointer, [1] size, [2] bit offset in the current byte, [3] byte index), with stack args nbits and out*. Ret… |
| `004c876e` | WriteLenPrefixedBits | likely | Thiscall on a bit-buffer object (this[1] = bit count, this[0] points to the data at +4) with one stack stream arg. Writes the bit count with WriteVarUint6 (004… |
| `004ca5cc` | WriteCountedBytesVar | likely | Thiscall(this, string-holder): reads the length n at **arg; writes n with a 1-byte header when n < 0x7e, otherwise a 0x7e marker plus a 2-byte length when n < … |
| `004ca63f` | WriteUtf16BigEndianSwapped | likely | Thiscall(this, ushort* text, count) with a 2048-entry (4 KB) stack buffer: when count is negative it measures the zero-terminated text; then in chunks of at mo… |
| `008c9d20` | BoundedReader_ReadDword_B | likely | Same body as BoundedReader_ReadDword (008ca980): when fewer than 4 bytes remain before the end pointer at reader+0xc it writes 0 to *param_2, sets the byte at … |
| `008c9dc0` | BoundedReader_ReadBool | likely | When fewer than 1 byte remains before the end pointer at reader+0xc, writes 0 to *param_2, sets the byte at reader+0x10 to 1 and pins the cursor to the end. Ot… |
| `008ca050` | RecordArray0x28_ReadUntilZeroKey | likely | Sets the element count at param_1+4 to 0, then loops: reads a dword key from the bounded reader at param_2 (reader cursor +8, end +0xc, sticky flag +0x10); a z… |
| `008ca980` | BoundedReader_ReadDword | likely | Compares the end pointer at reader+0xc with the current pointer at reader+8 plus 4 bytes. When fewer than 4 bytes remain it writes 0 to *param_2, sets the byte… |
| `008d5d20` | BoundedReadI16I32x3_008d5d20 | likely | Reads a 14-byte field group from the bounded reader in arg1 (cursor +8, end +0xc, error +0x10): int16 sign-extended into this+4, then int32 into this+8, this+0… |
| `008d5dd0` | BoundedReadI16I32x3_008d5dd0 | likely | Reads a 14-byte field group from the bounded reader in arg1 (cursor +8, end +0xc, error +0x10): int16 sign-extended into this+4, then int32 into this+8, this+0… |
| `008d5e80` | BoundedReadI16I32x2_008d5e80 | likely | Reads a 10-byte field group from the bounded reader in arg1: int16 sign-extended into this+4, then two int32 into this+8 and this+0xc. The two int32 are checke… |
| `008d5ee0` | BoundedReadI16I32_008d5ee0 | likely | Reads a 6-byte field group from the bounded reader in arg1: int16 sign-extended into this+4, then int32 into this+8. A short read stores 0 in the field, sets r… |
| `008d5f40` | BoundedReadI16I32x2F32x2Bool3_008d5f40 | likely | Reads a 21-byte field group from the bounded reader in arg1: int16 (sign-extended) to this+4; int32 to this+8 and this+0xc; float32 to this+0x10 and this+0x14,… |
| `008d61c0` | BoundedReadI16I32x2_008d61c0 | likely | Reads a 10-byte field group from the bounded reader in arg1: int16 sign-extended to this+4, then int32 to this+8 and this+0xc, each with its own bounds check. … |
| `008d6250` | BoundedReadI16I32_008d6250 | likely | Reads a 6-byte field group from the bounded reader in arg1: int16 sign-extended into this+4, then int32 into this+8. A short read stores 0 in the field, sets r… |
| `008d62b0` | BoundedReadI16I32_008d62b0 | likely | Reads a 6-byte field group from the bounded reader in arg1: int16 sign-extended into this+4, then int32 into this+8. A short read stores 0 in the field, sets r… |
| `008d6310` | BoundedReadI16I8_008d6310 | likely | Reads a 3-byte field group from the bounded reader at arg1 (stack, ret 4): int16 sign-extended into this+4, then one signed byte sign-extended into this+8. A s… |
| `008d6370` | BoundedReadI16I8I32x2BoolBlob_008d6370 | likely | Reads a record from the bounded reader in arg1: calls 008d6310 for an int16 at this+4 and signed byte at this+8; reads two int32 at this+0x10 and this+0x14 wit… |
| `008d63f0` | BoundedReadI16I8I32x2I32_008d63f0 | likely | Reads a 15-byte field group from the bounded reader at arg1: calls 008d6310 for an int16 at this+4 and signed byte at this+8; reads two int32 at this+0x10 and … |
| `008d6460` | BoundedReadI16I32x2_008d6460 | likely | Reads a 10-byte field group from the bounded reader in arg1: int16 sign-extended to this+4, then int32 to this+8 and this+0xc, each with its own bounds check. … |
| `008d64f0` | BoundedReadI16F32I32x5_008d64f0 | likely | Reads a 26-byte field group from the bounded reader in arg1: int16 sign-extended to this+4; two int32 to this+0x10 and this+0x14 under one 8-byte bounds check;… |
| `008d6600` | BoundedReadI16Bool2_008d6600 | likely | Reads a 4-byte field group from the bounded reader in arg1: int16 sign-extended to this+4, then two bytes each stored as (byte != 0) at this+8 and this+9. A sh… |
| `008d67a0` | BoundedReadI16x2_008d67a0 🌐 | likely | Reads two int16 from the bounded reader in arg1, each sign-extended: the first into this+4 and the second into this+8. A short read stores 0 in the field, sets… |
| `008d6890` | BoundedReadI16x2_I32_Pair_008d6890 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads one int32 into this+0x18, then two int32 into this+0x10 and this+0x14 und… |
| `008d6900` | BoundedReadI16x2_Pair_F32_008d6900 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d6980` | BoundedReadI16x2_Pair_I32_008d6980 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d69f0` | BoundedReadI16x3_Pair_008d69f0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads one int16 sign-extended into this+0xc, then two int32 into this+0x10 and … |
| `008d6a60` | BoundedReadI16x3_Pair_Bool_I32x4_008d6a60 | likely | Calls 008d69f0 on the reader (three int16 and a pair of int32 at this+0x10/0x14), then reads one byte as a bool into this+0x18 and four int32 into this+0x1c, t… |
| `008d6b40` | BoundedReadI16x2_Pair_F32_008d6b40 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d6bc0` | BoundedReadI16x2_Pair_I32x2_008d6bc0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d6c50` | BoundedReadI16x2_Pair2_Bool_I32x3_Bool_008d6c50 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 and two int32 into this+0x18 and t… |
| `008d6d80` | BoundedReadI16x2_Pair2_Bool_I32x6_Bool_008d6d80 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 and two int32 into this+0x18 and t… |
| `008d6f10` | BoundedReadI16x2_Pair_I32x3_008d6f10 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d6fd0` | BoundedReadI16x2_Pair_I32x3_008d6fd0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d7090` | BoundedReadI16x2_Pair_I32x2_008d7090 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d7120` | BoundedReadI16x2_Pair_I32x2_008d7120 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d71b0` | BoundedReadI16x2_Pair_F32_008d71b0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d7230` | BoundedReadI16x2_Pair_I32_008d7230 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d72a0` | BoundedReadI16x2_Pair_I32_Pair_008d72a0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d7340` | BoundedReadI16x2_Pair_I32_008d7340 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte bounds check, the… |
| `008d73b0` | BoundedReadI16x2_Pair_I32x2_F32x3_Bool3_008d73b0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then int32… |
| `008d75d0` | BoundedReadI16x2_Pair2_008d75d0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then two i… |
| `008d7650` | BoundedReadI16x2_Pair_I32_008d7650 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one i… |
| `008d76c0` | BoundedReadI16x2_Pair_I32_008d76c0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one i… |
| `008d7730` | BoundedReadI16x2_Pair_I32_008d7730 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one i… |
| `008d77a0` | BoundedReadI16x2_Pair_I32_008d77a0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one i… |
| `008d7810` | BoundedReadI16x2_Pair_Bool_008d7810 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one b… |
| `008d7880` | BoundedReadI16x2_Pair_I32x2_Bool_008d7880 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then int32… |
| `008d7980` | BoundedReadI16x2_I32_Blob_008d7980 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), reads one int32 into this+0x14, then calls ReadBoundedLengthBlob (008bfc00) with ECX… |
| `008d7a60` | BoundedReadI16x2_I32_Pair_008d7a60 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then one int32 into this+0x18, then two int32 into this+0x10 and this+0x14 under one… |
| `008d7ad0` | BoundedReadI16x2_Pair_I32x2_Pair2_008d7ad0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then two int32 into this+0x10 and this+0x14 under one 8-byte check, then two int32 i… |
| `008d7bc0` | BoundedReadI16x2_Pair_I32_008d7bc0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one i… |
| `008d7c70` | BoundedReadI16x2_Pair_I32x2_008d7c70 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then two int32 into this+0x10 and this+0x14 under one 8-byte check, then two int32 i… |
| `008d7d00` | BoundedReadI16x2_Pair_Bool_008d7d00 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one b… |
| `008d7d70` | BoundedReadI16x2_Pair_VirtSub_Bool_008d7d70 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then calls… |
| `008d7e40` | BoundedReadI16x2_Pair_I32x3_008d7e40 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then three… |
| `008d7f00` | BoundedReadI16x2_Pair2_008d7f00 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then two i… |
| `008d7f80` | BoundedReadI16x2_Pair2_I32x6_008d7f80 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then two int32 pairs into this+0x10/0x14 and this+0x18/0x1c (each under one 8-byte c… |
| `008d80d0` | BoundedReadI16x2_Pair_Bool_008d80d0 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one b… |
| `008d8140` | BoundedReadI16x2_I32_Pair_008d8140 | likely | Calls 008d6830 on the reader (int16 sign-extended into this+4 and this+8), then one int32 into this+0xc, then two int32 into this+0x10 and this+0x14 under one … |
| `008d81b0` | BoundedReadI16x2_008d81b0 | likely | Reads two int16 from the bounded reader in arg1, each sign-extended: the first into this+4 and the second into this+8. A short read stores 0 in the field, sets… |
| `008d8210` | BoundedReadI16x2_I32x2_008d8210 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0xc and this+0x10, each with its own bounds check. A … |
| `008d8280` | BoundedReadI16x2_I32x2_Bool_008d8280 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0xc and this+0x10 and one byte stored as a bool at th… |
| `008d8350` | BoundedReadI16x2_I32x5_Pair_008d8350 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads five int32 into this+0xc, this+0x10, this+0x14, this+0x18 and this+0x1c, … |
| `008d8490` | BoundedReadI16x2_I32x2_008d8490 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0xc and this+0x10, each with its own bounds check. A … |
| `008d8570` | BoundedReadI16x2_I32_Pair_I32x4_Bool_008d8570 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), reads one int32 into this+0xc, then calls 008caaa0 with ECX=this+0x10 (its two u32 l… |
| `008d8670` | BoundedReadI16x2_I32x2_008d8670 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0xc and this+0x10, each with its own bounds check. A … |
| `008d86e0` | BoundedReadI16x2_I32x6_008d86e0 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads six int32 into this+0xc through this+0x20 (0xc, 0x10, 0x14, 0x18, 0x1c, 0… |
| `008d8870` | BoundedReadI16x2_I32_Blob_008d8870 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), reads one int32 into this+0xc, then calls ReadBoundedLengthBlob (008bfc00) with ECX=… |
| `008d8910` | BoundedReadI16x2_I32x2_008d8910 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0xc and this+0x10, each with its own bounds check. A … |
| `008d89c0` | BoundedReadI16x2_I32x4_008d89c0 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads four int32 into this+0xc, this+0x10, this+0x14 and this+0x18, each with i… |
| `008d8af0` | BoundedReadI16x2_I32x5_008d8af0 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads five int32 into this+0xc through this+0x1c, each with its own bounds chec… |
| `008d8bc0` | BoundedReadI16x2_I32x5_Bool_008d8bc0 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads five int32 into this+0xc through this+0x1c, each with its own bounds chec… |
| `008d8cb0` | BoundedReadI16x2_I32x2_008d8cb0 | likely | Calls 008d81b0 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0xc and this+0x10, each with its own bounds check. A … |
| `008d8d20` | BoundedReadI16x2_008d8d20 | likely | Reads two int16 from the bounded reader in arg1, each sign-extended: the first into this+4 and the second into this+8. A short read stores 0 in the field, sets… |
| `008d8d80` | BoundedReadI16I8_008d8d80 | likely | Reads a 3-byte field group from the bounded reader in arg1: int16 sign-extended into this+4, then one signed byte sign-extended into this+8. A short read in ei… |
| `008d8de0` | BoundedReadI16I8_I32x3_008d8de0 | likely | Calls 008d8d80 on the reader (int16 at this+4 and signed byte at this+8), then reads three int32 into this+0xc, this+0x10 and this+0x14, each with its own boun… |
| `008d8e70` | BoundedReadI16x2_008d8e70 | likely | Reads two int16 from the bounded reader in arg1, each sign-extended: the first into this+4 and the second into this+8. A short read stores 0 in the field, sets… |
| `008d8ed0` | BoundedReadI16x2_Pair_Bool_008d8ed0 | likely | Calls 008d8e70 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one b… |
| `008d8f40` | BoundedReadI16x2_Pair_Bool_008d8f40 | likely | Calls 008d8e70 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one b… |
| `008d8fb0` | BoundedReadI16x2_Pair_Bool_008d8fb0 | likely | Calls 008d8e70 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then one b… |
| `008d9020` | BoundedReadI16x2_Pair_I32x2_008d9020 | likely | Calls 008d8e70 on the reader (int16 sign-extended into this+4 and this+8), then reads two int32 into this+0x10 and this+0x14 under one 8-byte check, then two i… |
| `008d90b0` | BoundedReadI16I8_008d90b0 | likely | Reads a 3-byte field group from the bounded reader in arg1: int16 sign-extended into this+4, then one signed byte sign-extended into this+8. A short read in ei… |
| `008d9210` | BoundedReadI16I8_I32x2_008d9210 | likely | Calls 008d90b0 on the reader (int16 sign-extended into this+4, one signed byte into this+8), then reads two int32 into this+0xc and this+0x10, each with its ow… |
| `008d9280` | BoundedReadI16I8_I32x2_008d9280 | likely | Calls 008d90b0 on the reader (int16 sign-extended into this+4, one signed byte into this+8), then reads two int32 into this+0xc and this+0x10, each with its ow… |
| `008e2470` | ReadCountedRegionOrLength | likely | Reads a signed length word from the bounded cursor, advances it, rejects negative or out-of-range spans by setting the error flag and clamping the cursor, and … |
| `008e3710` | ReadS16PairThenBoolByte | likely | Calls 008d6830 with destination in ECX and reader on the stack to read two signed 16-bit fields into +4/+8; then reads a bounded byte into +0xC as bool. Each s… |
| `008e3760` | ReadFixedRecord_008e3760 | likely | Reads successive bounded 32-bit fields into destination offsets +4 through +0x30, two byte flags, length-driven spans, additional fields through +0xac, and set… |
| `008e3c30` | ReadRepeatedFourU32Rows | likely | Calls 00723600 to remove existing linked-list nodes. Reads a signed 32-bit row count from the bounded reader, zeroing it and setting/clamping the reader error … |
| `008e3cf0` | ReadS16PairThenBoolByte | likely | Calls 008d6830 with destination in ECX and reader on the stack to read two signed 16-bit fields into +4/+8; then reads a bounded byte into +0xC as bool. Each s… |
| `008e3d60` | ReadS16PairThenTwoU32 | likely | Calls 008d6830 with destination in ECX and reader on the stack to read two signed 16-bit fields into +4/+8, then reads two bounded u32 values into +0x10/+0x14.… |
| `008e3e70` | ReadS16PairThenU32 | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded u32 into destinat… |
| `008e3ef0` | ReadS16PairThenU32 | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded u32 into destinat… |
| `008e3fc0` | ReadS16PairThenU32 | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded u32 into destinat… |
| `008e4010` | ReadS16PairThenU32 | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded u32 into destinat… |
| `008e4060` | ReadS16PairThenU32 | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded u32 into destinat… |
| `008e40b0` | ReadS16PairThenBoolByte | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded byte into destina… |
| `008e40f0` | ReadS16PairThenBoolByte | likely | Uses destination as ECX and passes the reader on the stack to 008d81b0, which reads two signed 16-bit fields into +4/+8; then reads a bounded byte into destina… |
| `00911420` | WriteDwordClamped4 | likely | cdecl-style body with stdcall-cleaned callee: copies up to 4 bytes of the dword *arg2 into the writer arg1 at its cursor [+0x2020]: count = remaining if remain… |
| `00911c30` | WriteLenPrefixedBytes | likely | thiscall (this=writer arg1 in ECX; arg2 = source with data pointer +4 and length +8). Writes the 4-byte length [arg2+8] (clamped to the writer's remaining spac… |
| `00912790` | WriteU16FromPlus4ThenSub | likely | thiscall (this = source arg1 in ECX, writer arg2 on stack). Writes 2 bytes: the low 16 bits of the dword at [this+4], through FUN_007cc400 with count = min(2, … |
| `009268b0` | WriteListCountAndRecords | likely | Writes a list count and iterates nodes through next links at +0x14, writing two 32-bit fields from each node. |
| `0093c700` | InitBoundedReader | likely | Initializes a local bounded reader with begin/end pointers, zero failure flags, and the supplied size, then returns its initialized status byte. |
| `0093c790` | ReadRecordHeaderFields | likely | Reads a signed 16-bit and 8-bit field, then a 32-bit field through the bounded reader; marks failure on insufficient bytes. |
| `0093ca50` | DecodeBoundedRecord | likely | Reads dwords, flags, paired 64-bit values, nested data, and trailing integers into a destination while advancing or failing a bounded reader. |
| `0093f950` | ReadU32OrZero | likely | Reads a 32-bit value into destination+100 when available; otherwise stores zero, marks failure, moves the cursor to end, then calls 0092afb0. |
| `0093f9a0` | ReadU32IntoOffset70 | likely | Prepares the stream, reads a 32-bit word into object+0x70 or writes zero and marks exhaustion, then calls 0092afb0. |
| `0093fa20` | WriteHeaderWordsAndValue | likely | Combines two 16-bit header words, writes up to two bytes to the bounded output, advances its cursor, and passes offset 8 to 009268b0. |
| `0093fa90` | WriteHeaderThenFields | likely | Writes a capacity-bounded two-byte header, advances the cursor, then calls 00935830 to serialize additional fields. |
| `00941670` | ReadBytesOrFail | likely | Sets a byte-range cursor, invokes 0093f9a0 to read a 32-bit value, and returns the cursor failure byte. |
| `00953790` | StreamReadBoolDwordBool | likely | Thiscall (ECX = output record, arg = bounded byte cursor with cursor at +8, end at +0xc and overflow flag at +0x10): reads one byte as a bool into the record a… |
| `00953df0` | StreamReadWordBlobDword | likely | Thiscall (ECX = output record, arg = bounded byte cursor with cursor +8, end +0xc, overflow flag +0x10): reads a 16-bit value into record+4, then a length-pref… |
| `009548c0` | WriteDwordToBoundedBuffer | likely | Cdecl-style (arg1 = writer, arg2 = source pointer slot): computes the free bytes in the writer's 8 KB buffer (capacity at +0x2018), writes min(4, free) bytes o… |
| `009563e0` | ReadBoundedFieldsThenDelegate | likely | Checks a signed-length field against the supplied cursor bounds, marks invalid input and advances to the end on failure, otherwise consumes the field and calls… |
| `009b0660` | ReadInt32Array | likely | Reads a signed element count, marks reader error on invalid length, resizes destination, and copies each 32-bit value with bounds checks. |
| `009b0700` | ReadIntTree | likely | Clears an ordered tree, reads a count and that many integer values from the bounded reader, and inserts each value. |
| `009b1bd0` | ReadMiniGameGroupRecord | likely | Reads five dwords and flags from bounded reader, consuming counted byte sequences and setting reader error on truncation. |
| `009b1df0` | ReadMiniGameTypeRecord | likely | Reads integer fields, booleans, and length-counted payload sections from bounded reader, setting reader error when fields run out. |
| `009d4230` | ReadFloat3Checked | likely | Reads three 32-bit values from the bounded reader (cursor +8, end +0xc, error byte +0x10, NaN flag +0x11) into the destination float3 in order; on exhaustion w… |
| `009d42d0` | ReadCountedVec3Array | likely | Reads a 32-bit count from the reader (rejects negative or larger-than-remaining counts by setting the error byte), grows the destination via 009d3550 when its … |
| `009d4360` | ReadCountedFloatArray | likely | Reads a 32-bit count (rejects negative or overlong counts), grows the destination float array through 007d7250 when its capacity is below the count, sets its c… |
| `009d4e20` | ReadS16Pair | likely | Reads two signed 16-bit values from the bounded reader: the first is sign-extended into out+4, the second into out+8; on exhaustion writes 0, sets the error by… |
| `009d4e80` | ReadS16PairAndCountedBytes | likely | Reads an int16 pair via 009d4e20 into out+4 and out+8, then a signed length word; a negative or overflowing length sets the error byte, otherwise that many byt… |
| `009d53a0` | ReadKeyedIStringRecord | likely | Reads a 32-bit value into record +0, a length-prefixed byte run copied through 00894b10 into the record, then a 32-bit value into record +0x14; exhaustion writ… |
| `009d59d0` | ReadCountedVec3ArrayThiscall | likely | Reads a 32-bit count (rejects negative or overlong counts), grows the destination 12-byte array via 009d2a60 when its capacity is short, sets the count (+8), t… |
| `009d61b0` | ReadCountedDwordFloat4Records | likely | Reads a 32-bit count (rejects negative or overlong counts), grows the destination via 009d3620 when capacity is short, sets the count, then reads each record w… |
| `009d6230` | ReadCountedDwordFloat4Records50 | likely | Reads a 32-bit count (rejects negative or overlong counts), grows the destination via 009d36d0 when capacity is short, sets the count, then reads each record w… |
| `00a61ca0` | WriteTwoShortHeaderFields | likely | Forms two 16-bit values from the object's fields at +4 and +8, writes each value to the output stream subject to the two-byte remaining-capacity check, and adv… |
| `00a8d0a0` | ReadBoundedWord | likely | Advances the bounded reader, checks that four bytes remain, writes the next 32-bit word to destination+0x0c, or zeros it and marks reader error. |
| `00a8d0e0` | ReadBoundedWordVariant | likely | Advances the bounded reader, checks that four bytes remain, writes the next 32-bit word to destination+0x0c, or zeros it and marks reader error. |
| `00a8d140` | ReadBoundedWordVariant2 | likely | Advances the bounded reader, checks that four bytes remain, writes the next 32-bit word to destination+0x0c, or zeros it and marks reader error. |
| `00a8d190` | ReadBoundedByte | likely | Advances the bounded reader, checks for one remaining byte, stores that byte as an integer at destination+0x0c, or zeros it and marks reader error. |
| `00a8d1f0` | ReadMixedBoundedFields 🌐 | likely | Reads five 32-bit fields and three byte-sized fields into offsets 0x0c through 0x20; each short read zeros that destination, sets reader error, and clamps the … |
| `00a8d330` | ReadTwoBoundedWords 🌐 | likely | Reads two 32-bit words into destination offsets 0x0c and 0x10; a short read zeros the affected field, marks reader error, and clamps the cursor. |
| `00b3a6e0` | ReadBoundedStatusAndRecord 🌐 | likely | Reads two signed 16-bit values, stores the following dword at destination offset 0x18, then parses the remaining record through the bounded reader and returns … |
| `00b3aa30` | Allocate28ByteRecordAndRead | likely | Allocates a 0x28-byte record, initializes its default dwords from two global values, assigns the pointer to the output, and reads the remaining record fields f… |
| `00b3ab20` | WriteStructuredRecordToBoundedStream | likely | Writes the record's initial values, serializes its nested field and length-prefixed byte member, then writes the remaining dword fields while advancing the bou… |
| `00b3af80` | WriteThreeValuesAndNestedMembers | likely | Writes three dword values through the bounded writer, then serializes the nested members at offsets 0x14 and 0x38 as length-prefixed byte fields. |
| `00b3b080` | ReadThreeDwordsAt30 | likely | Reads three consecutive dword values from the bounded reader into destination offsets 0x30, 0x34, and 0x38, marking the reader failed and zeroing each unavaila… |
| `00b3b130` | WriteTwoValuesAndNestedMember | likely | Writes dwords from source offsets 4 and 8 to the bounded stream, then serializes the nested member at source offset 0x10. |
| `00b3b1d0` | WriteNestedValuesAndTrailingFields | likely | Serializes nested members at offsets 8 and 0x10, writes the dword at offset 0x18 and a length-prefixed member at 0x1c, then writes the first dword. |
| `00b3d570` | ReadInt16PairAnd8ByteRecords | likely | Reads two signed 16-bit fields and then parses a counted sequence of 8-byte elements into the record array at destination offset 0x18, returning success only w… |
| `00b3d5f0` | ReadInt16PairAndNestedRecord | likely | Reads two signed 16-bit values and a dword into destination offset 0x1c; then allocates and parses a nested 0x28-byte record into destination offset 0x18. |
| `00b3d810` | ReadCounted60ByteRecords 🌐 | likely | Reads and bounds-checks a record count, resizes the destination for 0x60-byte elements, and parses each counted record while the bounded reader remains valid. |
| `00b3d8a0` | ReadCounted214ByteRecords 🌐 | likely | Reads and bounds-checks a record count, resizes the destination for 0x214-byte elements, and parses each counted record while the bounded reader remains valid. |
| `00b3da50` | ReadStructuredRecordAndLists | likely | Reads successive dwords and boolean bytes into destination fields, skips bounded byte regions, reads paired dwords, and parses a counted list into the nested m… |
| `00b3dd80` | WriteTwoValuesAndCountedList | likely | Writes the two leading dwords from the source record, then serializes the counted list beginning at its nested member. |
| `00b3de40` | WriteCountedNestedRecords | likely | Writes the source count, walks linked records through their next pointer, writes each record's dword value, and serializes each record's nested value. |
| `00b3ded0` | WriteCountedNestedRecordsVariant | likely | Writes the source count, walks linked records through their next pointer, writes each record's dword value, and serializes each record through the variant nest… |
| `00b7dd10` | ParseAndValidateReaderRange | likely | Builds a reader over the supplied byte range, calls 00b7ae90 to decode one fixed record, then returns success only when the reader error byte is clear and eith… |
| `00b7ddc0` | WriteTwoHalfwords | likely | Combines two 2-byte values from the supplied argument and fields at +4/+8, copies each through 007cc400 into the writer buffer subject to remaining capacity, a… |
| `00b7fc40` | SerializeRecord_00b7fc40 🌐 | likely | Writes the two-halfword helper output, passes +0x10 to 008fb810, emits bytes +0x18/+0x19, and advances the writer cursor for copied fields. |
| `00b7ffd0` | SerializeRecord_00b7ffd0 🌐 | likely | Writes +0x0c as one byte and +0x10 as a capacity-limited dword, then passes +0x18 to 008fb810. |
| `00baaeb0` | ReadShortAndDword | likely | Reads a signed 16-bit value and then a dword from a bounded cursor; on insufficient space, stores zero, sets the error byte, and moves the cursor to its end. |
| `00baaf10` | ReadRecordFields_00baaf10 | likely | Calls the short-and-dword reader, then reads a dword, a boolean byte, and another dword into fields +0xc, +0x14, and +0x10, marking cursor error on truncation. |
| `00baafa0` | ReadRecordFields_00baafa0 | likely | Calls the short-and-dword reader, reads five dwords into fields +0xc through +0x1c and a byte into +0x20, and flags a NaN at +0x1c. |
| `00bab0b0` | ReadRecordFields_00bab0b0 | likely | Calls the short-and-dword reader, reads four dwords into fields +0xc through +0x18, and marks cursor float error if the last value is NaN. |
| `00bab170` | ReadRecordFields_00bab170 | likely | Calls the short-and-dword reader, reads dwords at +0xc, +0x10, +0x14, and +0x18, checks +0x14 for NaN, then reads a byte into +0x1c. |
| `00bab340` | ReadRecordWithinBounds_00bab340 | likely | For nonnull inputs, builds a cursor over the supplied range, calls FUN_00baaf10, and returns 1 only if cursor error is clear and the configured end condition p… |
| `00bab3b0` | ReadRecordWithinBounds_00bab3b0 | likely | Builds a bounded cursor, calls FUN_00baafa0, and returns 1 only if cursor error is clear and the configured end condition passes. |
| `00bab420` | ReadRecordWithinBounds_00bab420 | likely | Builds a bounded cursor, calls FUN_00bab0b0, and returns 1 only if cursor error is clear and the configured end condition passes. |
| `00bab490` | ReadRecordWithinBounds_00bab490 | likely | Builds a bounded cursor, calls FUN_00bab170, and returns 1 only if cursor error is clear and the configured end condition passes. |
| `00bbf540` | StreamReadDwordRecord | likely | Pseudocode (484 bytes): reads eight 32-bit values from the stream object at arg2 (cursor at +8, end at +0xc, error flag at +0x10) into this+0x10, +0x14, +0x18,… |
| `00bd0340` | WriteFieldsAndBlocks | likely | Serialises object param_1 into writer param_2: eight dword fields at +0x10 through +0x2C, each written with 007CC400 in chunks clipped to the writer's remainin… |
| `00bd31d0` | ReaderReadTwoInt16 | likely | Thiscall with a reader argument (cursor at +8, end at +0xC, error byte at +0x10). Reads two 16-bit values from the cursor, each after a bounds check of two byt… |
| `00c84f00` | ReadCountedElements | likely | Consumes a count and repeatedly advances the input reader while invoking element cleanup and append helpers. |
| `00c84f70` | ReadCountedNestedElements | likely | Consumes a count, reads each element value, invokes a nested conversion helper, and advances the reader state. |
| `00c85530` | ReadRatingEntry | likely | Reads two leading values and a nested record through the bounded reader, clearing output fields when the first read fails. |
| `00c888d0` | ReadFirstTimeEventEntryHeader | likely | Reads the 16-bit and 8-bit header fields, then a 32-bit value and a one-byte flag, marking reader error on underflow. |
| `00c89010` | WriteFirstTimeEventHeader | likely | Writes the 16-bit field at object offset 4 and the one-byte field at offset 8 to the bounded output writer. |
| `00c89150` | WriteFirstTimeEventByteField | likely | Writes the shared header fields, then writes the byte stored at object offset 0x0c and advances the output cursor. |
| `00c891b0` | WriteFirstTimeEventDwordField | likely | Writes the shared header fields, then writes the dword stored at object offset 0x0c and advances the output cursor. |
| `00c89210` | WriteFirstTimeEventDwordField | likely | Writes the shared header fields, then writes the dword stored at object offset 0x0c and advances the output cursor. |
| `0101ad20` | XmlStackBuffer_Dtor | likely | Destructor body: clears this+0x4bc and this+0x4c0, frees the buffer at this+4 (0073b890 then _free) when non-null, calls 0101a710 on the HashList at this+0x230… |
| `010451c0` | SequentialFieldParse6 | likely | Thiscall, two stack args (param_2 = buffer or cursor, param_3 = context). Zeroes dwords at this+0x18, +0x1c, +0x20, +0x24. Then calls 0072d900 on (param_2, thi… |
| `010880c0` | ReadProjectileStateBlock | likely | Thiscall with one stack arg (reader). Reads four dwords into this+0x10, +0x14, +0x18 and +0x1c from the reader cursor (reader+0x8; on exhaustion writes 0, sets… |
| `012fbe10` | SerialWritePtrVector_013ffb20 | likely | Thiscall with ECX = holder (its element element-writer object is *ECX) and one stack range pointer: writes the element count ((end-begin)>>2, 0 when begin is n… |
| `012fbec0` | SerialWritePtrVector_013ff9b0 | likely | Same loop as SerialWritePtrVector_013ffb20 (012fbe10): writes the element count through 013c2640, then calls 013ff9b0(*ECX, &element) for each 4-byte element a… |
| `012fc6a0` | SerialWriteKeyPairTree | likely | Thiscall with ECX = holder (its element writer object is *ECX) and one stack pointer to a tree-like container: writes the count at container+8 through 013c2640… |
| `012fc750` | SerialWriteKeyPairTreeB | likely | Same walk as SerialWriteKeyPairTree (012fc6a0) with per-entry writer 013ffbf0 and tree step 012af3e0: writes the count through 013c2640, writes each node's pai… |
| `012fcdc0` | SerialWriteStdStringVector | likely | Thiscall with ECX = writer holder and one stack pointer to a vector of 28-byte std::string elements: writes the element count ((end-begin)/28, 0 when begin is … |
| `01311eb0` | PropertySetLogEntries | likely | Fastcall. When the logging gate 012d09e0 is set, writes 'PropertySet <begin> 0x%p' to wa_properties.log, then for each entry of the map at this+4 writes '\t Pr… |
| `013129e0` | FactoryAllocCtor_01312930 | likely | Factory with no arguments. Allocates storage with 00d833a6 and, when the allocation is non-null, runs the constructor 01312930 on it and returns the object; re… |
| `01312a40` | RegisterFactory_0x17 | likely | Static initializer with no arguments. Registers factory 013129e0 under classID 0x17 through 013ce130 and again through 013ce230 (the two registry calls take th… |
| `0134c310` | SerialReadPolymorphicObject | likely | Cdecl reader with two stack args (arg1 = stream, passed as ECX to 013c24d0 and as the Read argument; arg2 = out pointer). Reads an integer flag through 013c24d… |
| `013587f0` | StringDwordMapWrite | likely | Thiscall (ECX = pointer to the writer pointer), one stack arg (the tree object), ret 4, under an EH frame. Writes the tree's size word (+8) through 013c2640 (S… |
| `0135d8b0` | SerialReadStlContainer5 | likely | Fastcall reader (ECX = reader object): starts the container read through 013c2690 to get the element count, then for each element zeroes a dword and a byte, re… |
| `0135d9e0` | SerialReadStlContainer5_2 | likely | Fastcall reader, byte-for-byte the same control flow as SerialReadStlContainer5: reads the count through 013c2690, then for each element reads a dword plus a f… |
| `01366010` | SerialReadStringKeyedMap | likely | Reads an element count with 013c2690 (SerialStartReadContainer), then for each element reads one string through 013ffe50, assigns it, builds a (string, 0) pair… |
| `01366210` | SerialWriteTreeElements | likely | Starts a container write with 013c2640 (count = *(param_2+8)), walks the tree at param_2 in order from its head (+4) via 0135ad00/0134f430, and for each node w… |
| `01369390` | SerialReadIntKeyContainer | likely | Reads an element count with 013c2690 (SerialStartReadContainer), then for each element reads one dword key through 01401af0; a failed read returns through 0135… |
| `013694e0` | SerialReadIntKeyRecordTree | likely | Reads an element count with 013c2690 (SerialStartReadContainer), then for each element reads a record through 01401b20 into a local (int key, int, byte and a s… |
| `01389810` | ReadStringPairVector | likely | Thiscall on a holder whose first dword is the serial reader. Reads an element count with 013c2690 and reserves the holder's 56-byte vector through 01388070. Fo… |
| `01393eb0` | ReadKeyedRecordContainer | likely | Fastcall. Reads the element count through 013c2690 (SerialStartReadContainer). For each element it reads a three-dword record through 01400050 (failure returns… |
| `013bb080` | SerialReadElems4 | likely | Thiscall reader for a container of 4-byte elements: reads the element count through 013c2690 (SerialStartReadContainer), reserves that many elements with 013ba… |
| `013bcba0` | SerialReadElems4 | likely | Same body as 013bb080: reads the element count through 013c2690, reserves through 013bc490, reads each element through 01400110 and pushes it with 013bcb30; re… |
| `013d3e10` | SerialWriteInstanceIdVector | likely | Writes a vector of 16-byte elements: the element count (end-begin)>>4 goes out through SerialStartAppendContainer (013c2640); then for each element at begin+16… |
| `013d53d0` | SerialReadInstanceIdVector | likely | Reads a count-prefixed vector of 16-byte InstanceID elements, the read counterpart of 013d3e10: SerialStartReadContainer (013c2690) reads the count into a loca… |
| `013d5ae0` | SerialReadBitVector | likely | Reads a bit vector from the serializer: SerialStartReadContainer (013c2690) reads the element count into a local; the vector is reserved to (count+31)>>5 dword… |
| `013d6b50` | SerialWriteObjectPtrVector | likely | Writes a vector of 4-byte element values (object pointers): the count (end-begin)>>2 goes out through SerialStartAppendContainer (013c2640); then for each elem… |
| `013d78e0` | SerialReadObjectPtrVector | likely | Reads a count-prefixed list of object pointers: SerialStartReadContainer (013c2690) reads the count into a local; the vector is reserved to that count through … |
| `013f2130` | SerialWriteVec4Elements | likely | Thiscall (ECX = writer record [0] = element writer context, one stack argument = sequence record [1] = begin, [2] = end): computes the element count (0 when be… |
| `013fa2e0` | SerialWriteMapEntries | likely | Thiscall (ECX = writer object; argument = tree object): writes the container header with the tree size at this+8 through 013c2640, then walks the tree in order… |
| `013fb0e0` | SerialReadMapElements | likely | Reads a container count through 013c2690 into a local. For each element it reads an 8-byte pair with 013ffdb0 (returns 0 on failure), then inserts the pair int… |
| `013fbce0` | SerialWriteMapElements | likely | Fastcall-style writer over the family-C int-keyed map (node key at +0xc, 16-byte value at +0x10). Calls 013c2640 to write the count, then walks the tree in ord… |
| `013ff9b0` | SerialWriteInt32Ptr | likely | Takes a pointer to a dword in arg2, loads the dword and calls 013c23f0 with ECX=arg1 (the serialization buffer) and that value as one stack argument. Returns. |
| `013ff9d0` | SerialWriteInt32Pair | likely | Writes the dword at *arg2 with 013c23f0; if that returns nonzero it also writes the dword at arg2+4 with 013c23f0. Returns void. |
| `013ffa30` | SerialWriteIntThenString | likely | Writes the dword at *arg2 with 013c23f0; if that returns nonzero, calls 013c2860 with arg2+4 (a string object) and returns. |
| `013ffa60` | SerialWriteIntThenByte | likely | Writes the dword at *arg2 with 013c23f0; if that returns nonzero, writes the byte at arg2+4 zero-extended, through 013c23f0 again. |
| `013ffa90` | SerialWriteStringThenInt | likely | Calls 013c2860 with arg2 (a string object); if that returns nonzero, writes the dword at arg2+0x1c with 013c23f0. |
| `013ffac0` | SerialWriteTwoStrings | likely | Calls 013c2860 with arg2 (a string object); if that returns nonzero, calls 013c2860 with arg2+0x1c. Returns void. |
| `013ffb10` | SerialWriteString | likely | Pushes arg2 and calls 013c2860 with ECX=arg1 (the buffer). Returns. |
| `013ffb20` | SerialCallChildWriteBool | likely | Loads the object pointer stored at *arg2 into ECX, calls its vtable slot at +0x28 with arg1 pushed, and returns whether the result is nonzero. |
| `013ffb40` | SerialCallChildWriteBool | likely | Same body as 013ffb20 (folded duplicate): calls the vtable slot at +0x28 of the object at *arg2 with arg1 and returns nonzero as bool. |
| `013ffb60` | SerialCallObjectWriteBool | likely | Calls the vtable slot at +0x28 of the object at arg2 with arg1 and returns whether the result is nonzero. |
| `013ffb80` | SerialWriteIntThenEmbeddedWriteBool | likely | Writes the dword at *arg2 with 013c23f0; if that returns zero it returns false. Otherwise calls the vtable slot at +0x28 of the object embedded at arg2+4 with … |
| `013ffbf0` | SerialWriteIntThenPtrWriteBool | likely | Writes the dword at *arg2 with 013c23f0; if that returns zero it returns false. Otherwise loads the object pointer at arg2+4 and calls its vtable slot at +0x28… |
| `013ffd10` | SerialWriteFourInt32s | likely | Writes four dwords at *arg2, arg2+4, arg2+8 and arg2+12 with 013c23f0, stopping at the first failed write and returning false; returns the result of the fourth… |
| `013ffd60` | SerialWriteThreeInt32s | likely | Writes dwords at *arg2, arg2+4 and arg2+8 with 013c23f0, each guarded by the previous result; returns the third write's result, or 0 if an earlier write failed. |
| `013ffda0` | SerialReadInt32Ptr | likely | Calls 013c24d0 with ECX=arg1 (the reader) and arg2 (the destination address) pushed; returns. Reads one dword into *arg2 by delegation. |
| `013ffdb0` | SerialReadInt32Pair | likely | Reads a dword into *arg2 with 013c24d0; if that returns nonzero, reads a second dword into arg2+4 the same way. Returns void. |
| `013ffde0` | SerialReadIntThenString | likely | Reads a dword into *arg2 with 013c24d0; if that returns nonzero, calls 013c28d0 (SerialReadString) with arg2+4 and returns. |
| `013ffe10` | SerialReadIntThenBoolField | likely | Reads a dword into *arg2 with 013c24d0, then reads a second dword into a local with 013c24d0; on success stores (local != 0) as a byte at arg2+4 and returns 1,… |
| `013ffe50` | SerialReadStringThenInt | likely | Calls 013c28d0 (SerialReadString) with arg2; if that returns nonzero reads a dword into arg2+0x1c with 013c24d0. Returns void. |
| `013ffe80` | SerialReadTwoStrings | likely | Calls 013c28d0 (SerialReadString) with arg2; if that returns nonzero calls it again with arg2+0x1c. Returns void. |
| `013fff30` | SerialReadFourInt32s | likely | Reads four dwords with 013c24d0 into *arg2, arg2+4, arg2+8 and arg2+0xc, each guarded by the previous result; returns the fourth read's result, or 0 if an earl… |
| `013fffe0` | SerialReadLengthPrefixedBlob | likely | Reads a dword into *arg2 with 013c24d0 (ECX=arg1, the serialization buffer), then reads a dword length into arg2+8. It allocates length+1 bytes with 00d833a6 t… |
| `01400050` | SerialReadThreeInt32s | likely | Reads dwords with 013c24d0 into *arg2, arg2+4 and arg2+8, each guarded by the previous read; returns the third read's result, or 0 if an earlier read failed. |
| `014000b0` | SerialReadStringField | likely | Pushes arg2 and calls 013c28d0 (SerialReadString) with ECX=arg1. Returns. |
| `014000c0` | SerialReadObjectByClassId | likely | Reads one integer with 013c25b0 into a stack local (the class id); calls 013ce130 to obtain the factory registry object, calls the registry's vtable slot [0] w… |
| `01400110` | SerialReadObjectByClassId | likely | Body identical to 014000c0 (folded duplicate): reads a class id with 013c25b0 into a local, obtains the factory registry from 013ce130, creates the object thro… |
| `01400220` | SerialReadIntThenObjectAt4ByClassId | likely | Reads one dword into *arg2 with 013c24d0; if nonzero reads a class id with 013c25b0, obtains the factory registry from 013ce130, creates the object via the reg… |
| `01400b20` | SerialWriteTripleVector | likely | Writes the element count ((end-begin)/12, 0 if begin is null) with 013c2640 (SerialStartAppendContainer), then for each 12-byte element writes its three dwords… |
| `01400c10` | SerialWriteKeyedChildVector | likely | Writes the element count ((end-begin)>>3, 0 if begin is null) with 013c2640, then for each 8-byte element writes its dword key (*element) with 013c23f0; if tha… |
| `01400d80` | SerialWriteIntThenSub4 | likely | Writes the dword at *arg2 with 013c23f0; if nonzero calls 012fbec0 with arg2+4 (a subobject) and returns. Returns void. |
| `01400e60` | SerialWriteIntPairThenTripleVector | likely | Writes dwords at *arg2 and arg2+4 with 013c23f0, each guarded by the previous result; if both succeed calls 01400b20 (SerialWriteTripleVector) with arg2+8 and … |
| `01400f70` | SerialWriteIntThenSub4Via0138d7d0 | likely | Writes the dword at *arg2 with 013c23f0; if nonzero calls 0138d7d0 with arg2+4 and returns. Returns void. |
| `01400fb0` | SerialWriteIntPairList | likely | Stdcall, one stack argument (ret 4): arg1 is a list header whose +4 is the sentinel node and +8 the element count. Writes the count with 013c2640, then walks t… |
| `01401800` | SerialReadTripleVector | likely | Reads the element count with 013c2690 (SerialStartReadContainer); if positive, reserves that many 12-byte elements with 014014a0 (StlVector12Reserve), then for… |
| `014018c0` | SerialReadKeyedChildVector | likely | Reads the count with 013c2690 and reserves it with 014015c0 (StlVector8Reserve). For each element: reads an int with 013c24d0 into a local (then overwritten), … |
| `014019a0` | SerialReadIntThenSub4Via01300540 | likely | Reads a dword into *arg2 with 013c24d0; if nonzero calls 01300540 with arg2+4 and returns. Returns void. |
| `01401a70` | SerialReadIntPairThenTripleVector | likely | Reads dwords into *arg2 and arg2+4 with 013c24d0, each guarded by the previous result; if both succeed calls 01401800 (SerialReadTripleVector) with arg2+8 and … |
| `01401af0` | SerialReadIntThenSub4Via01395910 | likely | Reads a dword into *arg2 with 013c24d0; if nonzero calls 01395910 with arg2+4 and returns. Returns void. |
| `01401b20` | SerialReadIntsBoolsStringThenSub | likely | Reads, each guarded by the previous result: dword at arg2, dword at arg2+4, a dword stored as a byte bool at arg2+8, a dword stored as a byte bool at arg2+0xc,… |
| `01401c40` | SerialWriteStringPairVector | likely | Thiscall: writes the element count ((end-begin)/0x38, 0 if begin is null) with 013c2640; then for each 56-byte element copies it to a local (012a2b50) and writ… |
| `01402170` | RegisterFactory_0x151 | likely | Static initialiser: passes 0x151 and the function 01402110 to the factory registry 013ce130, then the same pair to 013ce230. Returns void. |
| `01402920` | RegisterFactory_0x68 | likely | Static initialiser: passes 0x68 and the function 014028c0 to the factory registry 013ce130, then the same pair to 013ce230. Returns void. |
| `01416850` | SerializeList16 | likely | Thiscall with one stack argument (a list header with first at +4 and last at +8). Writes the element count (n = (last-first) >> 4, or 0 when first is null) thr… |
| `01418020` | RecordListDeserialize | likely | Thiscall with one stack argument, which the loop overwrites with the key. Reads an element count with FUN_013c2690 (SerialStartReadContainer). For each element… |
| `0141aa90` | ReadPolymorphicObject | likely | Two stack arguments (the reader context and an out pointer to a held object); the stack-cleanup count is not verified from bytes. Reads a null-flag integer wit… |
| `01443f80` | SerialWriteTreeEntries | likely | Thiscall with one stack argument (a tree-shaped map record). Starts an append container with 013c2640 using the dword at +0x8 of the record (the count), then w… |
| `01473440` | ReadIntFlagPairList | likely | Calls 013c2690 (serial read container start) with &local_28 for a count, then loops count times: 013ffdb0(*param_1, &int) reads an int and a flag byte (local_2… |
| `01475500` | RegisterFactoryClassId0xF9 | likely | Passes constant 0xf9 and the address of 01475490 to 013ce130 and then again to 013ce230 (factory-registry calls) and returns. |
| `01475a80` | RegisterFactoryClassId0x14A | likely | Passes constant 0x14a and the address of 01475a10 to 013ce130 and then to 013ce230 (factory-registry calls) and returns. |
| `014830f0` | RegisterFactory0x1da | likely | Registers factory 01483090 under classID 0x1da (474) by calling 013ce130(classID, factory) and 013ce230(classID, factory). |
| `01483630` | RegisterFactory0x1d9 | likely | Registers factory 014835d0 under classID 0x1d9 (473) by calling 013ce130(classID, factory) and 013ce230(classID, factory). |
| `01484040` | RegisterFactory0x1c6 | likely | Registers factory 01483fd0 under classID 0x1c6 (454) by calling 013ce130(classID, factory) and 013ce230(classID, factory). |
| `01484920` | SerializeCountedVector4 | likely | Writes a counted list of 4-byte elements: the count (end minus begin, shifted right by 2, or 0 when begin is null) through SerialStartAppendContainer 013c2640,… |
| `014857d0` | DeserializeCountedVector4 | likely | Reads a counted list of 4-byte elements: the count through SerialStartReadContainer 013c2690, reserves that count (01485270), then for each element calls the e… |
| `014858b0` | RegisterFactory0x1c5 | likely | Registers factory 01485850 under classID 0x1c5 (453) by calling 013ce130(classID, factory) and 013ce230(classID, factory). |
| `01486370` | SerializeTreePairs8 | likely | Writes an in-order map of 8-byte key/value pairs: the element count (the map size word at +8) through SerialStartAppendContainer 013c2640, then walks from the … |
| `014871c0` | DeserializeTreePairs8 | likely | Reads a counted list of 8-byte pairs: the count through SerialStartReadContainer 013c2690, then for each element reads a pair through the element reader 014002… |
| `01487ea0` | RegisterFactory0x167 | likely | Registers factory 01487e30 under classID 0x167 (359) by calling 013ce130(classID, factory) and 013ce230(classID, factory). |
| `014880f0` | RegisterFactory0x17c | likely | Registers factory 01488090 under classID 0x17c (380) by calling 013ce130(classID, factory) and 013ce230(classID, factory). |
| `01493eb0` | PropKeyValueMapWrite | likely | Thiscall with the serializer object dereferenced from ECX. Starts a serial container append through 013c2640 on the container state at +8, walks the list at co… |
| `014a4360` | DwordVector_Serialize | likely | Computes the DWORD count as (end-begin)/4 of the vector at its param+4/+8 (0 when begin is null), calls 013c2640 with that count, writes each DWORD through 013… |
| `014a4a40` | Uint32Vector_Deserialize | likely | Reads a count through 013c2690 (SerialStartReadContainer), calls 014a4810 to reserve that many DWORDs, then loops count times reading each element through 0140… |
| `014a63e0` | DwordKeyTree_Serialize | likely | Writes the entry count at param_2+8 through 013c2640, then walks the tree whose head is at param_2+4 from its first node. For each node it passes the DWORD key… |
| `014a6720` | DwordKeyTree_Deserialize | likely | Reads an element count through 013c2690, then for each element reads a DWORD key and a 32-bit flag through 013ffdb0 and inserts the key with the boolean (flag … |
| `014cb450` | SerialReadVector4_ViaReader013ffda0 | likely | Calls 013c2690 (SerialStartReadContainer) for the element count, reserves that many slots via 014ca9a0, then for each element calls 013ffda0(reader, &value) an… |
| `014cfbd0` | SerialWriteObjectPtrVector | likely | Computes the element count of the vector argument (+0x4/+0x8 of the container), calls 013c2640 (SerialStartAppendContainer) with it, then for each element poin… |

### `engine.service`

**Engine::AccountService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ecbb0` | Engine::AccountService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013b9890 first, then stores Engine::AccountService::vftable (018d6044) at this. |
| `012ecc10` | Engine::AccountService::DtorBody | confirmed | Destructor body: stores Engine::AccountService::vftable (018d6044) at this and calls the base destructor 013b9640. |
| `012ecc60` | Engine::AccountService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the AccountService vftable (018d6044), calls the base destructor 013b9640, and frees this with _free when bit 0 of the flag … |

**Engine::ChatService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012eccc0` | Engine::ChatService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013dbca0, then stores Engine::ChatService::vftable (018d6054) at this. |
| `012ecd20` | Engine::ChatService::DtorBody | confirmed | Destructor body: stores Engine::ChatService::vftable (018d6054) at this and calls the base destructor 013dbc40. |
| `012ecd70` | Engine::ChatService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the ChatService vftable (018d6054), calls the base destructor 013dbc40, and frees this when bit 0 of the flag is set. Virtua… |

**Engine::CollectionService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ecdd0` | Engine::CollectionService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013dc850, then stores Engine::CollectionService::vftable (018d6068) at this. |
| `012ece30` | Engine::CollectionService::DtorBody | confirmed | Destructor body: stores Engine::CollectionService::vftable (018d6068) at this and calls the base destructor 013dc920. |
| `012ece80` | Engine::CollectionService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the CollectionService vftable (018d6068), calls the base destructor 013dc920, and frees this when bit 0 of the flag is set. … |

**Engine::DeckService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ecee0` | Engine::DeckService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013a9f20, then stores Engine::DeckService::vftable (018d609c) at this. |
| `012ecf80` | Engine::DeckService::DtorBody | confirmed | Destructor body: stores Engine::DeckService::vftable (018d609c) at this and calls the base destructor 013aa090. |
| `012ecfd0` | Engine::DeckService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the DeckService vftable (018d609c), calls the base destructor 013aa090, and frees this when bit 0 of the flag is set. Virtua… |

**Engine::GameService** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ed040` | Engine::GameService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013217f0, then stores Engine::GameService::vftable (018d614c) at this. |
| `012ed0a0` | Engine::GameService::DtorBody | confirmed | Destructor body: stores Engine::GameService::vftable (018d614c) at this and calls the base destructor 01321680. |
| `012ed0f0` | Engine::GameService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the GameService vftable (018d614c), calls the base destructor 01321680, and frees this when bit 0 of the flag is set. Virtua… |
| `0131df50` | Engine::GameService::UpdateExitToMainMenuTimer | confirmed | Thiscall with one float stack arg (frame time). When this+0xc is non-null and the virtual +0x34 of the object at this+0xc0 returns true, it adds the float to t… |

**Engine::GameServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0131c920` | Engine::GameServiceBase::ScalarDeletingDtor | confirmed | Thiscall-style with one byte flag argument, returns this. Calls the destructor body 0131c7a0, then calls _free on the object when flag bit 0 is set (test byte,… |
| `0131c7a0` | Engine::GameServiceBase::DtorBody | likely | Fastcall (ECX=service object). Stores Engine::GameServiceBase::vftable into the object, frees the buffer at +0x24 when it is non-null and zeroes the three dwor… |
| `0131c8a0` | Engine::GameServiceBase::GameServiceBase | likely | Fastcall (ECX=new object), returns this. Runs 01230d10 (base setup), stores Engine::GameServiceBase::vftable at +0, sets +8 and +0x10 to -1 and +0xc to 0, init… |

**Engine::LobbyService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ed150` | Engine::LobbyService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013dedb0, then stores Engine::LobbyService::vftable (018d61a4) at this. |
| `012ed1b0` | Engine::LobbyService::DtorBody | confirmed | Destructor body: stores Engine::LobbyService::vftable (018d61a4) at this and calls the base destructor 013dee20. |
| `012ed200` | Engine::LobbyService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the LobbyService vftable (018d61a4), calls the base destructor 013dee20, and frees this when bit 0 of the flag is set. Virtu… |

**Engine::NetworkService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ed260` | Engine::NetworkService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013df1e0, then stores Engine::NetworkService::vftable (018d61cc) at this. |
| `012ed2c0` | Engine::NetworkService::DtorBody | confirmed | Destructor body: stores Engine::NetworkService::vftable (018d61cc) at this and calls the base destructor 013df240. |
| `012ed310` | Engine::NetworkService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the NetworkService vftable (018d61cc), calls the base destructor 013df240, and frees this when bit 0 of the flag is set. Vir… |

**Engine::TournamentService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ed370` | Engine::TournamentService::Ctor | confirmed | Fastcall constructor (returns this): calls the base constructor 013df660, then stores Engine::TournamentService::vftable (018d61ec) at this. |
| `012ed3d0` | Engine::TournamentService::DtorBody | confirmed | Destructor body: stores Engine::TournamentService::vftable (018d61ec) at this and calls the base destructor 013df6c0. |
| `012ed420` | Engine::TournamentService::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the TournamentService vftable (018d61ec), calls the base destructor 013df6c0, and frees this when bit 0 of the flag is set. … |

**Service::ServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01230d10` | Service::ServiceBase::ServiceBase | confirmed | Stores the Service::ServiceBase vftable at +0 and stores the calling thread id from GetCurrentThreadId at +4, then returns this. |
| `01230d70` | Service::ServiceBase::~ServiceBase | confirmed | Scalar deleting destructor: stores the Service::ServiceBase vftable at +0 and calls _free on this when bit 0 of the argument is set. |
| `01230d30` | Service::ServiceBase::DestructBody | likely | Stores the Service::ServiceBase vftable at +0 and returns. It frees nothing. |

**Service::ServiceManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e0a40` | Service::ServiceManager::DtorBody | confirmed | Stores Service::ServiceManager::vftable into the object, frees the buffer at +0x14 (vector begin) when non-null, zeroes +0x14, +0x18 and +0x1c, then destroys t… |
| `012e0b20` | Service::ServiceManager::ServiceManager | confirmed | Constructor: stores Service::ServiceManager::vftable, initialises the tree proxy at this+4 through 012e0ab0, zeroes the vector words at +0x14, +0x18 and +0x1c,… |
| `012e0b80` | Service::ServiceManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor of Service::ServiceManager (vtable slot 0): calls 012e0a40 on the object, then frees it with _free when the flag argument's bit 0 is… |
| `012e0ba0` | Service::ServiceManager::GetOrCreate | likely | Lazy singleton accessor: when DAT_01cbd72c is null, allocates 0x20 bytes with operator new and constructs the object through 012e0b20, storing the result in DA… |

### `engine.statemachine`

**StateMachine** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013f60c0` | StateMachine::TypeName | confirmed | Returns a pointer to the string literal StateMachine (6-byte body: mov eax, imm; ret). |
| `013f6300` | StateMachine::StateMachine | confirmed | Constructor (ECX = this, SEH frame): stores PersistentBase::vftable, then PersistentComponent::vftable, then StateMachine::vftable; zeroes the vector and strin… |
| `013f64b0` | StateMachine::ScalarDeletingDtor | confirmed | Scalar deleting destructor (virtual slot 0 of StateMachine::vftable): calls the teardown 013f5c50, then frees this with _free when bit 0 of the flag argument i… |
| `013f60d0` | StateMachine::GetStaticDword | likely | Returns the global dword DAT_01cbe098 (6-byte body: mov eax, [addr]; ret). |
| `013f60e0` | StateMachine::SetStaticDword | likely | Stores its single stack argument into the global dword DAT_01cbe098 (12-byte body). |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013f64d0` | StateMachineDumpStack | confirmed | Debug dump of the state machine (fastcall, ECX = this; /GS cookie). Copies the state vector at this+8 into a local vector via 013f5410, pushes the current stat… |
| `013f6100` | StateMachineCopyStackPush | likely | Stdcall-style (ret 4; ECX = this). Copy-assigns this object's state vector (the vector object at this+8) into the stack argument, which is itself a vector obje… |
| `013f6130` | StateMachinePushState | likely | Pushes a state onto the state vector at this+8 via 013f5a40, after asserting the state argument is non-null (StateMachine.cpp line 0x22d). It calls the new sta… |
| `013f6200` | StateMachineSetEnvironment | likely | Stdcall-style (ret 4; ECX = this). Asserts the environment argument is non-null (line 0x30c), pushes it onto the vector at this+0x30 via 013f5ab0, stores it at… |
| `013f7a80` | StateMachineFactoryCreate | likely | Factory create: calls operator new with size 0x1bc (444 bytes); when the allocation is non-null it constructs a StateMachine through 013f6300 and returns the o… |
| `013f7af0` | StateMachineFactoryRegister | likely | Registers the StateMachine factory (013f7a80) under class id 0xe9 by calling 013ce130 and then 013ce230 with the same id and factory pointer. |

### `engine.steam`

**CCallback<ClientSteamImpl,GameOverlayActivated_t,0>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae1370` | OverlayCallbackDtor | confirmed | Installs the overlay callback vtable and unregisters this callback when flag bit 0 at object+4 is set. |
| `00ae1390` | OverlayCallbackDispatch | confirmed | Loads callback function pointer from object+0x10 and tail-jumps to it with the first argument as the event pointer. |
| `00ae13a0` | OverlayCallbackInvoke | confirmed | Loads callback pointer from object+0x10, passes its argument, and returns after the indirect call. |

**CCallback<ClientSteamImpl,MicroTxnAuthorizationResponse_t,0>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae13d0` | MicroTxnCallbackDtor | confirmed | Installs the microtransaction callback vtable and unregisters this callback when flag bit 0 at object+4 is set. |
| `00ae13f0` | MicroTxnCallbackDispatch | confirmed | Loads callback function pointer from object+0x10 and tail-jumps to it for event dispatch. |
| `00ae1400` | MicroTxnCallbackInvoke | confirmed | Loads the callback pointer from object+0x10, passes its event argument, and returns after the indirect invocation. |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae1780` | SteamClientCtor | confirmed | Stores enable flags, initializes overlay callback fields, registers Steam callback id 0x14b, initializes microtransaction callback data, registers callback 0x9… |
| `00ae12d0` | SteamClient::GetUserIdFromOwner | likely | Loads the owning object through the first pointer, checks its initialized byte at +5, obtains SteamUser, and returns the 64-bit interface value or zero. |
| `00ae1860` | CreateSteamClient | likely | Allocates 0x30 bytes through the allocator, initializes the Steam client object with the supplied enable value, stores the returned pointer in the output slot,… |

### `engine.ui`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a81110` | ValueCellAssignPtr_00a81110 | likely | Second copy of the 31-byte value-cell assignment helper (identical machine code to 00a7bfc0 and 009969b0): assigns the value pointed to by param_2 into the des… |

### `engine.valuedata`

**ValueData** (28)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127a480` | ValueData::GetClassName | confirmed | Returns the constant C string ValueData (6 bytes: mov eax, imm32; ret). Virtual slot [11] of ValueData::vftable. |
| `0127a4c0` | ValueData::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls FUN_01300cd0 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. Returns with ret 4 (one st… |
| `012fa750` | ValueData::GetExpressionTreePayload | confirmed | Fastcall with ECX = value entry: returns the dword at +8 when the type tag at +4 is 5; otherwise calls 012f5a60 with the assertion 'mTypeID == kExpressionTreeT… |
| `012fa780` | ValueData::GetIntValueMapPayload | confirmed | Fastcall with ECX = value entry: when the tag at +4 is not 0xe calls 012f5a60 with 'mTypeID == kIntValueMapTypeID' (ValueData.h line 0x141) and returns 0 if st… |
| `012fa7b0` | ValueData::GetPropertySetPayload | confirmed | Fastcall with ECX = value entry: returns the dword at +8 when the type tag at +4 is 9; otherwise calls 012f5a60 with 'mTypeID == kPropertySetTypeID' (ValueData… |
| `012fa7e0` | ValueData::GetMessagePayload | confirmed | Fastcall with ECX = value entry: returns the dword at +8 when the type tag at +4 is 10; otherwise calls 012f5a60 with 'mTypeID == kMessageTypeID' (ValueData.h … |
| `012fa810` | ValueData::GetInstanceIDPayload | confirmed | Fastcall with ECX = value entry: returns the dword at +8 when the type tag at +4 is 0xb; otherwise calls 012f5a60 with 'mTypeID == kInstanceIDTypeID' (ValueDat… |
| `012fa840` | ValueData::GetInstanceIDListPayload | confirmed | Fastcall with ECX = value entry: returns the dword at +8 when the type tag at +4 is 0xc; otherwise calls 012f5a60 with 'mTypeID == kInstanceIDListTypeID' (Valu… |
| `012fa910` | ValueData::Ctor | confirmed | Fastcall with ECX = object: stores PersistentBase::vftable, PersistentComponent::vftable and finally ValueData::vftable into *this; clears the dword at +8, set… |
| `012fa980` | ValueData::CtorBool | confirmed | Thiscall with ECX = object and one byte argument: stores the PersistentBase, PersistentComponent and ValueData vtables, sets the type tag at +4 to 1, clears th… |
| `012fa9f0` | ValueData::CtorInt | confirmed | Thiscall with ECX = object and one dword argument: stores the PersistentBase, PersistentComponent and ValueData vtables, sets the type tag at +4 to 2, and stor… |
| `012faaa0` | ValueData::BooleanText | confirmed | Fastcall with ECX = value entry: when the type tag at +4 is not 1 calls 012f5a60 with 'mTypeID == kBooleanTypeID' (ValueData.cpp line 0x3ec); returns the C str… |
| `012fb500` | ValueData::GetIntValueMapField8 | confirmed | Fastcall with ECX = value entry: when the tag at +4 is not 0xe asserts 'this->isIntValueMapValue()' (ValueData.cpp line 0x446) and 'mTypeID == kIntValueMapType… |
| `012fb560` | ValueData::IntegerListCount | confirmed | Fastcall with ECX = value entry: when the tag at +4 is not 6 asserts 'this->isIntegerListValue()' (ValueData.cpp line 0x488) and 'mTypeID == kIntegerListTypeID… |
| `012fd060` | ValueData::ConvertComponentToData | confirmed | Thiscall with ECX = value entry and one stack output/stream argument: calls the object's vtable slot +8 with the argument, then writes the type tag through 013… |
| `012fd4b0` | ValueData::CtorCString | confirmed | Thiscall with ECX = new object and one C-string argument: stores PersistentBase, PersistentComponent and ValueData vtables, sets the type tag at +4 to 3, clear… |
| `012fd580` | ValueData::CtorString | confirmed | Thiscall with ECX = new object and one std::string pointer argument: stores PersistentBase, PersistentComponent and ValueData vtables, sets the type tag at +4 … |
| `012fd640` | ValueData::EquivalentTo | confirmed | Thiscall with ECX = value entry A and one stack pointer to value entry B: when the tags are equal, compares by tag: 1 and 2 compare the bool/int payloads, 3 co… |
| `012fda60` | ValueData::AppendDisplayText | confirmed | Thiscall with ECX = value entry and one output string argument: appends a text form per tag to the output: 0 'invalid', 1 'boolean true/false', 2 'integer N' (… |
| `012fe190` | ValueData::ConvertToSignedString | confirmed | Thiscall with ECX = value entry and one output std::string: initialises the output empty; for tag 2 formats the integer with _sprintf as '%d' when negative and… |
| `01300cd0` | ValueData::DtorBody | confirmed | Destructor body (called from the scalar deleting destructor 0127a4c0): sets the vtable to ValueData::vftable, calls ValueData_SetType (01300680) with type 1 wh… |
| `013010e0` | ValueData::CopyFrom | confirmed | Thiscall (ECX = destination, stack arg = source ValueData). Switches on the source type id at +4: boolean (012c5100), integer (01251720), string (012c6ec0), ty… |
| `01301bc0` | ValueData::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, the serializer). Calls slot 4 (+0x10) of this with the serializer and a local, reads an integer type id with Seri… |
| `012fb3f0` | ValueData::FactoryCreate | likely | Cdecl-style with no arguments: allocates 0xc bytes through 00d833a6; when the allocation is nonnull constructs the entry with 012fa910 (ValueData::Ctor) and re… |
| `012fb470` | ValueData::ExpressionTreeInvoke | likely | Thiscall with ECX = value entry, two stack arguments (context param_2, argument param_3): returns 0 when the tag at +4 is not 5 and asserts 'mTypeID == kExpres… |
| `012fc470` | ValueData::IntValueMapNodeValueAt | likely | Thiscall with ECX = value entry and one count argument: when the tag at +4 is not 0xe asserts 'this->isIntValueMapValue()' (ValueData.cpp line 0x46a) and 'mTyp… |
| `012fe040` | ValueData::ConvertToString | likely | Thiscall with ECX = value entry and one output std::string: initialises the output (capacity 15, empty); tag 2 formats the integer with _sprintf '%d' and assig… |
| `01301840` | ValueData::CopyDeep | likely | Thiscall (ECX = destination ValueData, stack args = source ValueData and a context argument). If the source type is 8 (list): calls ValueData_SetType (01300680… |

**(no class)** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c0a60` | ValueDataMap_CollectStrings | confirmed | Like 012c0590, but for each matching ValueData entry it asserts the entry type is string (ValueData.h line 0x11d, mTypeID == kStringTypeID), takes the string v… |
| `012e3460` | ValueDataLookupBool | confirmed | __thiscall (ECX = X, saved in ESI), stack args: key string and default byte, ret 8. Builds a local ValueData (012fa910), calls the lowercase lookup 012e2c30 on… |
| `012ffbd0` | ValueData_DebugFormat | confirmed | Thiscall on a ValueData (ECX) with a trace object argument. Switches on the type id at +4 and writes one type-specific text line through 013508b0: boolean via … |
| `013049f0` | ContainsIntInListAt0xfa5 | confirmed | If this+0x2c is non-null, calls 013111d0 on the object at this+0x2c with key 0xfa5 into a local ValueData. If the tag is 6 (int list), scans the list for the i… |
| `01380ab0` | Member20_LookupInteger | confirmed | Thiscall with one stack argument (ret 4). Constructs a temporary ValueData through 012fa910, calls 013111d0 on the member at this+0x20 with (arg1, &temporary).… |
| `01380c90` | ValueSource_ReadString | confirmed | Thiscall (ECX = this) with one stack argument (ret 4, returns arg1). Builds a temporary ValueData, assigns an empty string into the output std::string, then ca… |
| `01383a30` | ChainIntListContains_Key0x0c | confirmed | One stack argument (the value; calling convention not read). Builds a temporary ValueData and fills it through 013833c0 with key 0xc; when the entry's type tag… |
| `01383ef0` | ChainLookupString | confirmed | Thiscall with the key as a stack argument (out std::string as the first stack pointer, returns it). Looks up the key with 013831f0 in the container at this+0x7… |
| `013c9fb0` | ParseValueToken | confirmed | cdecl(char **cursor, ValueData *value); no ret N. Reads one token from *cursor. A token starting with '(' is cut at the next tab (the cursor moves past the tab… |
| `012e2a50` | StrValuePairCopyCtor | likely | __thiscall (ECX = destination), one stack argument (source pair), ret 4. Sets the destination string to empty (capacity 0xf, size 0, first byte 0), assigns the… |
| `012e2ad0` | StrValuePairCopyCtor2 | likely | Byte-for-byte the same body as 012e2a50 (StrValuePairCopyCtor): empty destination string, assign whole source string via 00420ff1, ValueData copy at this+0x1c … |
| `012e2bb0` | StrValuePairCtorFromValue | likely | __thiscall (ECX = destination), stack args: source string (param_2) and a ValueData pointer (param_3), ret 8. Sets the string to empty, assigns the whole of pa… |
| `012e2c30` | ValueDataLookupLowerCopy | likely | __thiscall (ECX = object X), stack args: key string (param_2) and destination ValueData pointer (param_3), ret 8. If the destination is non-null: lower-cases t… |
| `012e2ec0` | PairCtorFromByValueArgs | likely | cdecl (plain ret), returns its first stack argument (destination pair pointer). Its second and third stack arguments are a std::string and a ValueData passed b… |
| `012fca90` | ValueDataTag2or6ContainsDword | likely | Thiscall with ECX = value entry and one dword argument: when the tag at +4 is 2 returns whether the payload dword at +8 equals the argument; when the tag is 6 … |
| `01300680` | ValueData_SetType | likely | Thiscall (ECX = ValueData) with the new type id as stack arg. Returns at once when the type tag at +4 already equals the argument. Otherwise destroys the curre… |
| `01306370` | IntIntMapLookup_0xfed | likely | Thiscall with one stack argument (the key). Returns 0 unless 01302310 returns true. Fetches ValueData key 0xfed through the object's vtable slot +0x3c (slot 15… |
| `013064c0` | IntIntMapLookup_0xfee | likely | Thiscall with one stack argument (the key). Same body as 01306370 except the ValueData key is 0xfee: returns 0 unless 01302310 returns true; fetches key 0xfee … |
| `01308810` | IntListCopyKey0xfbd | likely | Thiscall with one stack argument (returned as the result). If the holder at this+0x2c is non-null, fetches ValueData key 0xfbd through 013111d0 (ECX = this+0x2… |
| `013131f0` | EhCleanupValueDataAt4 | likely | Fastcall cleanup funclet for an unwind path. Stores the object in its frame and calls 01300cd0 (ValueData::DtorBody, accepted label in another shard) on the me… |
| `01320840` | TryGetIntegerListValue | likely | Calls vtable slot +4 of its first argument with key param_2 and a local out-pointer. On success it asserts mTypeID == kIntegerListTypeID (ValueData.h line 0x13… |
| `01365910` | IntValueMapSetEntry | likely | Asserts that the argument is non-null (assert text 'value', Game.cpp line 0x250d) and that the object's type word at +0x344 is 0xe (assert 'mTypeID == kIntValu… |
| `01380c10` | Member20_GetValue | likely | Thiscall with two stack arguments (ret 8, returns arg1). Constructs a ValueData in place at arg1 through 012fa910 (ECX = arg1), then calls 013111d0 on the memb… |
| `01380de0` | ValueSource_FormatKey276 | likely | Thiscall with one stack argument (out std::string, ret 4). Constructs a temporary ValueData through 012fa910, calls virtual slot +4 of the provider at this+0x1… |
| `01438e70` | FilterUpsertIntegerListValue | likely | Thiscall with four stack arguments, a flag byte and a target-list flag. Selects the list at this+0x14 or this+4. When the first flag is non-zero it looks up a … |
| `01439170` | FilterUpsertStringListValue | likely | Thiscall with four stack arguments and two flag bytes. Selects the list at this+0x14 or this+4. When the first flag is set it looks up a matching entry with 01… |
| `014f5e50` | ValueListAddType2 | likely | Thiscall with two stack arguments (ret 8). Builds a local ValueData (012fa910), calls 01300680 on it with argument 2, stores param_2 at the local's +8 field, c… |

### `platform`

**SceaSharedUtil::CrashReporterHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089c4a0` | SceaSharedUtil::CrashReporterHandler::~CrashReporterHandler | confirmed | Trivial destructor body: writes the SceaSharedUtil::CrashReporterHandler vtable pointer at +0 and returns; reached only from EH unwind funclets. |
| `0089c4b0` | SceaSharedUtil::CrashReporterHandlerScalarDtor | confirmed | Scalar deleting destructor of the crash-reporter handler: stores the class vtable at +0 and calls _free when bit 0 of param_2 is set. |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0097a710` | PlatformStringMappingsLoad | likely | Frees the object at this+0x14 (two child references and a vtable slot 3 call) and zeroes +0x14 and +0x18; then copies the path ./Resources/PlatformStringMappin… |
| `0097adb0` | PlatformStringMapOwnerCtor | likely | Stores param_3 at this+8 and param_2 at +0xc, clears the byte at +4, installs the SoeUtil::Map<int,int,-1,0> vtable at this+0x10 and zeroes +0x14 and +0x18. Ca… |

### `platform.time`

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d81b20` | QpcRead64 | likely | Calls QueryPerformanceCounter on a local 8-byte value and returns that 64-bit count in EDX:EAX. If the call fails it executes int3. |
| `00d81b40` | QpcTicksToNanoseconds | likely | Calls QueryPerformanceFrequency on a local value (int3 on failure), then computes the stack argument (64-bit tick count) times the double at 0x01844bc8 (1e9) d… |
| `00d81bd0` | UnixNsNow | likely | Gets the system time as a FILETIME, subtracts the 1601-to-1970 offset (116444736000000000 in 100 ns units, done as an add of 0xfe624e212ac18000 with carry), an… |
| `00d81c80` | UnixNs_ToGmTm | likely | Divides a 64-bit nanosecond time (two stack arguments) by 1000000000 with __aulldiv, then fills the tm structure in the third argument with __gmtime64_s. Retur… |
| `010624c0` | AccumulateElapsedTicks | likely | Fastcall with the object in ECX (no stack arguments). Compares the 64-bit pair stored at +0x60/+0x64 with the global pair DAT_01cb72a8/DAT_01cb72ac. When they … |
| `01062510` | IncrementCounterAndSampleTick | likely | Fastcall with the object in ECX (no stack arguments). Increments the word at +0x6c, reads the current wall-clock value through 00702670, and stores its two dwo… |
| `013c5d50` | GetClockSecondsMillis | likely | Calls __ftime64_s into a local time structure, then stores the low 32-bit seconds word at param_1[0] and the millisecond field (ushort) as a word at param_1[1]. |

### `platform.udp`

**UdpLibrary::UdpPlatformDriver** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010da9f0` | UdpPlatformDriver::Ctor | confirmed | Stores UdpLibrary::UdpPlatformDriver::vftable at offset 0, allocates an 8-byte block through FUN_00d833a6(8) and stores it at this[1], writes 0xffffffff and 0x… |
| `010daaa0` | UdpPlatformDriver::DeleteVirtualSlot0 | confirmed | Calls FUN_010da120 (a function that also references UdpPlatformDriver::vftable), then calls _free on this when bit 0 of the stack byte argument is set. Returns… |
| `010daac0` | UdpPlatformDriver::TimeAndSeq | likely | On first use creates two TLS slots (DAT_01cb9808, DAT_01cb9804) with _atexit cleanup and calls timeBeginPeriod(1) once. Returns EDX:EAX where EAX = timeGetTime… |

**UdpLibrary::UdpPlatformThreadObject** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010daba0` | UdpPlatformThreadObject::Dtor | confirmed | Sets the vtable to UdpLibrary::UdpPlatformThreadObject::vftable. If the DWORD at *this[4] is non-zero it calls CloseHandle on it and zeroes it. Then calls _fre… |
| `010dac20` | UdpPlatformThreadObject::DeleteVirtualSlot4 | confirmed | Calls the destructor body 010daba0, then calls _free on this when bit 0 of the stack byte argument is set. Returns this with ret 4. |
| `010dac40` | UdpPlatformThreadObject::Ctor | confirmed | Calls base constructor 010da7b0 (UdpGuardedRefCount ctor, a shard S0596 target) on this, stores UdpLibrary::UdpPlatformThreadObject::vftable, allocates 8 bytes… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010db3a0` | UdpSleepMs | likely | On first call constructs the static UdpPlatformDriver at 01cb981c through 010da9f0 and registers an atexit cleanup (01748480). Then calls Sleep(param_1) throug… |

### `platform.win32`

**GuiApp** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c4d9b` | GuiApp::SleepSeconds | confirmed | Virtual method at GuiApp::vftable slot 75: calls Sleep with the argument times 1000 and returns 1. |
| `004c4db1` | GuiApp::SleepMilliseconds | confirmed | Virtual method at GuiApp::vftable slot 76: calls Sleep with the argument as milliseconds and returns 1. |

**SoeUtil::Thread** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fc4d60` | SoeUtil::Thread::~Thread | confirmed | Scalar deleting destructor (vtable slot [0] of SoeUtil::Thread): calls the destructor body 00fc4c30 on this, then calls _free(this) when bit 0 of the flag argu… |

**(no class)** (61)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048eaa1` | ResolveDrawDeviceWin32 | confirmed | Builds the class identifier for GFC_Drivers_Draw_DrawDeviceWin32 and queries the global driver registry, initializing it when absent. On a miss it calls FUN_00… |
| `0049141f` | ResolveFontDeviceWin32 | confirmed | Builds the class identifier for GFC_Drivers_Draw_FontDeviceWin32 and queries the global driver registry, initializing it when absent. On a miss it calls FUN_00… |
| `00c194b0` | OpenFreeRealmsWebsite | confirmed | Calls the launcher browser helper with the FreeRealms.com URL, logs the exact failure message when it returns false, restores the exception-list pointer and re… |
| `00f9a640` | LoadTgNtdllExports | confirmed | Returns 1 at once when DAT_01cb4c9c is set. Otherwise LoadLibraryA("ntdll"), returning 0 on failure, then GetProcAddress for 14 names (TGLaunchUNIXApp, TGUNIXA… |
| `00fc7520` | DirectoryDelete | confirmed | Plain function: narrow path (param_1) and a recursive flag byte (param_2), returns nothing. Converts the path to wide. When the flag is zero it calls RemoveDir… |
| `00fc7af0` | DirectoryScanRecurse | confirmed | Recursive directory enumeration with FindFirstFileW / FindNextFileW over the path built from param_4 (and param_5 when set) with the format "%s/%s/*" (or the o… |
| `0048dcf8` | GetCursorHotspot | likely | Resolves hotspot outputs: if no icon handle is stored and a name is present, loads a temporary icon with LoadIconA; on GetIconInfo success it writes x/y hotspo… |
| `0048ea78` | DtorVtables1769298 | likely | Destructor of the object whose vtables are the unnamed data pointers PTR_FUN_01769298 and PTR_LAB_01769290: installs both vtable pointers, runs FUN_0052f673 (b… |
| `0048ebd6` | ReleaseGlobalsAndUnregisterIds | likely | Releases and clears global interface pointers DAT_01bc6738 and DAT_01bc673c. It then queries the registry for stored identifiers DAT_01bc6974 and DAT_01bc6cc0,… |
| `004cb462` | InitCriticalSectionSpinOrPlain | likely | Thiscall(cs, spin count): on first use loads kernel32 InitializeCriticalSectionAndSpinCount through LoadLibraryA/GetProcAddress and caches the pointer; calls i… |
| `004e4abc` | GlobalCsEnter | likely | Fastcall: lazily initialises the global critical section pointer DAT_01bc7e30 through 004cb462 (on the object at 01bc7f60), calls EnterCriticalSection on it, a… |
| `004e4ae3` | GlobalCsLeave | likely | Lazily initialises the global critical section pointer DAT_01bc7e30 through 004cb462 when null, then calls LeaveCriticalSection on it. Returns nothing. |
| `004ebc40` | CriticalSectionLeaveLazy | likely | Leaves the global critical section DAT_01bc7e30, creating it first through 004cb462 (InitCriticalSectionSpinOrPlain) when the pointer is null. Returns nothing.… |
| `006fb7a0` | TlsSlotAlloc | likely | Thiscall with no stack arguments (ECX=this, ret): calls TlsAlloc through the import slot, stores the returned index at this, and returns this. |
| `006fb7f0` | TlsSlotAlloc | likely | Thiscall with no stack arguments (ECX=this, ret): calls TlsAlloc through the import slot, stores the index at this and returns this. Same body as 006fb7a0. |
| `006fb820` | TlsSlotSetValue | likely | Thiscall with one stack argument (value, ret 4): calls TlsSetValue(*this, value) and returns the value in EAX. |
| `006fb840` | TlsSlotAlloc | likely | Thiscall with no stack arguments (ECX=this, ret): calls TlsAlloc, stores the index at this and returns this. |
| `006fb870` | TlsSlotSetValue | likely | Thiscall with one stack argument (value, ret 4): calls TlsSetValue(*this, value) and returns the value in EAX. |
| `006fb890` | TlsSlotAlloc | likely | Thiscall with no stack arguments (ECX=this, ret): calls TlsAlloc, stores the index at this and returns this. |
| `006fb9c0` | TlsSlotStorageGet | likely | Returns the address of the global slot object DAT_01bdf284. On first call it sets bit 0 of DAT_01bdf28c, initialises the slot through 006fb840 and registers an… |
| `006ffcb0` | PlatformSleepMs | likely | Calls Sleep with its single stack argument (milliseconds) and returns. |
| `006ffcd0` | ComputerNameLoad | likely | Reads the local computer name with GetComputerNameW into a 1024-wide buffer and, on success, passes that buffer and its parameter to 007014c0 (string store); t… |
| `007000a0` | EnvironmentVariableToIString | likely | Reads the environment variable named by the argument with __dupenv_s; on success assigns the value to the output string through IStringAssign 00402ce0, frees t… |
| `007002a0` | ExecutablePathNormalized | likely | Gets the running executable's path with GetModuleFileNameW(NULL, buf, 0x4000), converts it into a StringFixed<char,2048> through 007014c0, then passes that str… |
| `007003b0` | CommandLineArgsAfterProgram | likely | Takes GetCommandLineW, converts it through 007014c0 into a StringFixed<char,8192>, calls 006fbde0 with the constant 0x20 and offsets into the text by its resul… |
| `00700530` | ConsoleTitleSet | likely | Converts the narrow string argument into a WideStringFixed<2048> through 00701290, calls SetConsoleTitleW on its buffer and then releases the temporary through… |
| `00700760` | CurrentDirectoryToString | likely | Reads the current working directory into a 0x2000-wide-character buffer with __wgetcwd; when that succeeds it converts the text through 007014c0, normalises ba… |
| `007009d0` | PathFullNormalized | likely | Converts the narrow path argument to backslashes (00703d50) and to wide text (00701290), resolves it with GetFullPathNameW into a 0x2000-wide-character buffer,… |
| `00700c30` | DiskFreeSpaceQuery | likely | Normalises the narrow directory argument to backslashes (00703d50), converts it to wide text (00701290) and calls GetDiskFreeSpaceExW with three 64-bit outputs… |
| `00701840` | FileReadBounded | likely | Opens the path argument read-only through 00703e10, takes the file size from 00702dd0 and reads min(requested, size) bytes with 00702d50 into the buffer argume… |
| `00701940` | FileWriteBytes | likely | Opens the path argument with open mode 3 through 00703e10, writes the given byte block through 00703150 (FileWrite), then closes the file (007034d0) and destro… |
| `007919a0` | ReleaseGraphicsDllAndSubobjects | likely | Fastcall. Calls 00791910 (ReleaseDeviceSubobjects), then releases the object at this+0x45c (vtable slot 0 with 1), frees the object at +0x26c through 007f6320 … |
| `00f8f610` | ModuleHandleAssign | likely | Stores arg2 as the HMODULE at this+0. When a previous nonzero handle is held it is released with FreeLibrary first. |
| `00f8f720` | ModuleHandleScalarDeletingDtor | likely | Calls FreeLibrary on the HMODULE at this+0 when it is nonzero and clears it; calls _free(this) only when bit 0 of the flag argument is set. Returns this. |
| `00f8ff80` | ModuleHolderFreeBlock | likely | Reads the block pointer at this, calls FreeLibrary on the HMODULE stored in the block when nonzero (then clears it), and frees the block with _free. |
| `00f8ffb0` | ModuleHolderFreeBlockClear | likely | Same as 00f8ff80 but also clears the pointer at this after freeing the block: FreeLibrary on the stored HMODULE when nonzero, _free on the block, then zero at … |
| `00f90220` | ModuleHolderFreeAndAssign | likely | If the block pointer at this is nonzero: FreeLibrary on its stored HMODULE (clearing it), _free the block and clear the pointer; then stores arg2 at this+0. |
| `00f9a500` | TgGetNativePathTail | likely | Calls 00f9a900, then tail-jumps through DAT_01cb4cc8 when it is non-null; that global is the TGGetNativePath export that 00f9a640 resolves. |
| `00fa01d0` | DelayLoad_RegQueryValueExA | likely | Void with no arguments: tail-calls the delay-load helper 00fa01da for the RegQueryValueExA import slot. |
| `00fa01f1` | DelayLoad_RegOpenKeyExA | likely | Void with no arguments: tail-calls the delay-load helper 00fa01da for the RegOpenKeyExA import slot. |
| `00fc45a0` | GuardDestroy | likely | Destroys a guard object: when its DebugInfo field equals 0x4e414d45 it closes the handle stored in LockCount (CloseHandle), otherwise it calls DeleteCriticalSe… |
| `00fc45c0` | GuardTryEnter | likely | Non-blocking enter: for a named guard (DebugInfo 0x4e414d45) waits on the handle with timeout 0 and returns true when the wait returned 0; otherwise returns Tr… |
| `00fc45f0` | GuardEnter | likely | Blocking enter: for a named guard (DebugInfo 0x4e414d45) waits infinitely on LockCount; otherwise calls EnterCriticalSection on the object. |
| `00fc4610` | GuardLeave | likely | Releases the guard: for a named guard (DebugInfo 0x4e414d45) calls ReleaseMutex on LockCount, otherwise LeaveCriticalSection on the object. |
| `00fc4630` | GuardCreateNamedMutex | likely | Stores the marker 0x4e414d45 at this+0 (the DebugInfo field that makes the guard named), formats 'Global\%s' with the argument into a StringFixed<256> and wide… |
| `00fc4a50` | ThreadEntryRelease | likely | Thread entry point with the object as its stack argument: calls the object's virtual slot [2] (+0x8) with the object as ECX, then decrements the reference coun… |
| `00fc4ad0` | ThreadSetPriority | likely | Stores the level argument at this+0xc. When the thread handle in the state object is valid (not -1) and the level is 0 to 4, calls SetThreadPriority with -2, -… |
| `00fc4ba0` | ThreadWait | likely | Waits on the thread handle in the state object with the timeout argument (-1 is infinite) through WaitForSingleObject and returns true when the wait returned z… |
| `00fc4bd0` | ThreadStart | likely | Increments the reference count at this+4 with a locked add, clears the flag byte at state+8, starts a thread through 0110dfd0 with stack size this+8, entry 00f… |
| `00fc4d80` | SemaphoreCreate | likely | Thiscall with two stack arguments: creates an unnamed semaphore with CreateSemaphoreA (initial count and maximum count from the arguments, default attributes, … |
| `00fc4dd0` | SemaphoreWait | likely | Waits on the semaphore handle stored at the object's first dword: a negative timeout waits infinitely, otherwise the argument is the timeout; returns true when… |
| `00fc4ed0` | CountedGateCtor | likely | Thiscall with a byte argument: initialises a guard through 00fc4580 with spin count 4000 and a semaphore through 00fc4d80 (initial count 0, maximum 0x7fffffff)… |
| `00fc5140` | CountedGateDtorBody | likely | Enters the guard at this+0, and when the flag at +0xb0 is set clears the flags at +0xb0 and +0xb1 and releases the pending count on the semaphore at this+0x88 … |
| `00fc5330` | FormatLastErrorWide | likely | Calls GetLastError, formats that code with FormatMessageW (FROM_SYSTEM flag 0x1000, 0x800 wide chars) into a stack buffer of 2048 WCHARs, then calls 007014c0 w… |
| `00fc54a0` | ModuleCellLoadLibraryW | likely | Thiscall with two stack arguments: a narrow path (param_2) and an optional narrow directory (param_3). Converts both to wide strings. When param_3 is nonzero i… |
| `00fc72c0` | CreateDirectoryTreeW | likely | Plain function (no receiver) with a narrow path (param_1) and a flag byte (param_2), returns nothing. Converts the path to a narrow fixed string and, when the … |
| `00fc76f0` | MoveFileNarrow | likely | Plain function: two narrow paths (param_1 source, param_2 destination), returns nothing. Converts both to wide strings (00703d50 and 00701290), calls MoveFileW… |
| `010da690` | CriticalSectionAlloc | likely | Allocates a 0x18-byte block through 00d833a6 (operator new), stores its pointer at the first dword of the argument object and initialises it with InitializeCri… |
| `010da6b0` | CriticalSectionFree | likely | Calls DeleteCriticalSection on the pointer stored in the first dword of its argument, then frees that block through _free (00d82463). |
| `010da6d0` | CriticalSectionEnter | likely | Fastcall with the lock object in ECX. Loads the critical-section pointer stored in the first dword of the lock and calls EnterCriticalSection on it. |
| `010da6e0` | CriticalSectionLeave | likely | Fastcall with the lock object in ECX. Loads the critical-section pointer stored in the first dword of the lock and calls LeaveCriticalSection on it. |

### `platform.win32.clipboard`

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00440f13` | RegisterGClipboardWin32 | confirmed | Class registration: calls FUN_0053DA53 for the class named 'GClipboardWin32' with a 0x18 size argument, registers the local reference through FUN_004CCF27 and … |
| `0044394f` | RegisterGClipboardAccessGStringObject | confirmed | Class registration with no return value: builds the class record for the name "GClipboardAccess_GStringObject" with size-kind 0x18 through FUN_0053da53, extrac… |
| `004475ba` | ClipboardReadTextMatching | confirmed | Reads the Win32 clipboard and returns the text that matches the queried GString: it walks the formats with EnumClipboardFormats(0), maps each format to a name … |
| `0044775e` | ClipboardWriteOrEmpty | confirmed | Clipboard write/clear: with param_1 null it calls EmptyClipboard and returns whether that succeeded; otherwise it resolves the access class for the given GStri… |
| `00443617` | CopyGStringToWin32Clipboard | likely | Publishes text to the Win32 clipboard in two formats for the GString-shaped slot at this+0x30 (length dword at +0, UTF-16 payload at +4): it interns the class … |
| `00446878` | TempCtorCallDtor | likely | Constructs a temporary of the 01763928 clipboard-access class on its own stack frame with FUN_004466ce, calls the clipboard-copy routine FUN_00443617 with the … |

### `platform.win32.display`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091e710` | BaseClient::SyncMaximizedState | confirmed | Small virtual (slot 2): when FUN_008dd8f0 reports an active display, obtains the display object via FUN_00823810, queries its virtual slot +0x60 for the window… |

**GDisplayD3D9Impl** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0046cef2` | GDisplayD3D9Impl::~GDisplayD3D9Impl | confirmed | Destructor body: restores the object's two vtable pointers to PTR_FUN_01765040 / PTR_LAB_0176503c, releases the object at +0x24 through its vtable slot 8, free… |
| `0046cf8c` | GDisplayD3D9Impl::BlitImage | likely | Blits a source image (param_2) into the display's client area: reads the image's +0x94/+0x90 fields, returns 0 when the display or image is missing, and when t… |
| `0046d05e` | GDisplayD3D9Impl::PresentTrackedRect | likely | Flushes the object's tracked rect: returns 0 with no display, otherwise copies the tracked rect at +0x6c into a local region with FUN_004c9ed9 and blits it wit… |
| `0046d0c1` | GDisplayD3D9Impl::PresentRect | likely | Flushes the rect passed in param_2: returns 0 with no display, converts it with FUN_004c9ed9, optionally clips it against the display's client rect (FUN_004ca1… |
| `0046d9ed` | GDisplayD3D9Impl::DtorThunk | likely | Deleting-destructor variant: runs the GDisplayD3D9Impl destructor body FUN_0046cef2 and, when the low bit of param_2 is set, frees the object through thunk_FUN… |

**(no class)** (25)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00440569` | _InitMultipleMonitorStubs | confirmed | One-time multimonitor stub setup: records the NT-platform flag (FUN_00440525), resolves USER32 GetSystemMetrics, MonitorFromWindow, MonitorFromRect, MonitorFro… |
| `004407b3` | xGetMonitorInfoA | confirmed | Multimonitor stub wrapper for GetMonitorInfoA: calls the resolved USER32 GetMonitorInfoA when available; otherwise, for the pseudo-handle 0x12340042 with a buf… |
| `00440931` | xEnumDisplayDevicesA | confirmed | Multimonitor stub wrapper for EnumDisplayDevicesA: calls the resolved USER32 EnumDisplayDevicesA when available; otherwise, for (0,0) with a buffer of at least… |
| `004409bc` | InitMonitorApiFromUser32 | confirmed | Returns 1 when DAT_01BC4480 is already set or when USER32 EnumDisplayMonitors and GetMonitorInfoA resolve. On first load it also stores MonitorFromWindow. Miss… |
| `0045b7bd` | LoadGetMonitorInfo | confirmed | One-time loader for user32!GetMonitorInfoA: returns 1 when the cached function pointer at 0x1bc45f4 is set; otherwise LoadLibraryA("user32.dll") and GetProcAdd… |
| `0045bb27` | EnsureDirectDraw7 | confirmed | Ensures a DirectDraw7 interface exists: when it is null it LoadLibraryA("ddraw.dll"), fetches DirectDrawCreateEx, calls it with the IID at 0x17645dc into +4 an… |
| `008dd8f0` | ApplyDisplayWindowedSetting | confirmed | Queries the display mode helper 00f9a430; when it fails and the global flag DAT_01be47f1 is clear it constructs a StringFixed<64> with 00402a10, reads the conf… |
| `00915a90` | ResolveAndWriteDisplaySettings | confirmed | No arguments. Calls FUN_00790ab0(&w, &h) with locals initialised to 0; if it returns 0 logs FUN_006faad0(0,"Attempted to find optimal resolution failed. Revert… |
| `009173b0` | ReadDisplayMaximizedConfig | confirmed | Decoded from bytes: loads this+0x6d8 into ecx (not used by the callee), pushes -1, -1, 0, 0, the string 'Maximized' and the string 'Display', calls FUN_009107d… |
| `009179c0` | StoreDisplayResolutionConfig | confirmed | Thiscall with width (param_2) and height (param_3). Sets the byte at this+0x872 to 1 and builds a local StringFixed<64> (FUN_00402a10). Reads the Display confi… |
| `0091e000` | ApplyMaximizedDisplaySetting | confirmed | Reads the Display/Maximized config flag via FUN_009107d0, stores the comparison result against the requested value at this+0x871, and when they differ writes t… |
| `00922c10` | InitDeviceWithFallbacks | confirmed | Loops over display bring-up attempts: reads display settings through 00915b80, reads Rendering GraphicsQuality, TextureQuality and ShadowQuality through 009108… |
| `00fadd70` | QueryDirectDrawVideoMemory | confirmed | Loads ddraw.dll, zeroes a 0x218-byte block, enumerates adapters through DirectDrawEnumerateExA with 00fadd20 and creates a DirectDraw object with DirectDrawCre… |
| `00440525` | IsWin32NtPlatform | likely | Fills an OSVERSIONINFOA (size 0x94, fields zeroed by memset), calls GetVersionExA on it and returns true when dwPlatformId equals 2 (VER_PLATFORM_WIN32_NT). |
| `00440748` | xMonitorFromWindow | likely | Multimonitor stub wrapper for MonitorFromWindow: calls the resolved USER32 MonitorFromWindow when the stubs initialised; otherwise, for MONITOR_DEFAULTTONULL (… |
| `00440846` | xEnumDisplayMonitors | likely | Multimonitor stub wrapper for EnumDisplayMonitors: calls the resolved USER32 EnumDisplayMonitors when available; otherwise synthesises a single screen rect (0,… |
| `00440cf5` | CopyDescRetainString | likely | Copies the 15 dwords at param_2[0..0xE] to the same indices in param_1, passes param_2+0xF to FUN_004E8D36 for the trailing string member, and returns param_1. |
| `0045bb06` | DdrawModuleClose | likely | Releases the interface at +4 through its slot +8 and calls FreeLibrary on the module handle at +0. |
| `0045bb76` | QueryAvailableVideoMemory | likely | Ensures DirectDraw is loaded (FUN_0045bb27) and calls the interface's slot +0x5C - GetAvailableVidMem, the first method past the 0x58 IDirectDraw slot line - t… |
| `00790ab0` | ChooseDisplayResolution | likely | Thiscall with two output pointers (width, height), returns with the outputs zeroed first. Calls GetSystemMetrics(0) and (1) and discards them, then enumerates … |
| `00793ea0` | InitGraphicsAdapterFromPath | likely | Thiscall (ECX = this, param_2 unused, param_3 = C path). Copies the path into a heap buffer stored at this+0x43c, normalises the trailing slash, then creates t… |
| `008ed400` | DisplaySetWindowedMode | likely | Thiscall with a bool argument (wants windowed). When 00f9a430 returns nonzero it calls 00f9a580 and returns. Otherwise the target is 1 when the global DAT_01be… |
| `008ed480` | DisplayToggleWindowed | likely | Thiscall with no stack arguments. Calls 008dd8f0, then calls 008ed400(this, al == 0): it requests the windowed state opposite to the byte returned by 008dd8f0.… |
| `00f9a580` | TgSetFullscreenTail | likely | Calls 00f9a900, then tail-jumps through DAT_01cb4cfc when it is non-null; 00f9a7b0 resolves that global to TGSetFullscreen. |
| `00f9a5c0` | TgIsFullscreenTail | likely | Calls 00f9a900, then tail-jumps through DAT_01cb4d04 when it is non-null; 00f9a7b0 resolves that global to TGIsFullscreen. |

### `platform.win32.file`

**AppUtil::DefaultFileOpener** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fbf1f0` | AppUtil::DefaultFileOpener::NewDefaultFileRep | likely | Allocates 0x70 bytes through operator new (00d833a6), constructs a DefaultFileRep in the block with 00fbef80 when the allocation succeeds, and returns that poi… |

**AppUtil::DefaultFileRep** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fbef80` | AppUtil::DefaultFileRep::DefaultFileRep | confirmed | Calls the FileRep base constructor 00ae2d50, stores the AppUtil::DefaultFileRep vtable at this, calls 007039b0 to initialise the embedded byte-array and fixed-… |
| `00fbf0f0` | AppUtil::DefaultFileRep::~DefaultFileRep | confirmed | Scalar deleting destructor: calls the FileRep destructor body 00fbf090 on this, then calls _free(this) when bit 0 of the flag argument is set, and returns this. |
| `00fbefe0` | AppUtil::DefaultFileRep::Open | likely | Thunk: adds 8 to ECX (the embedded file state at this+8) and jumps to 00703e10. |
| `00fbeff0` | AppUtil::DefaultFileRep::Close | likely | Thunk: adds 8 to ECX and jumps to 007034d0. |
| `00fbf000` | AppUtil::DefaultFileRep::Read | likely | Thunk: adds 8 to ECX and jumps to 00702d50. |
| `00fbf010` | AppUtil::DefaultFileRep::Write | likely | Thunk: adds 8 to ECX and jumps to 00703150. |
| `00fbf020` | AppUtil::DefaultFileRep::GetSize | likely | Thunk: adds 8 to ECX and jumps to 00702dd0. |
| `00fbf030` | AppUtil::DefaultFileRep::QueryOffset | likely | Thunk: adds 8 to ECX and jumps to 00702e00. |
| `00fbf040` | AppUtil::DefaultFileRep::SeekAbsolute | likely | Pushes the two stack arguments (read from [esp+4] and [esp+8]) as call arguments, adds 8 to ECX, calls 00702e40, and returns with ret 8. |
| `00fbf070` | AppUtil::DefaultFileRep::GetStateField | likely | Thunk: adds 8 to ECX and jumps to 00702f40, which returns a dword stored in the file state. |
| `00fbf080` | AppUtil::DefaultFileRep::HasValidHandle | likely | Thunk: adds 8 to ECX and jumps to 00702f50, a query on the file object's handle validity. |

**AppUtil::FileRep** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fbf090` | AppUtil::FileRep::~FileRep | confirmed | Destroys the embedded members at this+8 through 00703a30, stores the AppUtil::FileRep vtable at *this, then stores the SoeUtil::RefCounted vtable (base-class o… |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004ec464` | PlatformCreateFileHolder | confirmed | Wraps CreateFile for a wide path holder (path chars at *param_1+4). Calls 004c59fa on param_1 first (error-mode guard). When DAT_01afbd14 == 2 it converts the … |
| `004ec526` | PlatformDeleteFileHolder | confirmed | Wraps DeleteFile for a wide path holder (chars at *param_1+4). Calls 004c59fa on param_1 first (error-mode guard). When DAT_01afbd14 == 2 it converts the path … |
| `004ec5c4` | PlatformMoveFileHolder | confirmed | Wraps MoveFile for two wide path holders (chars at *param_1+4 and *param_2+4). Calls 004c59fa on param_1 first (error-mode guard). When DAT_01afbd14 == 2 it co… |
| `004ec7ff` | PlatformGetFileAttributesHolder | confirmed | Wraps GetFileAttributes for a wide path holder (chars at *param_1+4). Calls 004c59fa on param_1 first (error-mode guard). When DAT_01afbd14 == 2 it converts th… |
| `004ec89d` | PlatformSetFileAttributesHolder | confirmed | Wraps SetFileAttributes for a wide path holder (chars at *param_1+4) with the attribute value param_2. When DAT_01afbd14 == 2 it converts the path through 004e… |
| `004ec9c0` | PlatformCreateDirectoryHolder | confirmed | Wraps CreateDirectory for a wide path holder (chars at *param_1+4) with security attributes param_2. Calls 004c59fa on param_1 first (error-mode guard). When D… |
| `004eca64` | PlatformRemoveDirectoryHolder | confirmed | Wraps RemoveDirectory for a wide path holder (chars at *param_1+4). Calls 004c59fa on param_1 first (error-mode guard). When DAT_01afbd14 == 2 it converts the … |
| `00d7e630` | FolderLock_Acquire | confirmed | If the dword at ECX is not -1 it returns at once (cookie check only). Otherwise it copies the folder path argument into a 260-byte stack buffer, appends the 4-… |
| `00d7ecd0` | FileMove_BaseNameIntoFolder | confirmed | Finds the base name of the source path after the last slash, backslash or colon, builds the destination with 00d7e4a0, and calls MoveFileA(source, destination)… |
| `01539830` | ViewHandleState_OpenNamed | confirmed | Thiscall (state at param_1, name string param_2): releases any previous view and handle (as 015397c0), takes the text after the last backslash of param_2, form… |
| `004c59fa` | SetErrorModeForDrivePath | likely | Thiscall-style (out byte at param_1, string object pointer in param_2): when the string length is at least 2 and its second wide character (buffer offset 2) is… |
| `004c5af7` | FileTellPos64 | likely | Fastcall (this in ECX, handle at +0x14): calls SetFilePointer(handle, 0, &high, FILE_CURRENT) and returns the 64-bit position (low DWORD in EAX, high in EDX); … |
| `004c5b47` | FileSize64 | likely | Fastcall (this in ECX, handle at +0x14): calls GetFileSize(handle, &high) and returns the 64-bit size (low in EAX, high in EDX); on failure with a nonzero GetL… |
| `004c5b86` | FileWriteBytes | likely | Thiscall (this in ECX, handle at +0x10): calls WriteFile with the buffer and count (synchronous, no overlapped) and returns the byte count written on success, … |
| `004c5ba9` | FileReadBytes | likely | Thiscall (this in ECX, handle at +0x14): calls ReadFile with the buffer and count (synchronous, no overlapped) and returns the byte count read on success, else… |
| `004c5c4f` | FileSeek32 | likely | Thiscall (this in ECX, handle at +0x14, offset param_2, origin param_3): calls SetFilePointer with the offset, no high word and the origin (0, 1 and 2 are pass… |
| `004c5c8b` | FileSeek64 | likely | Thiscall (this in ECX, handle at +0x14, low offset param_2, high offset param_3, origin param_4): maps the origin (0, 1, 2) and calls SetFilePointer with the o… |
| `004c5cf3` | StreamSetLengthRestorePos | likely | Thiscall (object in ECX, size param_2): saves the position through vtable slot +0x44, seeks to the argument (vtable slot +0x54 with 0), and when that seek is n… |
| `00d7cd10` | FileOpenCreateFileA_ModeString | likely | Cdecl, three stack arguments (handle out pointer, path, mode string). Maps the last 'a', 'r' or 'w' letter of the mode string to an access mask and creation di… |
| `00d7cdf0` | FileWriteCString | likely | Cdecl, two stack arguments (file handle, NUL-terminated string). Measures the string length, calls WriteFile with that many bytes (no terminator) and an unused… |
| `00d7e7b0` | Handle_CloseIfValid | likely | Fastcall (ECX = this). If the dword at this is not -1 it calls CloseHandle on it through the import slot and stores -1 back. Saves and restores ESI. |
| `00d7e810` | FindHandle_CloseGuarded | likely | Method with an installed SEH frame. Calls 00d818c0 on this on the normal path, then sets the frame state and calls 00d818e0 on this, and restores the exception… |
| `00d7e920` | Path_RemoveTreeIfNonEmpty | likely | If the first byte of the path buffer at ECX is nonzero, calls 00d7e2d0 on it with size 0x104 (260); otherwise returns. |
| `00d811c0` | PathAppendChild_CreateDirectory | likely | Appends a backslash (when the path does not already end with one) and the child name to the path buffer, stopping if the buffer limit would be exceeded. It the… |
| `00d81820` | FindData_CopyNameKindTime | likely | Register form: EAX = Win32 find-data record, ESI = destination record. Copies the NUL-terminated name at the record's +0x2c into destination +0; sets destinati… |
| `00d81870` | FindStateRecord_Reset | likely | Fastcall (ECX = record). Stores -1 at +0, and zero at +4 and +8, and returns the record in EAX. |
| `00d818c0` | FindHandle_CloseIfValid | likely | Fastcall (ECX = this). If the dword at this is not -1 it calls FindClose on it and stores -1 back. |
| `00d818e0` | FindHandle_CloseIfValid_Unwind | likely | Same body as 00d818c0: if the dword at ECX is not -1 it calls FindClose and stores -1. It is also called from exception-unwind funclets. |
| `00d81900` | FindFirst_BuildPatternAndOpen | likely | Thiscall (ECX = find record; first stack argument = directory; second stack argument is stored but not used for the search). Closes any open find handle at +0,… |
| `00fbe880` | OpenNewFileRepInto | likely | Calls 00fbe740 for the file-opener singleton, calls its vtable slot [1] (+0x4) to create a file rep, takes a reference on it (counter at +0x4), releases the pr… |
| `00fbe930` | CompareFileRepContents | likely | Opens two file reps through 00fbe880 (arguments 0 and 1), compares their sizes through vtable slot [6] (+0x18), and when they match reads both in chunks of up … |
| `00fbf4a0` | ReadTextFileToIString | likely | Opens the file rep in the RefPtr at param_1 through 00fbe880 (arguments 0 and 1), reads its whole contents into the IString through 00fbf250, and returns 1 whe… |
| `00fc0280` | CreateDirectoryForComposedPath | likely | Calls 007001c0, composes a path into a local IString with 00fc0070, then calls SHCreateDirectoryExA with the buffer pointer of that string (no security attribu… |
| `012f57f0` | FindFilesPushNames | likely | Calls FindFirstFileA on the pattern held in the std::string argument (its C string at +4 or the heap pointer when capacity is above 15). For each entry it copi… |
| `015397c0` | ViewHandleState_Close | likely | Fastcall: if the view word at this+0 is non-null it calls UnmapViewOfFile on (view-8) and clears it; if the handle word at this+8 is non-null it calls CloseHan… |

### `platform.win32.gdi`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00490fd5` | DrawGlyphGDI | likely | Draws one UTF-16 character through GDI onto the device context at +0x18: selects the font, sets white text and transparent background, and calls TextOutW at th… |
| `00491295` | MeasureCharABCWin32 | likely | Queries GetCharABCWidthsW and falls back to GetCharWidth32A plus GetTextExtentPoint32A when needed, then fills a 12-dword metrics record. Only in the GetCharAB… |

### `platform.win32.heap`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f0b0d0` | ProcessHeapAlloc | likely | Calls GetProcessHeap, then HeapAlloc(heap, 0, param_1) and returns HeapAlloc's pointer in EAX (cdecl, ret 0). |
| `00f0b0f0` | ProcessHeapFree | likely | Calls GetProcessHeap, then HeapFree(heap, 0, param_1) and returns HeapFree's BOOL in EAX (cdecl, ret 0). |

### `platform.win32.input`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091e500` | BaseClient::HandleDebugHotkey | confirmed | Input-message handler on BaseClient (virtual slot 64): given an event record where param_2[3] is a code and param_2[0] a state, it starts an image capture for … |

**Input::FRTWInputService** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0122f5a0` | Input::FRTWInputService::FRTWInputService | likely | Calls the Win32InputService ctor 012301e0 on this, stores the Input::FRTWInputService vftable at +0, stores 0xffffffff at +0x654 (param_1[0x195]), calls 0122e7… |

**Input::IKeyboard** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f6350` | Input::IKeyboard::DtorBody | confirmed | Stores the Input::IKeyboard vtable into *this and returns (7 bytes, no calls, no free). Called from three EH unwind funclets. |
| `012f6360` | Input::IKeyboard::~IKeyboard | confirmed | Scalar deleting destructor (vtable slot [0]): stores the Input::IKeyboard vtable into *this, and calls _free on this when bit 0 of the flag argument is set; re… |

**Input::IMouse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f6cb0` | Input::IMouse::DtorBody | confirmed | Stores the Input::IMouse vtable into *this and returns (7 bytes, no calls). Called from three EH unwind funclets. |
| `012f6cc0` | Input::IMouse::~IMouse | confirmed | Scalar deleting destructor (vtable slot [0]): stores the Input::IMouse vtable into *this, and calls _free on this when bit 0 of the flag argument is set; retur… |

**Input::InputService** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9a970` | InputService::Ctor | confirmed | Thiscall constructor: calls 0122f5a0 (base constructor) first, then stores Input::InputService::vftable at +0 and returns this. |
| `00f9aa20` | InputService::ScalarDeletingDtor | confirmed | Virtual slot 0 of Input::InputService: stores the vftable at +0, calls 0122dc80 (base destructor body), then frees this through _free when bit 0 of the byte ar… |
| `0122f7a0` | Input::InputService::AdvanceFrame | likely | Calls 012f6d40 and then 012f63e0 with no arguments and returns. |
| `0122f810` | Input::InputService::SetCursorFromResource | likely | Calls LoadCursorA(NULL, name) with the LPCSTR argument, stores the HCURSOR at this+0x18 and tail-jumps to SetCursor with it (SetCursor's return value is the re… |

**Input::InputServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0122f740` | Input::InputServiceBase::~InputServiceBase | confirmed | Scalar deleting destructor: stores the InputServiceBase vftable at +0, calls 01230d30, and calls _free on this when bit 0 of the argument is set. |
| `0122f660` | Input::InputServiceBase::InputServiceBase | likely | Calls the ServiceBase ctor 01230d10 on this, stores the Input::InputServiceBase vftable at +0 and writes 0 to the byte at +8. |
| `0122f6c0` | Input::InputServiceBase::DestructBody | likely | Stores the Input::InputServiceBase vftable at +0 and calls 01230d30 (ServiceBase dtor body). It does not free the object. |

**Input::Win32InputService** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012301e0` | Input::Win32InputService::Win32InputService | confirmed | Runs the InputServiceBase ctor 0122f660, stores the Win32InputService vftable at +0, calls 01230170 on the cursor map object at this+0xc (empty init), loads th… |
| `01230270` | Input::Win32InputService::~Win32InputService | confirmed | Scalar deleting destructor: calls 01230080 (DestructBody), then calls _free on this when bit 0 of the argument is set. |
| `0122fb90` | Input::Win32InputService::SetCursorByKey | likely | Looks up the key argument through 0122e3c0 in the map object at this+0xc; when the iterator is not the head (this+0x10) it stores the mapped HCURSOR (node+0x10… |
| `01230080` | Input::Win32InputService::DestructBody | likely | Stores the Win32InputService vftable at +0, then walks the cursor map from its begin to the head pointer at this+0x10, calling DestroyCursor on each node's +0x… |

**Input::Win32Keyboard** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f6390` | Input::Win32Keyboard::DtorBody | confirmed | Destructor body of the Win32 keyboard: stores the Input::Win32Keyboard vtable and then the Input::IKeyboard vtable into *this. No calls. |
| `012f64e0` | Input::Win32Keyboard::Win32Keyboard | confirmed | Constructor (fastcall, ECX = this, returns this): stores Input::IKeyboard then Input::Win32Keyboard vtables, clears the flag byte at +0x204, sets the byte at +… |
| `012f68f0` | Input::Win32Keyboard::~Win32Keyboard | confirmed | Scalar deleting destructor (vtable slot [0]): stores the Input::Win32Keyboard vtable, then the Input::IKeyboard vtable, into *this, and calls _free on this whe… |
| `012f6410` | Input::Win32Keyboard::KeyStateByte | likely | Win32Keyboard vftable[2]: with the key index argument (0..0xff) returns the byte at this+4+index; any index of 0x100 or more returns 0. |
| `012f6430` | Input::Win32Keyboard::KeyDownEdge | likely | Win32Keyboard vftable[3]: for the key index argument, returns the masked value (param_2 & 0xffffff00) when the index is above 0xff; returns 1 when the current … |
| `012f6460` | Input::Win32Keyboard::KeyUpEdge | likely | Win32Keyboard vftable[4]: for the key index argument, returns the masked value (param_2 & 0xffffff00) when the index is above 0xff; returns 1 when the current … |
| `012f6960` | Input::Win32Keyboard::SetKeyDown | likely | Win32Keyboard vftable[5]: for a key index below 0x100 copies the current table byte at this+4+index into the snapshot byte at this+0x104+index, then sets the c… |
| `012f6990` | Input::Win32Keyboard::SetKeyUp | likely | Win32Keyboard vftable[6]: for a key index below 0x100 copies the current table byte at this+4+index into the snapshot byte at this+0x104+index, then sets the c… |

**Input::Win32Mouse** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f6cf0` | Input::Win32Mouse::DtorBody | confirmed | Destructor body of the Win32 mouse: stores the Input::Win32Mouse vtable and then the Input::IMouse vtable into *this. No calls. |
| `012f6f90` | Input::Win32Mouse::~Win32Mouse | confirmed | Scalar deleting destructor (vtable slot [0]): stores the Input::Win32Mouse vtable then the Input::IMouse vtable into *this, and calls _free on this when bit 0 … |
| `012f6d80` | Input::Win32Mouse::HasMotion | likely | Win32Mouse vftable[1]: returns 0 when both dwords at this+0x18 and this+0x1c are zero, else 1. |
| `012f6e10` | Input::Win32Mouse::ButtonState | likely | Win32Mouse vftable[9]: for a button index below 3 returns the byte at this+0x20+index; any index of 3 or more returns 0. |
| `012f6e30` | Input::Win32Mouse::ButtonDownEdge | likely | Win32Mouse vftable[10]: for a button index above 2 returns the masked index value (param_2 & 0xffffff00); returns 1 when the current button byte at this+0x20+i… |
| `012f6e60` | Input::Win32Mouse::ButtonUpEdge | likely | Win32Mouse vftable[11]: for a button index above 2 returns the masked index value (param_2 & 0xffffff00); returns 1 when the current button byte at this+0x20+i… |

**LLKeyHook** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fbf720` | LLKeyHook::LLKeyHook | confirmed | Stores the LLKeyHook vtable at *this and returns. No other fields are set. |
| `00fbf870` | LLKeyHook::~LLKeyHook | confirmed | Scalar deleting destructor: stores the LLKeyHook vtable, and if the hook thread handle is non-null posts 0x401 to the thread, waits up to 10000 ms, clears the … |

**(no class)** (64)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00440428` | InitMouseDeviceEntry | confirmed | Populates an input-device descriptor: sets the type word [0] = 5 and [2] = 2, interns the wide device name L'Mouse' through FUN_004E889C and attaches it via FU… |
| `00442302` | RegisterGIODeviceSourceWin32 | confirmed | Class registration: calls FUN_0053DA53 for the class named 'GIODeviceSourceWin32' with a 0x18 size argument, registers the local reference through FUN_004CCF27… |
| `004426b5` | WindowsKeyToKeysym | confirmed | Windows virtual-key to keysym translation: large switch returning X11-style keysyms, e.g. VK_BACK -> 0xFF08, VK_TAB -> 0xFF09, VK_RETURN -> 0xFF0D, VK_SHIFT/VK… |
| `004432c0` | InitKeyboardDeviceEntry | confirmed | Populates an input-device descriptor: sets the type word [0] = 4 and [2] = 2, interns the wide device name L'Keyboard' through FUN_004E889C and attaches it via… |
| `00445f20` | DeviceKindRecordAndName | confirmed | Device description: copies the caller's record into *param_2, sets param_2[2] = 2, then switches on the device kind in param_1[0]: 1, 2 and 3 only name 'Input … |
| `009c0560` | CaptureMousePointer | confirmed | If not already captured, optionally releases GameScene pointer capture, captures the window mouse, records state, and hides/saves the cursor when configured. |
| `009c0600` | ReleaseMousePointerCapture | confirmed | Releases capture only when no capture flags remain and no transfer is active; restores cursor visibility and position if this object hid it. |
| `009cef40` | PollKeyboardStateBuffers | confirmed | For the selected 256-byte buffer, fills key bytes with 0x80 when needed or clears them, calls GetKeyboardState, and updates the cache flag. |
| `009cf1c0` | InitializeKeyboardActionBindings | confirmed | Initializes keyboard state flags and creates two SOE strings for each of 256 key indices using the formats isdown%d and ondown%d. |
| `0122e760` | LoadCursorTable | confirmed | Loads 19 cursor files named export:resources/cursors/cursor_*.cur with LoadCursorFromFileA (paths resolved via 012f4910) and inserts each (key, HCURSOR) pair t… |
| `004403d0` | BuildDeviceObjectTable | likely | Initializes the entry count at +0x40 to 1 and writes entry (2,0) at +0x28; appends (2,1), also appends (2,2) when byte +0x54 is nonzero, then appends (1,i) for… |
| `00441255` | GetKeysymModifierState | likely | Builds the modifier-key mask: walks a fixed table of (keysym, bit) pairs (0xFFE1 & 0xFFE2 ->5\|6, 0xFFE3/0xFFE4 ->0x50/0x60, 0xFFE9/0xFFEA ->0x500/0x600, 0xFF7… |
| `00441bd9` | EncodeWindowFlags | likely | Sets *param_3 = ((p & 2) << 4) \| (p & 0x88). Let u = -((p & 0x700) != 0) & 0x80000; the low-bit term is 0xC00000 when p bit 0 is clear, otherwise ~(p << 0x1B)… |
| `00441eb9` | ImeGetCompositionText | likely | IME helper: obtains the input context for the HWND at this+0x14 through the pointer table entry DAT_01BC4490, maps selector param_2 (1 -> 8, 2 -> 1, 4 -> 0x800… |
| `00441fc3` | ImeGetCursorPosition | likely | IME helper: obtains the input context for this+0x14 (DAT_01BC4490), reads the composition cursor position (kind 0x80) and the composition attribute buffer (kin… |
| `0044205e` | ImeSelectCandidate | likely | Obtains the IME context for the HWND at this+0x14, calls DAT_01BC44A4 with operation 0x12 and two caller-supplied stack values, releases the context, and retur… |
| `004424f8` | GetAsyncKeyFlags | likely | Key-state query: reads GetAsyncKeyState(param_1); sets bit 0 when the key is currently down, bit 0x20 when the caller's mask has bit 1 and the high bit is not … |
| `0044262d` | ResetKeyStateTables | likely | Resets the per-device key state: clears the byte at +0x12C, points +0x154 at the object itself, zeroes the 0x100-byte table at +0x28 and clears +0x128; returns… |
| `00442659` | GetModifierFlags | likely | Modifier mask builder: walks the global (virtual-key, flag) table DAT_01AF58A8 at 8-byte stride (17 entries, 0x88 bytes) reading GetKeyState for each key when … |
| `00442c28` | KeysymToWindowsKey | likely | Inverse translation: maps keysyms back to Windows virtual keys (0xFF08 -> 8, 0xFF09 -> 9, 0xFF0D -> 0xD, 0xFFE1 -> 0xA0, 0xFFE3 -> 0xA2, 0xFFE9 -> 0xA4, 0xFF50… |
| `004432fd` | InitKeyboardDeviceEntryEx | likely | Pushes stack words from entry offsets +0xC, +8, and +4 before calling FUN_004432C0; then clears dwords at the pointer loaded from entry +4 offsets +0x14 and +0… |
| `00443328` | TranslateKeyIndexToKeysym | likely | Key translation: rejects indexes beyond the object's key count (virtual +0x50) or whose table entry DAT_01AF5770 has a mapping of -1; otherwise translates thro… |
| `00445341` | InitMouseStateKind5 | likely | Mouse device state initialiser: caches the mouse-button count from GetSystemMetrics(0x2b) at +0x44 clamped to 8, clears +0x48/+0x4c/+0x50 (per-button state), s… |
| `004453c1` | SetMouseActiveFlag | likely | Sets the mouse-active byte at this+0x7c: when the requested value differs it stores it, builds a 0x1E-byte event record (flags dword 0 at +0, the requested val… |
| `0044543a` | InitMouseDeviceEntryEx | likely | Mouse device descriptor initialiser: runs FUN_00440428 (InitMouseDeviceEntry - type word 5, device name L'Mouse...'), then stores min(GetSystemMetrics(0x2b), 8… |
| `0044547c` | UpdateMouseState | likely | Per-frame mouse poll: returns 0 when the device pointer at +0x94 is null and 2 when the throttle check FUN_004400d8 says it is too early; otherwise it marks th… |
| `00445550` | SetMouseActiveMode | likely | Maps a mode selector onto the mouse active flag through FUN_004453c1: returns 0 when the device is not created (+0x94 == 0), sets the flag to 1 for mode 2, cle… |
| `00445695` | FormatKeyDisplayName | likely | Formats the display name of one device key/axis: reads the two-byte descriptor at *(this+4)+0x28+index*2; when the first byte is 2 it copies the name string fr… |
| `004457c5` | IteratorFactoryAt70_140 | likely | Iterator factory over the table owned by this: clears *param_2, stores the sentinel returned by FUN_004c04ca in param_2[1], sets the owner to *(this+0x70) and … |
| `00445957` | IteratorFactoryAt100_140 | likely | Iterator factory over a second device table: clears *param_2 and stores the FUN_004c04ca sentinel in param_2[1]; when the window is alive (byte +0x3c == 0) it … |
| `0044598d` | BuildMouseButtonMask | likely | Builds a mouse-button bitmask: for each button index below min(xGetSystemMetrics(0x2b), 8) it queries the object's virtual slot +0x88 for that index and sets b… |
| `004459d7` | GetCursorPosition | likely | Reads the cursor position into a POINT and, when param_3 is zero and the window is alive (byte +0x3c == 0), converts it with ScreenToClient using the HWND at t… |
| `00446152` | GetKeyStateModifierFlags | likely | Builds a modifier-key mask from the device's own key-state bytes: it walks the global (virtual-key, flag) table DAT_01af58a8 at an 8-byte stride for 0x88 bytes… |
| `00446181` | SetKeyboardActiveFlag | likely | Sets the keyboard-active byte at this+300 (0x12c): when the requested value differs it stores it, builds a 0x1E-byte event record (flags dword 0 at +0, the req… |
| `00446278` | SetKeyboardActiveMode | likely | Maps a mode selector onto the keyboard active flag through FUN_00446181: returns 0 when the device is not created (+0x144 == 0), sets the flag to 1 for mode 2,… |
| `004462c2` | IteratorFactoryAt150_81 | likely | Iterator factory for the keyboard device table: clears *param_2, stores the FUN_004c050a sentinel in param_2[1] and, when the device exists (+0x144 non-zero) a… |
| `0044645c` | FormatKeyNameFromKeysym | likely | Formats the name of one key: returns an empty string when the device does not exist; otherwise it translates the key index table entry (&DAT_01af5770)[index] t… |
| `0044650d` | GetKeyDownAsFloat | likely | Returns 1.0 as a float when the device exists and the state byte for key index param_2 (table DAT_01af5770, state at this+0x28+key) has bit 0 set, 0.0 otherwis… |
| `008a8cb0` | Wrapper_009c05d0 | likely | Byte-verified thunk (8b 89 44 06 00 00 / e9): loads this+0x644 into ecx and tail-jumps to FUN_009c05d0, which (when its +0x56 byte is clear) temporarily swaps … |
| `008a8cc0` | Wrapper_009c0670 | likely | Byte-verified thunk (8b 89 44 06 00 00 / e9 a5 79 11 00): loads this+0x644 into ecx and tail-jumps to FUN_009c0670, which clears the +0x56 flag and calls FUN_0… |
| `008a8cd0` | Wrapper_009c04f0_HideCursor | likely | Byte-verified (8b 89 44 06 00 00 / 6a 01 / e8 / c3): loads this+0x644 into ecx, pushes constant 1 and calls FUN_009c04f0, which records the cursor position and… |
| `008a8ce0` | Wrapper_009c04f0_ShowCursor | likely | Byte-verified (8b 89 44 06 00 00 / 6a 00 / e8 / c3): loads this+0x644 into ecx, pushes constant 0 and calls FUN_009c04f0, which restores the recorded cursor po… |
| `0094c760` | ActionNameIndexLookup | likely | Compares its NUL-terminated string argument, two bytes per step, with the name pointer (row offset 0, stride 0x18) of rows 0 to 80 of the table at 0x1b4198c; r… |
| `0094d0c0` | DefaultKeyByName | likely | Calls ActionNameIndexLookup (0094c760) on its first argument; writes the dword at row+0xc (0x1b41998 + 24*index) through its second pointer argument and return… |
| `009511f0` | KeyNameTablesRegister | likely | Cdecl-style with a name pointer and a key code: computes a hash of the name through 00705bc0, searches the bucket chains of the name table at 0x01b42174 (next … |
| `009c03d0` | InitializeInputCaptureState | likely | Clears capture state fields, initializes the associated object, and sets default center coordinates to 0x200 by 0x168. |
| `009c04f0` | SetCursorVisibilityAndRestorePosition | likely | When enabling capture, saves cursor position, hides the cursor, and sets a flag; when disabling, shows it, restores the saved position, clears the flag, and re… |
| `009c05d0` | CaptureMouseWithTemporaryOwner | likely | When capture has not been transferred, temporarily swaps the owner pointer, invokes mouse capture for the stored window, restores the owner, and records the tr… |
| `009c0670` | ClearTransferAndReleaseCapture | likely | Clears the transfer byte at +0x56 and calls the pointer-capture release routine using the stored window handle. |
| `009c0680` | DecodeWindowsMouseMessage | likely | Maps Windows mouse messages into internal event codes, tracks button/capture flags, creates or releases pointer capture, and computes relative motion and wheel… |
| `009c0a40` | DispatchWindowsInputMessage | likely | Routes recognized mouse and wheel messages to mouse-event decoders, converts key-like message ids into internal events, updates three held-input bits, and emit… |
| `009cefe0` | ReleaseKeyboardActionStrings | likely | Iterates 256 paired entries, obtains each string through virtual slot +0x24, hashes/releases it, decrements both reference counts, and clears both slots. |
| `009cf070` | UpdateKeyboardActionStates | likely | Flips the active keyboard buffer, polls it, then updates 256 action objects' key-down and transition fields from the current and previous high bits. |
| `009cf320` | InitializeKeyboardActionsAndReturnArgument | likely | Calls FUN_009cf1c0 to initialize keyboard action bindings, then returns its input argument. |
| `00a46030` | ShowCursorUntilVisible | likely | Calls ShowCursor(1) through the import pointer at 017514fc, and repeats while its return value is negative, so the display counter ends at zero or more. |
| `00c168f0` | SetKeyboardEnabled | likely | Lua C function bound as SetKeyboardEnabled: when exactly one numeric argument is present it reads it and calls FUN_0094c8c0(value != 0); returns 0. |
| `00c16960` | SetMovementEnabled | likely | Lua C function bound as SetMovementEnabled: when exactly one numeric argument is present it reads it and calls FUN_0094c8a0(value != 0); returns 0. |
| `00c169d0` | GetDefaultKeyBinding | likely | Lua C function bound as GetDefaultKeyBinding: with exactly one argument it reads the string (FUN_00e3a820) and looks it up through FUN_0094d0c0(string, &out); … |
| `00c16b30` | AddKeyBinding | likely | Lua C function bound as AddKeyBinding: requires more than one argument; reads arguments 1 and 2 as strings (FUN_00e3a820) and, when a third argument is present… |
| `00c16cc0` | SetKeyboardIntercept | likely | Lua C function bound as SetKeyboardIntercept: requires exactly one argument, reads it as a string (FUN_00e3a820) and calls FUN_00950d30(string); returns 0. |
| `00f9d010` | GetOrCreateInputService | likely | Fastcall registry accessor identical in structure to 00f9ce10 but keyed by DAT_01cbd918: creates Input::InputService through 00f9a970 on first use, registers i… |
| `00fbf660` | KeyHookMessageLoop | likely | Thread body: loops on GetMessageA. Message 0x402 installs a WH_KEYBOARD_LL hook (SetWindowsHookExA with type 0xd and callback LAB_00fbf620) when none is instal… |
| `00fbf780` | KeyHookSetInstalled | likely | With the flag in param_1: when the hook thread does not exist yet and the flag is set (or the previous request was set) and the request differs from the last o… |
| `012f6d40` | Input::Win32Mouse::FrameRollover | likely | Copies the position pair at this+8 and this+0xc to this+0x10 and this+0x14, clears the byte at this+0x29, the two dwords at this+0x18 and this+0x1c and the dwo… |

### `platform.win32.shell`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006faaf0` | WebBrowserLauncherOpen | confirmed | Opens a URL string with ShellExecuteA (verb open, show 1). A return value above 31 means success and returns 1. Otherwise it logs the error with GetLastError a… |
| `006faf80` | IStringResetCopyIfUrl | likely | Thiscall with one C-string argument: releases the old buffer when capacity is positive (count decrement, vtable slot [2] on the last reference), points the buf… |
| `006fb230` | ShellDesktopParseDisplayName | likely | Thiscall with two arguments (a PIDL out pointer and a path item). Stores the second argument at param_1[1], calls SHGetDesktopFolder for the shell desktop IShe… |

### `platform.win32.window`

**(no class)** (26)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00440b17` | RegisterGWindowManagerWin32 | confirmed | Class registration: calls FUN_0053DA53 to look up/create the class named 'GWindowManagerWin32' with a 0x18 size argument, registers the resulting local referen… |
| `00441141` | RegisterGWindowWin32 | confirmed | Class registration: calls FUN_0053DA53 for the class named 'GWindowWin32' with a 0x18 size argument, registers the local reference through FUN_004CCF27 and rel… |
| `0091d9f0` | CreateClientWindow | confirmed | Creates the game window when this+0x44 is still zero: reads the current display mode via FUN_00915b80, logs the parent HWND with the format string Launching wi… |
| `00440997` | PostOrSendMessage | likely | Message dispatch helper: sends WM via SendMessageA when param_5 is non-zero, otherwise posts it asynchronously via PostMessageA; parameters are (hwnd, msg, wPa… |
| `00440b84` | DefWindowProcWithFilter | likely | When a filter callback exists, calls it with (0, hwnd) for message 0x51, messages 0x10D-0x10F, or messages 0x282, 0x286, and 0x290; returns 0 if the callback c… |
| `00440bff` | GetSetPair58_5C | likely | Two-property accessor: for param_2 == 0 returns this+0x58 (param_3 == 2) or this+0x5C (param_3 == 3); for param_2 == 1 stores param_4 into the corresponding fi… |
| `00440d6a` | ShowMessageBox | likely | Accepts mode 0 or 1 and returns false for other values; converts local strings and param_1 through FUN_004E90AC/FUN_004E4E15, calls MessageBoxA(NULL, text, cap… |
| `004411ae` | GetSetCursor | likely | Cursor property accessor guarded by this+0x3C: reads return this+0x34 (0), this+0xE4 (1), this+0x20 (2) or this+0x24 (3); writes store the HCURSOR into those s… |
| `00441606` | SetCursorPosition | likely | GWindowWin32 method: guarded by this+0x3C, optionally maps the client point through ClientToScreen(this->hwnd at +0x34) when param_3 is clear, then calls SetCu… |
| `004416a1` | CaptureMouse | likely | GWindowWin32 method: when the window is alive, checks GetCapture(); if another window holds capture it returns 0, if this window already holds it returns 1, ot… |
| `004416ca` | ReleaseMouseCapture | likely | GWindowWin32 method: when the window is alive and GetCapture() reports this window, calls ReleaseCapture() and returns whether it succeeded; otherwise returns … |
| `00441711` | SetWindowText | likely | GWindowWin32 method: when the window is alive, sets the window caption through SetWindowTextW when the unicode flag DAT_01BC4458 is set (using the wide string … |
| `004417bb` | GetWindowText | likely | GWindowWin32 method: when the window is alive, queries GetWindowTextLengthA/W and then GetWindowTextA/W into the freshly sized string object, honouring the uni… |
| `0044185d` | InvalidateWindowRect | likely | GWindowWin32 method: when the window is alive, calls InvalidateRect(this->hwnd, rect, TRUE) using the caller rect (exclusive right/bottom increased by one) or … |
| `004418b8` | ValidateWindowRect | likely | GWindowWin32 method: when the window is alive, calls ValidateRect(this->hwnd, rect) using the caller rect (right/bottom increased by one) or the full window re… |
| `00441940` | ClientToScreenPoints | likely | GWindowWin32 method: when the window is alive, maps two consecutive POINTs through ClientToScreen(this->hwnd) and returns 1 only when both succeed, else 0. |
| `0044199b` | ScreenToClientPoints | likely | GWindowWin32 method: when the window is alive, maps two consecutive POINTs through ScreenToClient(this->hwnd) and returns 1 only when both succeed, else 0. |
| `00441a31` | GetWindowBorderInsets | likely | GWindowWin32 method: when the window is alive, obtains the client rect through its own virtual +0xD4 and the frame/border rect through virtual +0xFC, reads Get… |
| `00441d7f` | GetWindowFlags | likely | When this+0x3C is clear, starts from the cached mask at this+0x40; if HWND at this+0x34 is nonnull, reads GWL_STYLE (-0x10) and GWL_EXSTYLE (-0x14), then decod… |
| `00441e0e` | RestoreAndShowWindow | likely | When the window is alive, calls virtual +0x58 with this+0xCC if HWND is null. Otherwise calls 00440997 for WM_SYSCOMMAND/SC_RESTORE (0x112,0xF120), then for WM… |
| `004424b5` | ComputeMessageWaitTimeout | likely | Wait-budget helper: when the owning object's byte at +0xE4 is set and PeekMessageA reports no pending message, returns the value from FUN_005220B3 clamped to a… |
| `00445a6d` | ShowWindowFromState | likely | Window-state switcher: for a live window (the byte at this+0x3C is clear) it sets bit 1 in the owner's flag dword (*(this+0x28)+0x28), derives the ShowWindow c… |
| `00445af1` | SetFocusIfNotActive | likely | Focus setter: returns 0 when the window is not alive; otherwise it asks virtual slot +0xB0 for the current focus window and, when that is not this object or th… |
| `00445cfe` | UpdateCachedWindowRect | likely | Caches the window rectangle: for a live window (byte +0x3c clear) and a non-zero param_2 it calls GetWindowRect on the HWND at +0x34 and stores left, top, heig… |
| `008b8f30` | SetClientWindowEmbedMode | likely | Changes the game window's parent and style bits according to param_2 and param_3, writes param_2 at window extra offset -0x14, and returns whether embedded mod… |
| `00916f90` | ApplyWindowPlacement | likely | Reads window state into locals through FUN_00915b80 (twelve out-pointers). Calls FUN_008b8f30(local_2a==0, local_24, local_28); when it returns nonzero or the … |

## Source shards

[[S0001 00401040-00406b84|S0001]] (10), [[S0013 0043de42-004435e4|S0013]] (55), [[S0014 00443617-00447db4|S0014]] (25), [[S0018 004574f6-0045cca3|S0018]] (4), [[S0022 0046b77b-0046efeb|S0022]] (9), [[S0029 0048c619-00491c2c|S0029]] (8), [[S0040|S0040]] (13), [[S0041|S0041]] (24), [[S0048|S0048]] (3), [[S0049|S0049]] (10), [[S0141|S0141]] (16), [[S0142|S0142]] (23), [[S0143 00701a20-00707400|S0143]] (41), [[S0154|S0154]] (1), [[S0163|S0163]] (47), [[S0165|S0165]] (1), [[S0166|S0166]] (1), [[S0170|S0170]] (2), [[S0171|S0171]] (3), [[S0172 00798490-0079d890|S0172]] (2), [[S0173|S0173]] (3), [[S0174|S0174]] (1), [[S0175|S0175]] (1), [[S0184|S0184]] (1), [[S0186|S0186]] (5), [[S0192 00805a50-0080a790|S0192]] (3), [[S0196 0081b7d0-00821370|S0196]] (1), [[S0197 00821940-00827640|S0197]] (13), [[S0209|S0209]] (2), [[S0217|S0217]] (24), [[S0218 00896b90-0089d930|S0218]] (42), [[S0219 0089d990-008a3c80|S0219]] (1), [[S0220 008a3ce0-008a8500|S0220]] (8), [[S0221 008a86b0-008b1d80|S0221]] (11), [[S0222 008b1e00-008b7730|S0222]] (1), [[S0223 008b7790-008bcf40|S0223]] (32), [[S0224 008bcf60-008c5290|S0224]] (24), [[S0225 008c52a0-008ca980|S0225]] (16), [[S0227 008d0920-008d5c60|S0227]] (12), [[S0228|S0228]] (80), [[S0229 008d92f0-008dea46|S0229]] (34), [[S0230 008deaaa-008e47b0|S0230]] (63), [[S0231|S0231]] (36), [[S0232 008e82e0-008ec1d0|S0232]] (4), [[S0233 008ec2a0-008f0cc0|S0233]] (8), [[S0234 008f0d00-008f50c0|S0234]] (3), [[S0237 008ff3d0-00904e70|S0237]] (6), [[S0239 0090a890-009105b0|S0239]] (16), [[S0240 009105e0-009161d0|S0240]] (54), [[S0241 009162f0-0091aa10|S0241]] (10), [[S0242 0091af50-009200d0|S0242]] (5), [[S0243 00920130-00925ec0|S0243]] (4), [[S0244 00925fe0-0092b110|S0244]] (2), [[S0245 0092b170-0092f380|S0245]] (2), [[S0247 00934740-00939200|S0247]] (1), [[S0248 009392c0-0093d740|S0248]] (3), [[S0249 0093daf0-00942c60|S0249]] (15), [[S0250 00942c80-00947180|S0250]] (4), [[S0251 009471a0-0094a910|S0251]] (2), [[S0252|S0252]] (4), [[S0253|S0253]] (7), [[S0254 00954ec0-0095a710|S0254]] (12), [[S0255 0095a840-00960520|S0255]] (22), [[S0260|S0260]] (3), [[S0261 00976740-0097c910|S0261]] (5), [[S0263 00982f10-009897a0|S0263]] (13), [[S0267 0099ca10-009a1c60|S0267]] (4), [[S0270 009acf20-009b2140|S0270]] (5), [[S0273 009bd650-009c2c50|S0273]] (8), [[S0274 009c2cc0-009c8d80|S0274]] (1), [[S0275 009c8e90-009cdee0|S0275]] (47), [[S0276 009cdf80-009d3fe0|S0276]] (6), [[S0277|S0277]] (15), [[S0278 009d89c0-009e00c0|S0278]] (1), [[S0284 009f8470-009feec0|S0284]] (1), [[S0287 00a0c3a0-00a13140|S0287]] (57), [[S0290 00a1de60-00a243e0|S0290]] (1), [[S0294 00a35150-00a39a80|S0294]] (1), [[S0296 00a3f560-00a45650|S0296]] (4), [[S0297|S0297]] (4), [[S0300 00a59220-00a5da20|S0300]] (2), [[S0301 00a5da90-00a62140|S0301]] (1), [[S0307 00a7b930-00a81420|S0307]] (2), [[S0310 00a8ce60-00a91b00|S0310]] (7), [[S0313 00a9d3c0-00aa24e0|S0313]] (18), [[S0316 00aae6e0-00ab2c20|S0316]] (1), [[S0322 00acd6d0-00ad3250|S0322]] (1), [[S0325 00adee60-00ae4e70|S0325]] (24), [[S0326 00ae4fa0-00ae9b00|S0326]] (14), [[S0327|S0327]] (42), [[S0328|S0328]] (21), [[S0329|S0329]] (27), [[S0335 00b109b0-00b150b0|S0335]] (3), [[S0336 00b150e0-00b1b340|S0336]] (15), [[S0341 00b33bc0-00b39af0|S0341]] (8), [[S0342 00b39c10-00b3f310|S0342]] (15), [[S0347 00b56090-00b5b410|S0347]] (1), [[S0352 00b6cd80-00b71320|S0352]] (1), [[S0353 00b71490-00b77eb0|S0353]] (1), [[S0355 00b7c520-00b807c0|S0355]] (5), [[S0361 00b9a660-00ba0260|S0361]] (3), [[S0364 00ba74d0-00bac070|S0364]] (12), [[S0368|S0368]] (1), [[S0371|S0371]] (3), [[S0376 00be5b30-00bea020|S0376]] (5), [[S0380 00bf5ff0-00bf9410|S0380]] (1), [[S0384 00c0dcd0-00c171d0|S0384]] (5), [[S0385 00c17390-00c1f0a0|S0385]] (1), [[S0386 00c1f190-00c26aa0|S0386]] (2), [[S0394|S0394]] (2), [[S0398|S0398]] (1), [[S0400 00c6df10-00c73b60|S0400]] (1), [[S0404 00c84f00-00c89db0|S0404]] (16), [[S0410 00ca6401-00cab4e0|S0410]] (9), [[S0415 00cc1a00-00cc6bc0|S0415]] (1), [[S0416 00cc6c40-00ccbaa0|S0416]] (1), [[S0419 00cd5e90-00cdb1a0|S0419]] (2), [[S0427 00d01e40-00d07440|S0427]] (20), [[S0428 00d07500-00d0bf70|S0428]] (24), [[S0452|S0452]] (33), [[S0453|S0453]] (51), [[S0466|S0466]] (1), [[S0494 00e701f0-00e760b0|S0494]] (44), [[S0499|S0499]] (10), [[S0517|S0517]] (2), [[S0529|S0529]] (1), [[S0540|S0540]] (14), [[S0541|S0541]] (2), [[S0542|S0542]] (60), [[S0543|S0543]] (30), [[S0544|S0544]] (2), [[S0548|S0548]] (91), [[S0549|S0549]] (10), [[S0552|S0552]] (2), [[S0563|S0563]] (21), [[S0571 01044480-0104a2b0|S0571]] (2), [[S0572|S0572]] (6), [[S0575|S0575]] (5), [[S0581|S0581]] (1), [[S0582|S0582]] (3), [[S0584 01093ee0-01099b80|S0584]] (1), [[S0588|S0588]] (1), [[S0594|S0594]] (6), [[S0595|S0595]] (4), [[S0596|S0596]] (18), [[S0597|S0597]] (9), [[S0598|S0598]] (2), [[S0602|S0602]] (5), [[S0652|S0652]] (20), [[S0662|S0662]] (1), [[S0666 012773d0-0127d160|S0666]] (18), [[S0679|S0679]] (2), [[S0682|S0682]] (4), [[S0683|S0683]] (7), [[S0685|S0685]] (4), [[S0686|S0686]] (40), [[S0687|S0687]] (28), [[S0688|S0688]] (12), [[S0689|S0689]] (39), [[S0690|S0690]] (33), [[S0691|S0691]] (6), [[S0692|S0692]] (5), [[S0694|S0694]] (13), [[S0696|S0696]] (28), [[S0697|S0697]] (1), [[S0699|S0699]] (2), [[S0703|S0703]] (1), [[S0704|S0704]] (5), [[S0705|S0705]] (4), [[S0706|S0706]] (7), [[S0707|S0707]] (2), [[S0708|S0708]] (5), [[S0709|S0709]] (1), [[S0710|S0710]] (4), [[S0711 01369c70-0136dbd0|S0711]] (2), [[S0713|S0713]] (1), [[S0715|S0715]] (64), [[S0716|S0716]] (2), [[S0718|S0718]] (1), [[S0719|S0719]] (2), [[S0721|S0721]] (11), [[S0722|S0722]] (2), [[S0723|S0723]] (5), [[S0725|S0725]] (4), [[S0726|S0726]] (9), [[S0727 013be8e0-013c3b90|S0727]] (9), [[S0728|S0728]] (1), [[S0729|S0729]] (7), [[S0730|S0730]] (2), [[S0731|S0731]] (5), [[S0732|S0732]] (34), [[S0736|S0736]] (14), [[S0737|S0737]] (13), [[S0738|S0738]] (2), [[S0739|S0739]] (60), [[S0743|S0743]] (3), [[S0748|S0748]] (16), [[S0749|S0749]] (7), [[S0751|S0751]] (1), [[S0753|S0753]] (5), [[S0761|S0761]] (4), [[S0764|S0764]] (10), [[S0767|S0767]] (1), [[S0770|S0770]] (4), [[S0777|S0777]] (3), [[S0785|S0785]] (1), [[S0793|S0793]] (8), [[S0797|S0797]] (15)
