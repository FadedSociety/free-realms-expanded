# System: ui

61 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0089aac0` | `GuiResetter::~GuiResetter` | GuiResetter | Base destructor of the GUI reset helper instantiated in multiple TUs (two unwind funclets cite this emission). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `0089aad0` | `GuiResetterScalarDtor` | GuiResetter | Deleting destructor of the GUI reset helper. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `0089cfd0` | `GuiUserDataTable::~GuiUserDataTable` | GuiUserDataTable | Destruction of the GUI user-data table object (Scaleform-side data container, per the Gui prefix). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008a6e90` | `InitTooltipComponents` |  | Builds the tooltip/ability-effect component set of the client object: the two ability tooltip tables are created from the Tooltips.ActiveAbilityEffects and Tooltips.PassiveAbilityEffects data-source names, and every com… | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a77a0` | `BaseClient::ShowZoneServerHostingStatus` | BaseClient | BaseClient vtable slot 4 shared with PlayClient; emits the hosting status through a callback. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a7af0` | `BaseClient::DispatchOnUpdateAction` | BaseClient | BaseClient vtable slot 49 shared with PlayClient; fires OnUpdate and then executes the two trailing update calls. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a7b30` | `OptionsShowOrContextQuit` |  | Decides between reopening the options screen and quitting the current context based on a context-depth setting and two component pointers; the only caller is 00902cc0. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a8bf0` | `CallLua_RespawnWindow_Hide` |  | Client-side helper that programmatically hides the respawn window through the original Lua UI surface; reached from 0092f460. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `00985200` | `BossesDataSource_RegisterGuid` |  | Client-side registration of a boss for the target-health bar; per the vault note Native Loss Boss Display it is invoked (together with setting the actor's boss bit 0x80) by the CombatPacketEnableBossDisplay handler 00a2… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `009852a0` | `BossesDataSource_UnregisterGuid` |  | Client-side unregistration of a boss from the target-health bar registry; per the vault note Native Loss Boss Display it is called by the CombatPacketEnableBossDisplay handler 00a2b910 after the actor's boss bit 0x80 is… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00ab91b0` | `HousingProcessor::NotifyNewHouse` | HousingProcessor | Subtype 49 handler per the DispatchMessage label: shows a new-house notification, preferring the script function HousingNotifyNewHouse and falling back to a formatted message. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00ab94b0` | `HousingProcessor::SetIsInInstance` | HousingProcessor | Pushes the instance-membership flags of the zone to the Lua-facing housing state; called by the zone-data handler HandleZoneData (00ac36c0, accepted label). | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00ab96e0` | `CuteModeSetSizeScriptCall` |  | Pushes a cute-mode size change to the script layer; called from the subtype 50 handler 00abb050. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00ab9860` | `HousingProcessor::HandleMaxFixtures` | HousingProcessor | Subtype 53 handler per the DispatchMessage label: forwards the fixture-limit message to the script layer and falls back to a fixed placement-denied text. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00abaca0` | `HousingProcessor::HandleInvitationDeclined` | HousingProcessor | Subtype 48 handler: notifies the Lua-facing housing callback using the invited_player field key; the passed value?s meaning and whether it represents a player name are unresolved. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00abc0d0` | `ApplyHousingSearchReply` |  | Replaces the housing search result rows with the reply rows and refreshes the data source when the update flag is set. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |

## 🟢 likely (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a7500` | `InitComponentSubset` |  | Likely initializes a reduced set of the same component family as 008a6e90: nine constructors and the +0x5dc..+0x63c stores overlap. The sole caller 00715ec0 is not traced, so ownership/context is open. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008ba020` | `ReleaseUiModuleReferences` |  | Unbinds and releases the client's UI module references. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00984f00` | `GuiDataSourceCastCheck` |  | Checks a data-source runtime type and optionally formats a diagnostic after a failed cast. The final reporting call is a stub and emits nothing; no owning class or specific UI use is established by this body. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00985190` | `DataSourceManager_FindBossesByName` |  | Resolves the UI data source named 'BaseClient.Bosses' to its BossesDataSource object; the callers 00985200/009852a0 pass the global manager pointer from 0x01be1090 in ecx (visible in their bytes) and then refresh the da… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00b73d80` | `MeasureMaxLineWidth` |  | Text measuring helper (longest line length x10) used by the soccer UI code path FUN_00b7e910. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00c14e70` | `GetLastClickedScreenLocation` |  | Returns the two screen coordinates of the last click for scripts (the global script table row 0x0182e048 points at this address). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14f20` | `BindStringValue` |  | Binds a UI string value to a script-visible name so the UI can be refreshed when the value changes. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14f70` | `UnbindStringValue` |  | Removes a previously created string-value binding. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15290` | `GetCameraHeading` |  | Returns the camera heading angle (the atan2 of the camera direction vector) to scripts. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15380` | `OnMiniGameFeedback` |  | Script callback for mini-game feedback; in this build the body performs only the argument check, so it has no observable effect beyond validation. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c153b0` | `OnBugReportUiSelectCategory` |  | Bug-report UI callback that selects a report category. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15470` | `SetCollectionFilterByCategory` |  | Sets the collection-journal filter to a category. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15c90` | `SetHeadSize` |  | Script hook that scales the head-size option; the name is the table row 0x0182de30 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00ce43e0` | `IconDataDerivedCtor` |  | Constructs an IconData-based icon object and loads its two text members from an optional source record. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce45e0` | `IconDataDerivedCopyCtor` |  | Copy constructor for the IconData-derived element used by the element array reallocation in 00ce46a0. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce4b60` | `IconDataElemCreate` |  | Creates the IconData-derived element at an index of its array and constructs its members. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## 🟡 speculative (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099d2d0` | `SetCurrentSelection ? ⚠rename` |  | Selection or mode update inferred from the caller context; this function has no feature-specific anchor. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00a5c8b0` | `ReleaseLoadingScreenMovieAndInitData ?` |  | Reset loading-screen movie state and ensure its data object exists. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00ad5f60` | `PortraitDataCtor ?` |  | Constructs the attachment list and six fixed strings, initializes counters and dimensions to 300 by 300, and assigns the literal Portrait to its label string. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad6070` | `PortraitDataDtor ?` |  | Destroys six fixed strings and the embedded attachment list, and is reached from exception cleanup paths. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ce7420` | `ParseUiColorHex ?` |  | Builds a temporary string, parses it as base 16 with strtoul, ORs the value with 0xff000000, and stores it at +4. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cfa510` | `MessageDataSource::GetColumnName ?` | MessageDataSource | Provides column labels for the message datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa5d0` | `BuildMessageDatasourceRows ?` |  | Builds rows for the MessageDataSource when invoked by its methods. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa770` | `MessageDataSource::GetRowCount ?` | MessageDataSource | Returns the number of rows in the message datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa790` | `MessageDataSource::MessageData::Init ?` | MessageDataSource | Initializes a message record string field. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa860` | `MessageDataSource::MessageData::ScalarDeletingDtor ?` | MessageDataSource | Scalar deleting destructor for a message record. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa8c0` | `CopyMessageRecord ?` |  | Copies a message datasource record. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa940` | `MessageDataArray::Reallocate ?` |  | Grows or replaces storage for message datasource records. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfab80` | `MessageDataArray::AppendDefault ?` |  | Appends one initialized record to the message datasource array. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfac10` | `MessageDataSource::AddMessageRecord ?` | MessageDataSource | Adds or initializes one row in the message datasource array. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfad00` | `MessageDataSource::Cleanup ?` | MessageDataSource | Cleans up the message datasource and its owned records. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfade0` | `MessageDataSource::MessageDataSource ?` | MessageDataSource | Constructs the message datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfae80` | `MessageDataSource::ScalarDeletingDtor ?` | MessageDataSource | Scalar deleting destructor for MessageDataSource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfaea0` | `MessageDataSource::GetCell ?` | MessageDataSource | Supplies message datasource cell values. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a8c40` | `— ⚠reject` |  | Hides the main HUD surfaces when either client-state probe reports true (a clean-screen/leave-state path); the domain rests on the callee 00ad11d0's Lua method names. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `009a0a60` | `— ⚠reject` |  | Formats a price/progress display using record fields at offsets 0x78 through 0x80. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00abc670` | `— ⚠reject` |  | Registers a house name entry (thumbnail or Houses table) with the UI; DispatchMessage calls it for subtype 0x11 per its label, and the name-request sender 00ac4e90 also uses it. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00ad3790` | `— ⚠reject` |  | When offset +0x14 is nonzero, passes the character GUID at manager offsets +0x3a0/+0x3a4 to 00b4c350 and forwards the resulting pair and DAT_0181a34c to 010d23f0 when successful. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00c150c0` | `— ⚠reject` |  | Tells the radial menu system which item the cursor is over, with its screen coordinates. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15120` | `— ⚠reject` |  | Tells the radial menu system that the cursor left an item. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c151a0` | `— ⚠reject` |  | Returns the player's world position (x, y, z) to scripts; the alternate branch taken when FUN_008a7d50 succeeds is not recovered by the decompiler. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c15400` | `— ⚠reject` |  | Bug-report UI callback that submits the report (category, description and two further text fields). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16df0` | `— ⚠reject` |  | Reports the argument (a player name or message) as offensive through the moderation path; the name is the table row 0x0182e628 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00cfaab0` | `— ⚠reject` |  | Gets or creates a message datasource row record. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfac80` | `— ⚠reject` | MessageDataSource | Updates the message datasource after a message record removal. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ab91b0` | `HousingProcessor::NotifyNewHouse` | recv | BaseHousingPacket |  |  |
| `00ab9860` | `HousingProcessor::HandleMaxFixtures` | recv | BaseHousingPacket |  |  |
| `00abaca0` | `HousingProcessor::HandleInvitationDeclined` | recv | BaseHousingPacket |  |  |
