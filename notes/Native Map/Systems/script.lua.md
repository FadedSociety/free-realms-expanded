# System: script.lua

246 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (48)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008baab0` | `PlayClient::InvokeGuiShutdownCallback` | PlayClient | Runs registered GUI shutdown script handling. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00977740` | `ClientLuaReadClientStringField` |  | Resolves a Lua handler name stored as a string or number in the Lua global table Client; callers append a method name and dispatch the result. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979120` | `ClientLuaInternals::ClientLuaInternals` | ClientLuaInternals | Constructs the native Lua host object (ClientLuaInternals) and publishes it as the global internals pointer. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009791c0` | `ClientLuaInternals::DtorBody` | ClientLuaInternals | Destructor body of the native Lua host object: closes its Lua state and destroys its map. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979260` | `ClientLuaInternals::ScalarDeletingDtor` | ClientLuaInternals | Scalar deleting destructor of ClientLuaInternals, installed at ClientLuaInternals::vftable slot [1]. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009797a0` | `ClientLuaCallByName` |  | Generic native-to-Lua call bridge: invokes a named Lua global function or Object:Method with tagged arguments and optionally returns the result as a tagged value. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009c5290` | `ExecuteLuaEventScalarDeletingDtor` | ExecuteLuaEvent | Scalar deleting destructor for ExecuteLuaEvent. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `00a18960` | `Array_ClientLuaParam_BufferHook` | SoeUtil::Array<ClientLuaParam,20,1> | Buffer acquisition for the script argument array, serving up to twenty parameters inline. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a8de60` | `CombatGroupHandlerSetHealth` |  | Forwards a health update to the CombatGroupHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8e000` | `CombatGroupHandlerSetPower` |  | Forwards a power update to the CombatGroupHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8e170` | `GroupHandlerAddPlayer` |  | Forwards group-member additions to the applicable Lua handlers and records changed keys. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8e3e0` | `GroupHandlerRemovePlayer` |  | Forwards group-member removal to the applicable Lua handlers and updates the changed-key list. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8ec00` | `GroupHandlerAddAbilityEffect` |  | Forwards an ability-effect addition to the GroupHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8ee30` | `GroupHandlerRemoveAbilityEffect` |  | Forwards an ability-effect removal to the GroupHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00bfe520` | `RegisterAdventurersJournalBindings` |  | Registers Lua callbacks exposed by AdventurersJournal. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe540` | `RegisterClientNudgeBindings` |  | Registers Lua callbacks exposed by ClientNudge. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe5e0` | `RegisterSocialShareBindings` |  | Registers Lua callbacks exposed by SocialShare. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe710` | `SocialShare::RequestLoginURL` | SocialShare | Requests a login URL from a supported social-share provider. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfea50` | `RegisterLuaUtilBindings` |  | Registers utility callbacks exposed to Lua. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfec70` | `RegisterFactoryBindings` |  | Registers Lua callbacks exposed by Factory. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff1b0` | `RegisterCharacterCreateServerUtilsBindings` |  | Registers Lua callbacks for character creation server utilities. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff1d0` | `RegisterClientEffectBindings` |  | Registers Lua callbacks exposed by ClientEffect. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff480` | `RegisterMountBindings` |  | Registers Lua callbacks exposed by Mount. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bffbc0` | `Mount::Upgrade` | Mount | Requests a mount upgrade using the selected mount and upgrade item. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bffca0` | `RegisterQuickChatBindings` |  | Registers Lua callbacks exposed by QuickChat. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bffd80` | `RegisterCoinStoreScreenBindings` |  | Registers Lua callbacks exposed by the coin store screen. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00dc0` | `RegisterClaimScreenBindings` |  | Registers Lua callbacks exposed by ClaimScreen. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01000` | `RegisterCinematicBindings` |  | Registers Lua callbacks exposed by Cinematic. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01dd0` | `RegisterLuaDataSourceMetatable` |  | Sets up the Lua metatable used by data-source wrappers. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01f70` | `RegisterRecipeBindings` |  | Registers Lua callbacks exposed by Recipe. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01fd0` | `RegisterMarketplacePortalScreenBindings` |  | Registers Lua callbacks exposed by the marketplace portal screen. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c021d0` | `RegisterPromotionalScreenBindings` |  | Registers Lua callbacks exposed by the promotional screen. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c02370` | `RegisterInGamePurchaseStoreBindings` |  | Registers Lua callbacks exposed by the in-game purchase store. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c04950` | `RegisterTradingCardGameBindings` |  | Registers Lua callbacks exposed by TradingCardGame. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c04a80` | `RegisterOnlineHelpWindowBindings` |  | Registers Lua callbacks exposed by OnlineHelpWindow. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c04ff0` | `RegisterGameActionQueueBindings` |  | Registers Lua callbacks exposed by GameActionQueue. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c05150` | `RegisterVideoUploadBindings` |  | Registers Lua callbacks exposed by VideoUpload. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c056a0` | `RegisterVideoCaptureBindings` |  | Registers Lua callbacks exposed by VideoCapture. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c1a4d0` | `CastLuaValueToGuiSurfaceCtrl` |  | Provides a typed GUI control conversion for the calling binding. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b1c0` | `ReadyForActivity` |  | Lua callback for the distinct activity-launch ReadyNotification path, not the pending mini-game 39/5 start flow. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c24f30` | `GetGfxCtrl` |  | Validates and unwraps the GfxCtrl Lua userdata. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c251f0` | `RegisterGfxCtrlMetatable` |  | Creates the GfxCtrl metatable for Lua bindings. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25c50` | `PushDsTable` |  | Wraps a data-source table for Lua scripts. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25c90` | `GetDsTable` |  | Validates and unwraps DsTable userdata. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c260b0` | `RegisterDsTableMetatable` |  | Creates the DsTable metatable used by Lua scripts. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c26160` | `DsTable_GetData` |  | Returns one data-source cell to Lua. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c269c0` | `RegisterImageControlMetatable` |  | Creates the ImageControl metatable for Lua UI scripts. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c893f0` | `DispatchFirstTimeEventScript` |  | Executes a script callback supplied in a first-time-event entry. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## 🟢 likely (46)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00976740` | `ClientLuaReadClientFloatField` |  | Native read of a numeric or string-convertible field of the Lua global table Client, as a float. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976860` | `ClientLuaToggleClientFlag` |  | Toggles a flag stored in the Lua global table Client; called from the debug hotkey handler. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976900` | `ClientLuaIsGlobalFunction` |  | Predicate: tells native code whether a named Lua global exists as a function before it calls it. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009769b0` | `ClientLuaIsGlobalFunctionGuarded` |  | Guarded form of the Lua global-function test; BaseClient GUI init uses it to check for a GuiOnInit callback. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976cb0` | `ClientLuaInternalsInitState` | ClientLuaInternals | Initialises the native Lua state for the client and installs the global Client table. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976d50` | `ClientLuaLoadRunLogErrorStatus` |  | Runs a Lua file and records load or run errors to LuaErrors.log; returns the error status. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976e90` | `ClientLuaLoadRunLogErrorVoid` |  | Runs a Lua file and logs a run error to LuaErrors.log; load status is not checked. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976fa0` | `ClientLuaCollectGarbageRateLimited` |  | Runs a full Lua collection at most once per interval stored in +0x1b8 (set to 1000 in 00976cb0). | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00977020` | `ClientLuaCollectGarbageForwarder` |  | Entry that requests the rate-limited Lua garbage collection through the holder pointer at this+4. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00977890` | `ClientLuaPushTaggedValue` |  | Serialises one tagged value (number, string, boolean, table or nil) onto the Lua stack for the generic Lua call bridge. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00978f80` | `ClientLuaBindSetNamedString` |  | Lua-facing setter that stores a string value under a string name in the internals map (through 009787a0). | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979280` | `ClientLuaInternalsCreate` |  | Factory that allocates and constructs the native Lua host object. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009792f0` | `TaggedValueLessThan` |  | Ordering comparator for tagged values, called by the tree insertion walker 0070a5a0 (accepted label). | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979590` | `ClientLuaParamReadStackValue` |  | Converts the Lua value on top of the stack into a tagged ClientLuaParam (nil, boolean, number, string, table) for the generic Lua call bridge. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979b70` | `ClientLuaParamArrayFromTypeString` |  | Converts a type-code string and its arguments into a ClientLuaParam array; the array has no consumer in this function. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00a188c0` | `Array_ClientLuaParam_Dtor ⚠downgrade` | SoeUtil::Array<ClientLuaParam,20,1> | Tears down the argument array a Lua binding builds for a script call; the element destructor handles the seven ClientLuaParam type tags. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a8fa80` | `ForwardRecordToGroupHelper` |  | Adapts a stored record for the group-member Lua callback helper. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a90790` | `ForwardFirstFieldToCallback` |  | Packet-dispatch adapter for the AchievementHandler objective callback; the domain comes from its callee. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a907b0` | `ForwardFirstFieldToCallbackVariant` |  | Packet-dispatch adapter for the AchievementHandler objective callback; the domain comes from its callee. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a907d0` | `ForwardFirstFieldToCallbackVariant2` |  | Packet-dispatch adapter for the AchievementHandler objective callback; the domain comes from its callee. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00bfe600` | `SocialShare::GetProviderStatus ⚠downgrade` | SocialShare | Queries the status for supported social-share providers. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfead0` | `FormatAndPushLuaString` |  | Formats a string and pushes it through the Lua stack helper. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfeb90` | `FormatAndPushLuaString` |  | Formats a string and pushes it through the Lua stack helper. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff8e0` | `Mount::Despawn ⚠downgrade` | Mount | Despawn path requests a dismount when a mount is active. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c17c00` | `PushDerivedBooleanThenZero` |  | Calls the output helper with a derived boolean followed by an unconditional zero; the caller stack contract is unknown. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17cf0` | `LuaSetCameraAutoAdjustment` |  | Lua-facing setter for the client camera auto-adjustment flag. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17f80` | `LuaGetActivityCountByName` |  | Returns the guarded count for the named activity. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18000` | `LuaGetActivityTimeRemainingByName` |  | Returns the guarded remaining-time value for a named activity. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c196a0` | `InvokeRowSelectionRenumbering` |  | Invokes a row-selection helper that marks a matching entry and renumbers other positive indices; this wrapper does not identify the owning UI list. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1ada0` | `GetGameSceneLocalPosition ⚠downgrade` |  | Returns two local-position coordinates through a GUI-facing argument helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1af80` | `SetQuickChatChannelNinePair` |  | Updates the channel-9 quick-chat value through the client chat-to-channel packet path. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b000` | `InvokePositionInputWithDefaults` |  | Invokes the client position-input routine using a GUID and a four-dword default vector; the vector contents are not identified here. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b310` | `PushClientFieldF78UnlessLookupReturnsZero` |  | Returns the client dword at +0xf78 unless the optional key lookup returns zero. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b7c0` | `ParseDelimitedIntegerPairForDisplayResolution` |  | Parses a delimited pair and forwards it to the display-resolution configuration routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c8f0` | `ExecuteConsoleCommandArgument → DoCommand` |  | Runs a console command line supplied through the argument interface. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1e520` | `ApplyClientStringWhenFlag62CSet → SetStringValue` |  | Conditionally invokes a client string helper based on client field +0x62c. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1e610` | `PushKeybindingUpdateResult → ChangeKeyBinding` |  | Updates a keybinding entry and returns the helper success value. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1ef10` | `FormatDurationString → LuaGetElapsedTimeString` |  | Returns a formatted duration string. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c21a90` | `RefreshCurrentClientLuaParams` |  | Refreshes the Current Lua data exposed to script consumers. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c253f0` | `GfxCtrl_SetProperty` |  | Sets a property through the GfxCtrl binding. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25710` | `GfxCtrl_GetProperty` |  | Reads a named property through the GfxCtrl Lua binding. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25d70` | `DsTable_GetRowCount` |  | Returns the data-source table row count to Lua. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25dc0` | `DsTable_GetColumnCountAtIndex` |  | Returns the indexed table count under the Lua getColumnCount(index) binding. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25ee0` | `DsTable_FindRowByUserData` |  | Finds a row by its userData field. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c26800` | `ImageControl_ApplyPairOperation5 → ImageControl_SetSize` |  | Applies a two-number operation through a shared ImageControl helper; selector five meaning is inferred. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c26850` | `ImageControl_ApplyPairOperation6 → ImageControl_SetPosition` |  | Applies a two-number operation through a shared ImageControl helper; selector six meaning is inferred. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟡 speculative (100)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009769f0` | `ClientLuaCounterIsNonZero ?` |  | Reads the dword at +0x1ac of the global internals pointer DAT_01be51a8 and returns whether it is nonzero (bool in eax), inside the SEH and stack-cookie prolog and epilog. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976a60` | `ClientLuaCounterIncrement ?` |  | Increments the dword at +0x1ac of the global internals pointer DAT_01be51a8 by one, inside the SEH and stack-cookie prolog and epilog. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00976ad0` | `ClientLuaCounterDecrement ?` |  | Decrements the dword at +0x1ac of the global internals pointer DAT_01be51a8 by one, inside the SEH and stack-cookie prolog and epilog. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00977140` | `TaggedValue160ArrayAppendRaw ?` |  | Appends one 0xa0-byte slot to the array at this+4 (count at +8, capacity at +0xc, growth through FUN_00713cf0 when count+1 exceeds capacity), increments the count, zeroes the slot's first dword and returns the slot poin… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009773f0` | `TaggedValue160ArrayAppendInt ?` |  | Appends one 0xa0-byte slot to the array at this+4 with the same growth as 00977140, then stores tag 1 in its first dword and copies the dword *param_2 into slot+8. Returns the slot pointer, or 0 when the slot is null. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00977480` | `TaggedValue160ArrayAppendDouble ?` |  | Appends one 0xa0-byte slot with the same growth as 00977140, stores tag 2 in its first dword and copies the 8-byte value *param_2 into slot+8. Returns the slot pointer, or 0 when the slot is null. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00977510` | `TaggedValue160ArrayAppendPair ?` |  | Appends one 0xa0-byte slot with the same growth as 00977140, stores tag 0 in its first dword, and passes the two dwords of *param_2 to FUN_00707240 with the slot's storage as destination. Returns the slot pointer. If th… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009775d0` | `TaggedValue160ArrayAppendBool ?` |  | Appends one 0xa0-byte slot with the same growth as 00977140, stores tag 5 in its first dword and copies the byte *param_2 into slot+8. Returns the slot pointer, or 0 when the slot is null. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00977c10` | `TaggedValue160ArrayAppendCopy ?` |  | Appends one 0xa0-byte slot to the array at this+4 (count at +8, capacity at +0xc, growth through FUN_00713cf0), sets the slot tag dword to 0, and assigns the array argument into the slot with FUN_0070a6d0 (ArrayAssign, … | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00978e20` | `TaggedValue160ArrayAppendAssignC620 ?` |  | Appends one 0xa0-byte slot to the array at this+4 (count at +8, capacity at +0xc, growth through FUN_00713cf0), sets the slot tag dword to 0, and assigns the argument into the slot with FUN_0070c620 (called with the slo… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979450` | `TaggedValueEquals ?` |  | Returns false when the tags differ. With equal tags: 1 compares the ints at +8 (0 when absent), 2 compares the doubles, 3 and 8 compare the strings through the byte-string comparator 00977030 (truthy result), 4 compares… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00979ac0` | `TaggedValue160ArrayAppendAssignC118 ?` |  | Appends one 0xa0-byte slot to the array at this+4 (growth through FUN_00713cf0 when count+1 exceeds capacity), zeroes its tag dword, and assigns param_2 into the slot with FUN_007118a0 (called with the slot as this). Re… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00bfe7e0` | `SocialShare::Callback ? ⚠downgrade` | SocialShare | Forwards a social-share Lua callback to the selected provider integration. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfea90` | `ParseLuaUnsignedGuid ? ⚠rename` |  | Converts a Lua string into a nonnegative 64-bit identifier used by several callback handlers. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfecb0` | `Factory::RequestEquipTool ? ⚠rename` | Factory | Requests that a factory tool be equipped. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff060` | `GetManagerValuePair ?` |  | When the manager exists, calls 0097c8c0 with a local output integer; on nonzero result pushes its return value and that integer and returns two. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff150` | `GetManagerValuePair ?` |  | When the manager exists, calls 0097c910 with a local output integer; on nonzero result pushes its return value and that integer and returns two. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfffe0` | `CallHelperAndPushArgumentResult ? → CoinStoreScreen::SelectItemCategory` |  | Requires one Lua argument and an available manager, calls 00c6f210 with that argument, pushes the result, and returns one. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00050` | `LookupAndPushManagerValue ?` |  | Requires one Lua argument and an available manager, passes its value to 00c703f0, pushes the returned value, and returns one. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c000c0` | `CallHelperAndPushArgumentResult ? → CoinStoreScreen::SelectInventoryCategory` |  | Requires one Lua argument and an available manager, calls 00c6f250 with that argument, pushes the result, and returns one. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01060` | `ApplyVectorAndRadius ?` |  | Requires three numeric Lua arguments, reads three float values and optional radius and boolean, calls 008c6110 with those values, then pushes its result. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01b90` | `CreateLuaClosureWithCapturedValue ?` |  | Allocates four bytes in the Lua state, stores param_2 in that slot, creates a closure using the registry key and upvalue -2, then returns the allocated slot. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01bd0` | `GetLuaDataSourceObject ?` |  | Sets a Lua stack top, looks up the named registry value, returns zero if absent, and otherwise dereferences its object pointer; logs an error when that pointer is null. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01c60` | `CallLuaDataSourceMethodTrue ?` |  | Fetches the registered data-source object, clears its registry entry if missing, otherwise calls vtable offset 0x14 with argument 1, then returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01ca0` | `CallLuaDataSourceMethodFalse ?` |  | Fetches the registered data-source object, clears its registry entry if missing, otherwise calls vtable offset 0x14 with argument 0, then returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01e60` | `IsLuaGuiDataSourceValue ? ⚠rename` |  | Checks whether a Lua value references a GuiDataSource and GuiDsValue object. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c026e0` | `DispatchSeventeenLuaValues ? → InGamePurchase::AddNewCreditCard` |  | Validates 17 Lua arguments and converted values, checks manager and required nonzero fields, calls 00b33bc0 with the validated values, then pushes its result. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03cf0` | `CallHelperWithTwoIntegers ? → SelectStoreBundleLuaCallback` |  | Requires two numeric Lua arguments and an active manager, reads both integers, calls 00b37850, pushes one on success and zero otherwise. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03d80` | `CallHelperWithInteger ?` |  | Requires one numeric Lua argument and an active manager, reads the integer, calls 00b23cb0, and pushes one on success or zero otherwise. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03e60` | `CallHelperWithTwoValues ?` |  | Requires two numeric Lua arguments and an active manager, extracts both values, calls 00b37b90, and returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03ed0` | `CallHelperWithThreeValues ?` |  | Requires three numeric Lua arguments and an active manager, extracts all three values, calls 00b3b3b0, and pushes zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c05170` | `ForwardNonemptyLuaString ?` |  | For one Lua argument, reads a string and if it is non-null and nonempty passes it to 008ef350 after obtaining the global manager; returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c051e0` | `ForwardLuaInteger ?` |  | For one Lua argument, converts it to an integer, obtains the global manager, calls 008ef510 with the value, and returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c05250` | `ForwardLuaInteger ?` |  | For one Lua argument, converts it to an integer, obtains the global manager, calls 008ef740 with the value, and returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c05510` | `LookupValueForLuaString ?` |  | For one nonempty Lua string, obtains the manager, passes the string to 008a8700, pushes its result, and returns one; otherwise pushes -1. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c17600` | `InvokeWildcardArgForwarder ? → ResolveRecordAndPassField50FromWildcardArg` |  | Forwarder to an otherwise unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c176a0` | `SetClientFlag658FromArg ? → SetClientBooleanField658FromBooleanArg` |  | Writes a client boolean field through a Lua argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c176e0` | `SetClientFlag659FromArg ? → SetClientBooleanField659FromBooleanArg` |  | Writes a client boolean field through a Lua argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17750` | `InvokeTwoFloatArgs ?` |  | Forwards a pair of numeric arguments to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c177e0` | `InvokeTwoFloatArgs2 ?` |  | Forwards two numeric arguments to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17860` | `InvokeTwoFloatArgs3 ?` |  | Forwards a pair of numbers to an unlabeled guarded client call. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c178f0` | `InvokeOneFloatArg ?` |  | Forwards a numeric value to an unlabeled client call. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17960` | `SetVectorFromArgs ? ⚠rename` |  | Updates a client value and replaces a related object; destination semantics remain unclear. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17a10` | `InvokeIndexedCallbackA ?` |  | Indexed callback adapter; destination feature is not established. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17a60` | `InvokeIndexedCallbackB ?` |  | Indexed callback adapter; destination feature is not established. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17bb0` | `RequestPointOfInterestTeleport ?` |  | Lua-facing adapter for the client point-of-interest teleport request. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17d30` | `InvokeFourFloatArgs ?` |  | Numeric argument adapter to an unlabeled four-value routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17df0` | `RequestGroupMapPing ?` |  | Lua-facing adapter for the accepted group map ping packet builder. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18310` | `SetClientFlagFromBoundedValue ? ⚠rename` |  | Updates client state through an unlabeled helper after a numeric range check. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c184f0` | `SetOutputFromSelectedValue ? → ReturnClientField474Or478OrSelector` |  | Argument/output adapter; the selected fields are owned by an unlabeled callee. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c185f0` | `SetOutputFromPairField ? → ReturnClientPairField47COr480` |  | Reads a selected value from a client pair through an unlabeled helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18840` | `SetOutputFromBooleanizedValue ?` |  | Maps one converted argument to an output boolean through an unlabeled helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c188f0` | `RequestStripEffect ?` |  | Calls the accepted client strip-effect packet sender from an argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18940` | `GetTwoClientValues ? → ReturnClientStateFields32CAnd328` |  | Returns two values from an unlabeled client helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c189a0` | `RequestLeaderboard ?` |  | Invokes the accepted leaderboard request path from an argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c189e0` | `InvokeTwoClientValues ? → RequestEncounterRunningListAndClearEntries` |  | Two-value adapter to an unlabeled client helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18aa0` | `StartQuestBreadcrumb ?` |  | Invokes the quest breadcrumb helper; feature attribution relies on the callee's accepted label. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18b10` | `GetOutputFromValueA ? → GetOrCreateAssociatedValueForKey` |  | Returns a transformed client value through an argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18b70` | `InvokeTwoClientValuesA ? → SetAssociatedValueForKey` |  | Two-argument adapter to a client routine; exact domain not established. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18be0` | `GetOutputFromValueB ? ⚠rename` |  | Returns a transformed client value through an argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18c80` | `SelectPlayerTitle ?` |  | Invokes the accepted player-title selection request routine from a Lua argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18d00` | `SetJobCustomizationSlot ?` |  | Calls the accepted job-customization slot setter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18d70` | `ClearJobCustomizationSlot ?` |  | Calls the accepted job-customization slot clear routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18de0` | `SelectJobCustomizationRow ?` |  | Calls the accepted job-customization row selection routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18e30` | `GetOutputFromClientValueA ? ⚠rename` |  | Typed argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18e80` | `GetOutputFromClientValueB ? ⚠rename` |  | Typed argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c192e0` | `GetThreeDefaultValues ? ⚠rename` |  | Returns three values through a Lua-facing output adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19350` | `SetClientFlagFromArg ?` |  | Stores an argument-derived boolean in client state. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19600` | `SetQuickChatEmoteFavoriteMode ? ⚠rename` |  | Touches the quick-chat emote favorites path; exact state changes are in the callees. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19650` | `SetQuickChatEmoteFavoriteMode2 ? ⚠rename` |  | Touches the quick-chat emote favorites path; exact state changes are in the callees. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19a10` | `ApplyOneOrTwoClientValues ? ⚠rename` |  | Argument adapter to an unlabeled client operation. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19f60` | `ForwardWildcardArgumentA ?` |  | Argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19fa0` | `ForwardWildcardArgumentB ?` |  | Argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1a170` | `SetClientFlagFromPositiveArg ?` |  | Stores an argument-derived boolean in client state. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1a460` | `RegisterGetNameBinding ?` |  | Registers or resolves a GetName-style binding through a generic helper; exact binding framework role remains tentative. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1ad60` | `GetClientPairAndStoreOutput ? → PushFormattedClientRecordPair` |  | Returns a pair-derived value through an unlabeled output helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1aef0` | `GetTwoPositionCoordinates ?` |  | Returns a pair of position coordinates from an unlabeled client helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b160` | `SendMemberQuitFromArgs ?` |  | Lua adapter to the activity-launch MemberQuit sender. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1bff0` | `GetOutputStringFromClientResource ?` |  | Returns a client resource string when the resource state is available. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c1d0` | `LookupStringForMappedClientCode ?` |  | Looks up a string for a mapped client code; the virtual method and the meaning of the output helper remain unidentified. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c2a0` | `LookupClientRecordStringA ?` |  | Returns a textual value from a client record lookup. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c3a0` | `LookupClientRecordStringB ?` |  | Returns a textual value from a client record lookup. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c4a0` | `ConvertClientValueToString ? ⚠rename` |  | Returns a converted client value as a string. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c790` | `GetWideStringArgumentA ? ⚠rename` |  | Forwards a wide-string argument to an unlabeled client method. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1c840` | `GetWideStringArgumentB ? ⚠rename` |  | Forwards a wide-string argument to an unlabeled client method. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1cd20` | `CallStringHelperWithFourArguments ? → GetDateString` |  | Lua adapter around a client string helper; no evidence in this function identifies a SoundDefinition class. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1ce60` | `CallStringHelperWithOneArgument ? → GetDayOfWeek` |  | Lua adapter around a client string helper; no evidence in this function identifies a SoundDefinition class. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1cf70` | `CallStringHelperWithTwoArguments ? → GetDateMonthYear` |  | Lua adapter around a client string helper; no evidence in this function identifies a SoundDefinition class. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1d3c0` | `CreateGameSceneObject ? ⚠rename` |  | Creates and registers a named object after calling the GameScene virtual method; the effect of that parent callback is not identified. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1d4c0` | `CopyArgumentStringToOutput ? ⚠rename` |  | Copies an argument-provided string into an output string value. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1de10` | `InvokeClientVirtualWithArguments ? → Translate` |  | Generic variadic client callback wrapper; exact virtual method meaning is unknown. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1e090` | `InvokeClientVirtualWithOneValue ? ⚠rename` |  | Single-value client callback wrapper; target virtual method meaning is unknown. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1e400` | `InvokeClientVirtualWithValueToString ? ⚠rename` |  | Returns a string produced by an unlabeled client virtual method. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1e720` | `SendQuickChatWhisper ? → ProcessQuickTellCommand` |  | Invokes the quick-chat whisper path, which builds and sends a tell packet. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1f0a0` | `InvokeFieldAdjustedClientMethod ? → LuaOnObjectiveEvent` |  | Forwards a string argument to a client method through an adjustor thunk. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c24f90` | `GfxCtrl_Action ?` |  | Lua binding for a GfxCtrl operation; its method name is unresolved. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25100` | `GfxCtrl_Action ?` |  | Lua binding for an unidentified GfxCtrl operation. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25330` | `GfxCtrl_ApplyFlags ? ⚠rename` |  | Exposes a flag query and conditional action through GfxCtrl userdata. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25e60` | `DsTable_EnableFlag ?` |  | Sets a boolean-like DsTable option to enabled. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25ea0` | `DsTable_DisableFlag ?` |  | Sets a boolean-like DsTable option to disabled. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## ⚪ unknown (52)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ebdd0` | `— ⚠reject` | SoeUtil::Array<ClientLuaParam,0,1> | Copies the source ClientLuaParam elements into the destination array after clearing its current active elements. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `00977dc0` | `— ⚠reject` |  | Switches on the tag dword at this: for tags 3, 6, 7 and 8 it calls the function pointer stored at this+8 with argument 0. It then stores 8 in the tag dword, and when this is not the sentinel 0xfffffff8 it calls FUN_0070… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00978ab0` | `— ⚠reject` |  | Initialises the IString param_2 as empty, then formats the tagged value *param_1 into it by tag: 0 assigns a constant string, 1 calls FUN_00821d00 on the int, 2 calls FUN_00978730 on the double, 3 formats the static ISt… | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009c51d0` | `— ⚠reject` | ExecuteLuaEvent | Constructs an ExecuteLuaEvent holding Lua callback parameters. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5350` | `— ⚠reject` |  | Creates and submits a Lua timer event using timer callback data. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `00bfed30` | `— ⚠reject` |  | Reads Lua argument one, converts it through 009e1a50 to a boolean, calls 00e3aa30 with that value, and returns one. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff6f0` | `— ⚠reject` | Mount | Spawns and automounts the selected mount, or refreshes the existing mount callback when a mount is already active. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bffa60` | `— ⚠reject` | Mount | Requests dismount or updates the mount button state according to current mount state. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bffda0` | `— ⚠reject` |  | Requires two truthy Lua arguments and an available manager, calls 00c70fe0 with both values, pushes the boolean result, and returns one. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfff70` | `— ⚠reject` |  | Requires one Lua argument and an available manager, calls 00c6f200 with that argument, pushes the result, and returns one. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01660` | `— ⚠reject` |  | Validates arguments, parses two 64-bit identifiers and optional scalar or boolean arguments, builds two temporary target objects, calls 008c6170, and cleans them up. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c02a50` | `— ⚠reject` |  | Validates 16 Lua arguments, converts numeric and boolean fields, checks required values and manager, calls 00b33e60, then pushes its result. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c02e50` | `— ⚠reject` |  | Validates five Lua arguments and the manager, extracts two strings and three numeric values, calls 00b37670, and pushes its result. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c02f60` | `— ⚠reject` |  | Validates Lua arguments one and three, reads them as values, optionally reads integer arguments two and four, checks the manager, then calls 00b33900. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03520` | `— ⚠reject` |  | For one truthy Lua argument, obtains its value and calls 00b37b80 when the manager is present; returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03860` | `— ⚠reject` |  | For one truthy Lua argument, reads its string and calls 00b20100 when both the string and manager are present; returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c038b0` | `— ⚠reject` |  | For one truthy Lua argument, reads its string and calls 00b20100 when both the string and manager are present; returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c039f0` | `— ⚠reject` |  | Reads one Lua 64-bit identifier, compares it to a global identifier, and when the active account object has the expected low flag and manager, calls 00b376d0; pushes the result. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03df0` | `— ⚠reject` |  | Requires one numeric Lua argument and an active manager, reads the integer, calls 00b37a10, and pushes one on success or zero otherwise. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c03f70` | `— ⚠reject` |  | Requires one string argument and active manager, copies the string to a temporary object, calls 00b37bb0 with the resulting value and zero, then pushes its result. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c04500` | `— ⚠reject` |  | Requires five numeric Lua arguments and a manager, checks a purchase record with 00c03ba0, invokes 00b2a4d0, calls a vtable method with three values, formats a log line, and pushes success. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c04bc0` | `— ⚠reject` |  | For one truthy Lua argument, reads its boolean value and stores whether the corresponding integer is nonzero at global object offset 0x18; returns zero. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c04c10` | `— ⚠reject` |  | For one nonempty Lua string, converts it to a temporary wide-string representation, calls 00995780 with that result, then destroys temporary data. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c17390` | `— ⚠reject` |  | Generic Lua argument adapter; the callback key identifies the scavenger-hunt combo data path. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c174a0` | `— ⚠reject` |  | Calls the accepted mini-game creation request builder; feature attribution follows that callee. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c174e0` | `— ⚠reject` |  | Calls the accepted mini-game creation request builder with its alternate flag. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17520` | `— ⚠reject` |  | Calls the accepted mini-game group creation request builder. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17570` | `— ⚠reject` |  | Calls the accepted mini-game group request builder. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c175c0` | `— ⚠reject` |  | The destination's exact gameplay role is not established from this wrapper alone. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17640` | `— ⚠reject` |  | Guarded forwarder; destination behavior is not inferred here. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17af0` | `— ⚠reject` |  | Forwards an argument-derived boolean to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c17b70` | `— ⚠reject` |  | Forwards an argument-derived boolean to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c186a0` | `— ⚠reject` |  | Applies two numeric arguments through an unlabeled client helper. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18a60` | `— ⚠reject` |  | Sets one client boolean field through a Lua argument adapter. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c18fc0` | `— ⚠reject` |  | Two-value adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19030` | `— ⚠reject` |  | Two-value adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c190a0` | `— ⚠reject` |  | Typed argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c190f0` | `— ⚠reject` |  | Lua adapter enters a client subobject adjustor thunk; raw call bytes do not support the decompiler claim that the converted ESI value is its pushed argument. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19140` | `— ⚠reject` |  | Two-value adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c191b0` | `— ⚠reject` |  | Two-value adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19220` | `— ⚠reject` |  | Typed argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19270` | `— ⚠reject` |  | Typed argument adapter to an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c19820` | `— ⚠reject` |  | Passes a three-component value into an unlabeled client routine. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b3d0` | `— ⚠reject` |  | Lua-bound UI entry for closest interaction. The cited note directly maps this binding to the address; its code reads the cached interaction state and calls the interaction helpers. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1b960` | `— ⚠reject` |  | Returns a nonnegative remaining-time difference for the matching client list entry. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1cbb0` | `— ⚠reject` |  | Looks up a record and exposes four of its fields; this wrapper does not establish that the record is a SoundDefinition. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1ec70` | `— ⚠reject` |  | Five-argument callback adapter to an unlabeled client virtual method. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c25a90` | `— ⚠reject` |  | Passes a parameter array through GfxCtrl userdata to a native operation. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c25d20` | `— ⚠reject` |  | Returns the data-source table column count to Lua. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c263d0` | `— ⚠reject` |  | Serializes a data-source table for Lua consumers. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c267a0` | `— ⚠reject` |  | Validates and unwraps ImageControl userdata. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c268a0` | `— ⚠reject` |  | Sets a property on ImageControl through the Lua interface. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00bffbc0` | `Mount::Upgrade` | send | MountBasePacket |  | Hypothesis: opcode 168/sub-opcode 14 carries a mount upgrade request with these two identifiers. No server implementation evidence is present. |
| `00bff8e0` | `Mount::Despawn ⚠downgrade` | send | PacketDismountRequest |  | Hypothesis: the client requests a mount dismount using mount opcode 168/sub-opcode 3; the server's state transition is unknown. |
| `00c1af80` | `SetQuickChatChannelNinePair` | send | QuickChatSendChatToChannelPacket |  |  |
| `00c17bb0` | `RequestPointOfInterestTeleport ?` | send | PacketZoneTeleportRequest |  |  |
| `00c188f0` | `RequestStripEffect ?` | send | PlayerUpdatePacketRequestStripEffect |  |  |
| `00c18c80` | `SelectPlayerTitle ?` | send | PlayerTitleRequestSelectPacket |  |  |
| `00c1e720` | `SendQuickChatWhisper ? → ProcessQuickTellCommand` | send | QuickChatSendTellPacket |  | Hypothesis: server receives a tell packet with target and message data; inference depends on 00b976a0 and QuickChatSendTellPacket serialization, not original server logic. |
| `00bff6f0` | `— ⚠reject` | send | PacketDismountRequest |  | Hypothesis: when active mount state requires it, client sends dismount before spawning the new mount. This depends on the two locally constructed packets and does not establish server acceptance rules. |
| `00bff6f0` | `— ⚠reject` | send | PacketMountSpawn |  | Hypothesis: when active mount state requires it, client sends dismount before spawning the new mount. This depends on the two locally constructed packets and does not establish server acceptance rules. |
| `00bffa60` | `— ⚠reject` | send | PacketDismountRequest |  | Hypothesis: the client sends mount opcode 168/sub-opcode 3 to request dismount; original server behavior is not available. |
