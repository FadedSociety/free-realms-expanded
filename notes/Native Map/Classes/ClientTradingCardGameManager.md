# Class: ClientTradingCardGameManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad3670` | `ClientTradingCardGameManager::InitializeAssets` | ✅ | virtual | If offset +0x40 is zero, sets state +0x2ba90 to 1, stores a 2-float result from 00702670 at +0x2bbc0/+0x2bbc4, and lazily creates and starts the AssetsTcg resource object named Free Realms TCG. |
| `00ad3740` | `ClientTradingCardGameManager::OnDisconnect` | ✅ | method | Logs session ticket, launch ticket and station name to ClientTCG.log, then calls 010d00d0 if offset +0x14 is nonzero. |
| `00ad3e40` | `ClientTradingCardGameManager::Shutdown` | ✅ | virtual | If manager offset +0x14 is set, disables active state, removes TcgMainWindow, releases an allocated member, clears asset/resource objects and map nodes, and sets state +0x2ba90 to zero. |
| `00ad47f0` | `— ⚠reject` | ⚪ | virtual | Creates startup ticket/session objects, initializes resource and sound state, logs realm/session/launch/station values, registers the manager callbacks, and sets state +0x2ba90 to 3. |
| `00ad4e80` | `ClientTradingCardGameManager::ApplyMusicLevels` | ✅ | function | When global audio setting +0x6d8 is active, sends two values to Sound/Minigames and Sound/MiniGameMusic controls; always writes the values to settings 0xb and 0xc. |
| `00ad4f50` | `ClientTradingCardGameManager::Update` | ✅ | virtual | Advances staged manager states 1 through 4, starts TradingCardAssets.txt loading, checks an asset-count threshold, updates child objects in a tree and completes audio/resource work when loading ends. |
| `00ad6300` | `— ⚠reject` | ⚪ | ctor | Installs manager and delayed-asset status-handler vtables, initializes map and embedded resources, sets fields and initial TCG state to zero. |
| `00ad6410` | `ClientTradingCardGameManager::Dtor` | ✅ | dtor | Restores manager vtables, shuts down TCG resources, releases embedded strings and allocator state, destroys its typed map, and is called from EH cleanup paths. |
| `00ad6510` | `ClientTradingCardGameManager::ScalarDeletingDtor` | ✅ | dtor | Calls manager destruction and frees the object allocation when the deleting-destructor flag bit is set. |
