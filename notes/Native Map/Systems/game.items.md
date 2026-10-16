# System: game.items

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae1a40` | `CanEquipItemInSlot ⚠downgrade` |  | Item equip validation helper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1ba0` | `RecalculateProfileItemCache` |  | Rebuilds cached profile item selection information. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1c40` | `GetCachedProfileItemValues` |  | Returns the cached profile item pair. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1d50` | `DestroyProfileItemOwner` |  | Owner cleanup path used by profile and item systems. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1f70` | `ProfileItemOwnerInit` |  | Initializes an item/profile owner object used by profile callers. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2090` | `InsertProfileItemAndRefreshCache` |  | Combined insert/cache refresh operation. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae19f0` | `— ⚠reject` |  | Item eligibility helper used by profile-related callers. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1df0` | `— ⚠reject` |  | Item action validation helper; wider meaning of the returned field is unclear. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

