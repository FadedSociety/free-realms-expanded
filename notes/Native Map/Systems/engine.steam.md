# System: engine.steam

19 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae1370` | `OverlayCallbackDtor` | CCallback<ClientSteamImpl,GameOverlayAc… | Steam callback cleanup. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1390` | `OverlayCallbackDispatch` | CCallback<ClientSteamImpl,GameOverlayAc… | Virtual dispatch method for the Steam overlay activation callback. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae13a0` | `OverlayCallbackInvoke` | CCallback<ClientSteamImpl,GameOverlayAc… | Callback invoke slot for the Steam overlay callback wrapper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae13d0` | `MicroTxnCallbackDtor` | CCallback<ClientSteamImpl,MicroTxnAutho… | Steam microtransaction callback cleanup. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae13f0` | `MicroTxnCallbackDispatch` | CCallback<ClientSteamImpl,MicroTxnAutho… | Virtual dispatch method for Steam microtransaction responses. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1400` | `MicroTxnCallbackInvoke` | CCallback<ClientSteamImpl,MicroTxnAutho… | Callback invoke slot for the Steam microtransaction callback wrapper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1780` | `SteamClientCtor` |  | Builds Steam integration object and registers its callbacks. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae12d0` | `SteamClient::GetUserIdFromOwner` |  | Pointer-indirected counterpart of 00ae11a0. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1860` | `CreateSteamClient` |  | Factory-like allocator for SteamClient integration state. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟡 speculative (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae1120` | `SteamClient::Initialize ?` |  | Steam API initialization wrapper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1140` | `SteamClient::Shutdown ?` |  | Steam API shutdown wrapper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae11a0` | `SteamClient::GetUserId ?` |  | Reads the active Steam user identifier. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae12a0` | `SteamClient::IsInitialized ?` |  | Small field accessor used by Steam client wrappers. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1470` | `RegisterSteamCallback ?` |  | Installs the Steam microtransaction authorization callback. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1550` | `RunSteamCallbacks ?` |  | Pumps Steam SDK callbacks. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae15e0` | `SteamClient::ScalarDeletingDtor ?` |  | Deleting destructor wrapper for Steam integration object. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1600` | `SteamClient::DeleteOwnedObject ?` |  | Releases a Steam integration object held by the owner. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae14b0` | `— ⚠reject` |  | Tears down Steam integration state and both callback registrations. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1630` | `— ⚠reject` |  | Processes Steam microtransaction authorization callback; it does not itself grant an in-game item. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

