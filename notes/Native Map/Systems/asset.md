# System: asset

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008aa530` | `HashListMap_AssetReadyHandlerEntry_Construct` | SoeUtil::HashListMap | Initialises the pending asset-ready callback table of the asset management layer. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008aa5d0` | `HashListMap_AssetPtr_8192_Construct` | SoeUtil::HashListMap | Initialises the id-to-Asset* table of the asset management layer. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae2ba0` | `InitializeFixedAssetPools` |  | Preallocates reusable asset-related string/list records. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00c8abb0` | `CopyIconDataFields` |  | Copies an object containing IconData; owning class is unknown. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8ae30` | `CopyEmbeddedIconAndFields` |  | Copies an object with embedded icon data; its owner type is unknown. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00cd9480` | `CreateReferencedAssetObject` |  | Creates and retains an asset-backed object used by the caller. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

