# System: debug

98 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (58)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008dddf1` | `DebugDumpFileLength` |  | Size step of the module version diagnostics; the state (file handle, name, log buffer) is carried in the caller-set registers EBP/ESI/EDI rather than parameters. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008dde8a` | `DebugDumpFileCreationTime` |  | Creation-time step of the module version diagnostics. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008ddf2c` | `DebugDumpFileModifiedTime` |  | Modification-time step of the module version diagnostics. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008ddfb5` | `DebugDumpFileIsDirectory` |  | Directory-flag step and teardown of the module version diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de160` | `PlayClient::DumpDiagnostics` | PlayClient | Entry point of the PlayClient diagnostics dump (client build version plus the following field chain); the vtable slot identifies the owner and the string identifies the build. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008deaaa` | `DebugDump_IsNonattackable` |  | Adds the nonattackable state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008deb0f` | `DebugDump_IsFalling` |  | Adds the falling state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008deb6f` | `DebugDump_IsIgnoringCollision` |  | Adds collision-ignore state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008debcf` | `DebugDump_IsJumping` |  | Adds jumping state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dec2f` | `DebugDump_IsJuking` |  | Adds juking state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dec8f` | `DebugDump_IsInWater` |  | Adds water state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008decef` | `DebugDump_IsSwimming` |  | Adds swimming state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008ded4f` | `DebugDump_IsMoving` |  | Adds movement state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dedae` | `DebugDump_MicrophoneAttached` |  | Adds microphone attachment state to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dee0d` | `DebugDump_Gender` |  | Adds the character gender field to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dee6a` | `DebugDump_ModelCustomizationName` |  | Adds the model customization name to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008deeca` | `DebugDump_HairModelName` |  | Adds the hair model name to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008def2a` | `DebugDump_HeadModelName` |  | Adds the head model name to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008def86` | `DebugDump_SkinToneName` |  | Adds the selected skin-tone name to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df000` | `DebugDump_FacePaintName` |  | Adds the selected face-paint name to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df07a` | `DebugDump_TerrainObjectId` |  | Adds the terrain object identifier to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df0d9` | `DebugDump_Heading` |  | Adds derived heading to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df158` | `DebugDump_VerticalVelocity` |  | Adds vertical velocity to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df1bb` | `DebugDump_VerticalOffset` |  | Adds vertical offset to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df21e` | `DebugDump_DistanceToGround` |  | Adds computed distance to ground to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df281` | `DebugDump_PlayerBlendTimeSec` |  | Adds player blend duration to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df2ef` | `DebugDump_WaterHeight` |  | Adds water height to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df35d` | `DebugDump_WaterDepth` |  | Adds water depth to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df3cb` | `DebugDump_WaterDisplacement` |  | Adds calculated water displacement to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df42e` | `DebugDump_Position` |  | Adds position to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df4af` | `DebugDump_Facing` |  | Adds facing vector to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df52e` | `DebugDump_LookDirection` |  | Adds look direction to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df5ac` | `DebugDump_Velocity` |  | Adds velocity vector to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df62b` | `DebugDump_GroundNormal` |  | Adds ground normal to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df6aa` | `DebugDump_GroundPosition` |  | Adds ground position to the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df7a0` | `DebugDump_ZoneGeometry` |  | Adds current zone geometry to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df7f8` | `DebugDump_RenderLevel` |  | Adds render-level status to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df8be` | `DebugDump_ShadowQuality` |  | Adds shadow quality to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df91b` | `DebugDump_RenderLevelValue` |  | Adds the graphics wrapper's render level to the dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df979` | `DebugDump_DisplayWidth` |  | Adds display width to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008df9d5` | `DebugDump_DisplayHeight` |  | Adds display height to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfa32` | `DebugDump_DisplayRefreshRate` |  | Adds display refresh rate to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfa8f` | `DebugDump_Fullscreen` |  | Adds fullscreen mode to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfaf9` | `DebugDump_CameraPosition` |  | Adds camera position to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfb77` | `DebugDump_CameraOrientation` |  | Adds camera orientation to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfc62` | `DebugDump_DeviceCapabilities` |  | Adds graphics device capabilities to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfd7c` | `DebugDump_MaxVertexShaderVersion` |  | Adds maximum vertex-shader version to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfdda` | `DebugDump_MaxPixelShaderVersion` |  | Adds maximum pixel-shader version to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfe38` | `DebugDump_TextureQuality` |  | Adds texture-quality setting to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfe99` | `DebugDump_AvailableVideoMemory` |  | Adds startup video-memory availability to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008dfef7` | `DebugDump_Locale` |  | Finishes the player and rendering diagnostics with locale. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008ed4a0` | `DumpBoneNamesToSink` |  | Debug listing of an actor's bone names and count. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008efad0` | `ReportCriticalError` |  | Reports a fatal client error: writes the critical-error log, prepares the crash-report URL, and exits the process when the exit flag is set. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `00942c80` | `DebugStatsPageFormat` |  | Debug overlay statistics text builder (timings, render, memory, audio, animation, zone, network, GUI pages). | [[Native Map/Shards/S0250 00942c80-00947180|S0250]] |
| `00a5c550` | `InitializeDebugDataCollections` |  | Initialize embedded arrays and list used by debug data. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00a5c800` | `CleanupDebugDataCollections` |  | Release the embedded debug collections initialized by 00a5c550. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00bf8680` | `LogStandingOn` |  | Debug log line recording what an actor stands on, written to StandingOn.log. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00cda810` | `FormatMovementDebugString` |  | Builds a movement-status debug line for the caller. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## 🟢 likely (34)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008de1ea` | `DebugDump_PlayerGuid` |  | Player-id step of the PlayClient diagnostics chain started by 008de160. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de249` | `DebugDump_ProxiedCharacter` |  | Character-presence branch of the diagnostics chain: reports the missing character or the character name. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de2a6` | `DebugDump_CharacterNameRuntime` |  | Character/runtime step of the diagnostics chain; it also anchors the module-version dump call for GFxWrap.dll. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de360` | `DebugDump_ActorDefinitionName` |  | Actor-definition step of the diagnostics chain started by 008de160 (no direct caller is listed because the chain enters it by a tail jump). | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de3bf` | `DebugDump_SubTextName` |  | Sub-text/nameplate step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de41f` | `DebugDump_CharacterGuid` |  | Guid step of the diagnostics chain; the guid comes from the character's virtual accessor rather than a raw field. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de48b` | `DebugDump_CharacterHashKey` |  | Hash-key step of the diagnostics chain (the hash key is the XOR of the two 32-bit halves). | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de4f0` | `DebugDump_ModelId` |  | Model-id step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de54d` | `DebugDump_HairTint` |  | Hair-tint step of the diagnostics chain; the value comes from the character customisation helper. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de5aa` | `DebugDump_EyeTint` |  | Eye-tint step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de607` | `DebugDump_MaxHitpoints` |  | Maximum-hitpoints step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de665` | `DebugDump_CurrentHitpoints` |  | Current-hitpoints step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de6c3` | `DebugDump_MaxMana` |  | Maximum-mana step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de721` | `DebugDump_CurrentMana` |  | Current-mana step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de77f` | `DebugDump_MaxLives` |  | Maximum-lives step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de7dd` | `DebugDump_CurrentLives` |  | Current-lives step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de83b` | `DebugDump_ExpectedSpeed` |  | Expected-speed step of the diagnostics chain. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de89c` | `DebugDump_AppRuntimeSeconds` |  | Uptime step of the diagnostics chain; the /1000 division in the caller shows the accessor returns milliseconds. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de919` | `DebugDump_IsStunned` |  | Status step of the diagnostics chain: bit 6 of the character status word is the stunned flag. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de97d` | `DebugDump_IsKnockedOut` |  | Status step of the diagnostics chain: bit 7 of the character status word is the knocked-out flag. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008de9e2` | `DebugDump_IsKnockedBack` |  | Status step of the diagnostics chain: bit 9 of the character status word is the knocked-back flag. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008dea46` | `DebugDump_IsPoppedUp` |  | Status step of the diagnostics chain: bit 0x14 of the character status word is the popped-up flag; the chain continues past this shard boundary. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008dfbf5` | `DebugDump_HorizontalViewAngle ⚠downgrade` |  | Adds horizontal view angle to the debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008efda0` | `BaseClient::FatalErrorForModeCode` | BaseClient | Fatal-error entry point that maps a three-way failure mode to a critical error code and exits. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `00915660` | `CleanupCall0090f980 → CleanupPerformanceProfilerSampleMap` |  | EH-only cleanup routine for an embedded PerformanceProfilerSampleMap member of its owner, which is unidentified here. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00922b70` | `Wrapper_0091d980` |  | Unwind entry that runs the metrics hash container destructor body 0091d980 (called from EH unwind funclets). | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00a0f640` | `ConsoleLineRing_Reset` |  | Clear/restart of the 500-line console text ring immediately before the ring's constructor/destructor pair (00a0f700/00a0f650). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0f650` | `ConsoleLineRing_Dtor` |  | Tears down the 500-line console ring buffer object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0f700` | `ConsoleLineRing_Ctor` |  | Builds the console/log output object: a 500-line ring plus a file name of the form '<argument>.outEdit'. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0f7a0` | `ConsoleLineRing_NewLine` |  | Starts a new empty console output line, dropping the oldest line when the 500-line ring is full. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0f7f0` | `ConsoleLineRing_Append` |  | Adds text to the current console output line (the counterpart of the new-line advance). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0f820` | `ConsoleLineRing_Render` |  | Pushes the accumulated console/log lines into the edit control that displays them. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00c21130` | `EmitSwfFrameInfo` |  | Emits the SWF frame-info block, per-display timing and primitive rows, and total through the manager callback. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00cd62f0` | `ParseBattleMagesCameraCommand` |  | Provides a local debug command for Battle Mages camera control. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095a840` | `EmitColoredSegmentPairs ?` |  | Draws a set of colored basis-like line pairs; exact scene meaning is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00d051c0` | `SetByteField22250 ?` |  | Stores the low byte of its stack argument at this+0x22250 and returns (ret 4). | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d051d0` | `CallThenReturnThis ?` |  | Passes this (ECX) to 006fb840 and returns this in EAX (12 bytes: push esi; mov esi,ecx; call 006fb840; mov eax,esi; pop esi; ret). | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ddc00` | `— ⚠reject` |  | Version-resource and file-length stage of the client diagnostic dump; 008de2a6 calls it with GFxWrap.dll, while timestamp steps continue through 008dddf1/008dde8a. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008df728` | `— ⚠reject` |  | Collects engine and graphics context for the player debug dump. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `01097040` | `— ⚠reject` |  | Formats a byte count as MB, KB or bytes for diagnostic output; its caller 01097c10 (asset dump) is in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00cd62f0` | `ParseBattleMagesCameraCommand` | send | PacketTunneledClientPacket |  | Hypothesis: the load-command branch can cause a client-originated tunneled request to reach the original service. This depends on 00cd6230, 00cd5fe0, and 00925640; no server-side camera behavior is established. |
