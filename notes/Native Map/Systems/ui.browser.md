# System: ui.browser

31 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00993cf0` | `EmbeddedWebBrowserLibrary::OnAssetsLoaded` | EmbeddedWebBrowserLibrary | Completes embedded-browser asset loading and records success or failure. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00993e30` | `EmbeddedWebBrowserLibrary::OnAssetLoaded` | EmbeddedWebBrowserLibrary | Handles the completion status for one embedded-browser asset load. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00993e90` | `EmbeddedWebBrowserLibraryDtor` | EmbeddedWebBrowserLibrary | Destroys the embedded browser library state. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994030` | `EmbeddedWebBrowserLibraryHandlerDtor` | EmbeddedWebBrowserLibraryHandler | Begins base handler teardown. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994040` | `EmbeddedWebBrowserLibraryHandlerScalarDeletingDtor` | EmbeddedWebBrowserLibraryHandler | Scalar deleting destructor for the browser library handler. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994060` | `GuiWebBrowserCtrl::UiHandlerDtor` | GuiWebBrowserCtrl::UiHandler | Resets the UI handler's vtable as part of destruction. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994070` | `GuiWebBrowserCtrl::UiHandlerScalarDeletingDtor` | GuiWebBrowserCtrl::UiHandler | Scalar deleting destructor for the web-browser UI handler. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009940f0` | `FindOnlineHelpBrowserControl` |  | Retrieves the browser interface from the named online-help window. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994150` | `FindOnlineHelpWindow` |  | Finds the online help window and queries a member through its interface. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009942e0` | `OnlineHelpSingletonDtor` | Singleton<OnlineHelpWindow> | Resets the online-help singleton subobject vtable. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994310` | `OnlineHelpSingletonScalarDeletingDtor` | Singleton<OnlineHelpWindow> | Scalar deleting destructor for the online-help singleton subobject. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994470` | `OnlineHelpWindowDtor` | OnlineHelpWindow | Destroys the online-help window's owned browser and string state. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009946d0` | `OnlineHelpWindow::ScalarDeletingDtorAdjustorMinus4` | OnlineHelpWindow | OnlineHelpWindow vtable destructor entry. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009946e0` | `OnlineHelpWindow::ScalarDeletingDtorAdjustorMinus8` | OnlineHelpWindow | OnlineHelpWindow secondary vtable destructor entry. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009947b0` | `OnlineHelpWindow::ScalarDeletingDtor` | OnlineHelpWindow | Scalar deleting destructor for OnlineHelpWindow. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00995060` | `OnlineHelpWindow::HandleUrlCommand` | OnlineHelpWindow | Routes supported online-help browser URL commands into game actions. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009954d0` | `HandlerOnlineHelpWindowOnStatus` |  | Forwards browser status events to the online-help script handler. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00995620` | `OnlineHelpWindow::OnNavigationResult` | OnlineHelpWindow | Handles online-help browser navigation completion. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00993ce0` | `SetField2BA70` |  | Sets a stored browser-related object pointer; feature context follows its callers. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00993de0` | `ProcessBrowserLoadState` |  | Processes asynchronous browser-load state and reports its result through a nested interface. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00993f00` | `LoadBrowserAssets` |  | Starts asynchronous browser-asset loading unless a load is already active; the owning class is not established by a direct vtable anchor here. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00993fb0` | `EmbeddedWebBrowserLibrary::Ctor ⚠downgrade` | EmbeddedWebBrowserLibrary | Initializes embedded-browser library state. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009940b0` | `EmbeddedWebBrowserLibraryScalarDeletingDtor` | EmbeddedWebBrowserLibrary | Scalar deleting destructor for the embedded browser library. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009943c0` | `QueryBrowserControlAndDispatchResult` |  | Likely forwards the queried online-help browser control to a state-guarded dispatch helper; the helper action is not identified. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009947d0` | `BaseClient::InitializeBrowserConfiguration` | BaseClient | Configures browser paths and state from BaseClient startup; this binding is supported by caller 009029a0 passing its this pointer. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009955f0` | `CanDispatchOrReportStatus` |  | Gates an online-help action and reports status 2 on failure; the feature attribution comes from the two called helpers. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00993dc0` | `SetStoppedAndCancelNestedObject ? ⚠rename` |  | Marks the handler stopped and requests cancellation from its nested object. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00994290` | `— ⚠reject` |  | Forwards the located browser control to an unidentified virtual operation; slot +0x8c semantics remain unknown. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009943d0` | `— ⚠reject` |  | Routes an online-help browser action through the available browser interface. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00994980` | `— ⚠reject` | OnlineHelpWindow | Constructs the online-help window and embedded-browser integration. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00995660` | `— ⚠reject` |  | Forwards the stored string to the queried browser control and sets a state field to 4; the forwarded value semantics and state meaning are unknown. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00995060` | `OnlineHelpWindow::HandleUrlCommand` | send | ClientHousingPacketEnterRequest |  |  |
