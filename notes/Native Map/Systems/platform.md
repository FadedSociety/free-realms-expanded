# System: platform

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0089c4a0` | `SceaSharedUtil::CrashReporterHandler::~CrashReporterHandler` | SceaSharedUtil::CrashReporterHandler | Base destructor of the crash-reporter hook (Scea = Sony Computer Entertainment America shared util, i.e. platform-side code). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `0089c4b0` | `SceaSharedUtil::CrashReporterHandlerScalarDtor` | SceaSharedUtil::CrashReporterHandler | Deleting destructor of the crash-reporter hook. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097a710` | `PlatformStringMappingsLoad` |  | Loads the platform string mapping table from ./Resources/PlatformStringMappings.txt into the object's map. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097adb0` | `PlatformStringMapOwnerCtor` |  | Constructs the owner of the platform string mapping table and loads PlatformStringMappings.txt into it. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |

