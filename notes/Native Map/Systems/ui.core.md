# System: ui.core

45 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0071bcd0` | `LookupLocalizationKey` |  | Passes a localization key to the client interface lookup method; whether the returned or updated text is retained is not shown. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071bd80` | `LookupLocalizationKey` |  | Passes a localization key to the client interface lookup method; whether the returned or updated text is retained is not shown. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `00902320` | `NotifyErrorMessage` |  | Sends an error message string to the Lua notification handler NotificationHandler:ErrorMessage. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `009023f0` | `NotificationErrorOrAlert` |  | Shows an error or alert notification through the Lua NotificationHandler, choosing ErrorMessage or AlertMessage by param_3. Caller PlayClient::HandleWorldShutdownNotice (0090d990) carries that label. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `009025a0` | `ShowDisconnectedPopup` |  | Shows the 'popupDisconnected' error popup through the Lua HUD:ShowErrorMessage handler; the sole static caller is 00904560. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `009046e0` | `HudSetIsInStartingZone` |  | Sends the starting-zone flag (byte argument) to the Lua HUD handler HUD:SetIsInStartingZone. Callers include PlayClient::SetStartingZoneState (0090e8a0), WaitForWorldReady (00936d00) and BeginPlayerZoning (0093d180). | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00904ab0` | `ShowTooltipFromCode` |  | Shows a tooltip through Lua TooltipMethods:ShowFromCode, with the constant tooltip type 0xe. Caller 00963dd0 is unlabelled. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00904bf0` | `HideTooltipFromCodeGated` |  | Hides the code-shown tooltip through Lua TooltipMethods:HideFromCode only when the tooltip slot holds tag 0xe. Callers 00961730 and 00963dd0 are unlabelled. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00904ce0` | `ShowTooltipRowsFromCode` |  | Builds tooltip rows for a named entry (FUN_00a78ca0) and shows them through Lua TooltipMethods:ShowFromCode with tooltip type 0x10. The caller 00963dd0 is unlabelled. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00904e70` | `HideTooltipFromCode` |  | Hides the code-shown tooltip through Lua TooltipMethods:HideFromCode. Callers 00961730 and 00963dd0 are unlabelled. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00a19fc0` | `ClientNameChangeStatusHandler` |  | Updates client name-change status and reports it through the UI callback path. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a8eff0` | `FlashTimerSetTimer` |  | Sets or extends the displayed Flash timer deadline. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a95cd0` | `UpdateFlashTimerAndClientUi` |  | Per-frame FlashTimer update followed by notification, minimap-related and click-to-move UI work. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00bf6310` | `NotificationImage::Ctor` | NotificationImage | Constructs one NotificationImage (constructed from NotificationImage wrapper 00bf6600). Base vtable first, most-derived vtable last, the MSVC constructor order. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00bf63d0` | `NotificationImage::DtorBody` | NotificationImage | Destructor body of NotificationImage; called by the scalar deleting destructor 00bf6680 and by EH unwind funclets. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00bf6680` | `NotificationImage::ScalarDeletingDtor` | NotificationImage | Deleting destructor of NotificationImage (vtable slot [0]). | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00c1e7f0` | `UiLogValueWithCategoryLevel5 → LogDebug` |  | Logs one UI value at level 5 under the default or supplied category. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1e910` | `UiLogValueWithCategoryLevel4 → LuaLogInfo` |  | Logs one UI value at level 4 under the default or supplied category. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1ea30` | `UiLogValueWithCategoryLevel3 → LuaLogWarning` |  | Logs one UI value at level 3 under the default or supplied category. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c1eb50` | `UiLogValueWithCategoryLevel2 → LuaLogError` |  | Logs one UI value at level 2 under the default or supplied category. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `0123a140` | `UI::UIServiceBase::DtorBody` | UI::UIServiceBase | Destructor body of the UI service base class; it tears down the member containers and the string. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123a270` | `UI::UIServiceBase::Ctor` | UI::UIServiceBase | Constructor of the UI service base class; initialises its containers and tree header. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123a320` | `UI::UIServiceBase::ScalarDeletingDtor` | UI::UIServiceBase | Vtable slot [0] of UI::UIServiceBase: the scalar deleting destructor. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004250ce` | `ResetEntriesAndPruneByFlag` |  | Prunes vector entries based on their flag byte, then invokes and releases a separate callback object. The UI-control context is inferred from neighboring methods; exact entry and callback types remain unknown. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `0042520f` | `RemoveByKeyPair` |  | Removal of a (short id, dword key) pair from the owner's element vector; single caller 00648dc8. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `0042528e` | `RemoveAllByKey` |  | Removes all elements belonging to one key from the owner's vector. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `004252f9` | `DispatchInternalEvent` |  | Dispatches either of two internal UI event ids; their meaning is inferred from the tooltip/control callers and remains specific to that event path. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `00904300` | `DisplayRespawnWindow ⚠downgrade` |  | Shows the respawn window through the Lua handler RespawnWindow:DisplayRespawn, passing a time value in seconds. The research note Research/Pieces/058 Death and revive cites this address as the display handler. Caller Pl… | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `009d0fa0` | `PositionGuiLabelAndReturnTextWidth ⚠downgrade` |  | Updates and lays out a GUI label using a measured string width. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009f8dc0` | `FindGuiEditCtrlByName` |  | Retrieves an edit control from the GUI object lookup. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f8e10` | `FindGuiWindowCtrlByName` |  | Retrieves a window control from the GUI object lookup. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00a86fa0` | `UIProcessorCreateObject0x34` | UIProcessor | Creates the object requested through the UIProcessor factory interface; the constructed type remains unresolved. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟡 speculative (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fa110` | `GetGuiComboBoxSelectedValue ? ⚠downgrade` |  | Reads the selected value from a named combo box. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00bf6570` | `NotificationImage::VirtualRegisterChild ? ⚠rename` | NotificationImage | NotificationImage::vftable[1]. When arg2 is nonzero: LOCK-increments the refcount at arg2+0x4, appends arg2 to the array at +0x88 (count +0x8c; grows through FUN_00bf4d70 when count+1 exceeds the capacity at +0x90), the… | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00bf65f0` | `NotificationImage::VirtualDecAndRelease ?` | NotificationImage | NotificationImage::vftable[2]. Decrements the dword at this+0x24, calls FUN_00bf5680, and returns with ret 8. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00bf6600` | `ConstructNotificationImageSetKey ?` |  | Calls the NotificationImage constructor 00bf6310 with *arg3, *arg4, *arg5 and arg6, stores *arg2 at this+0xa0, and returns this. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00c1edc0` | `SetClientTextColor ? ⚠rename` |  | Sets a color value on a resolved UI object. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `01239a70` | `UIServiceBase_ResetContainers ?` | UI::UIServiceBase | Virtual slot [1] of the UI service vtables: compacts the pointer vector at +0x40/+0x44 with _memmove_s when begin and end differ, erases the map at +0x50 through FUN_012394f0, resets its header and count at +0x54, then … | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `01239bc0` | `RebuildPtrVectorFromRing ?` |  | Walks the pointer vector at +0x40/+0x44 and the map at +0x4c/+0x50; when a flag argument is set and the ring count at +0x38 is nonzero it pops the tail (FUN_01237ff0), pushes a new string entry with FUN_01239490, releas… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `01239e30` | `RingPopThenRebuildFromString ?` |  | Initialises a local std::string, and when the ring count at +0x38 is greater than 1 pops the tail (FUN_01237ff0), assigns the string returned by FUN_012381b0 into the local, reads its field at +0x1c and calls FUN_01239b… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00424f37` | `— ⚠reject` |  | Shared slot-89 implementation also present in GRadioButtonCtrl, GComboBoxCtrl, GScrollPanelCtrl, GFormCtrl, GuiEditCtrl, GuiButtonCtrl, GuiLabelCtrl, GuiImageCtrl, and three unnamed UI-control vtables; the common base c… | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `00424fe0` | `— ⚠reject` |  | Removes one short-id registration from the control?s 16-byte record vector and cleans up its map/owner registration; the UI-control interpretation comes from its caller context. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `004251ba` | `— ⚠reject` |  | Keyed removal from the class's sorted element vector; the lower-bound + erase pair is the evidence. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `00425423` | `— ⚠reject` |  | Updates the named ToolTip child control from the supplied context. The string lookup supports the feature label, while the meaning of slot +0xa8 returning zero remains unknown. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `009fa830` | `— ⚠reject` |  | Clears the contents of a named edit control. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |

