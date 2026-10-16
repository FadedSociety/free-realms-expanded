# System: ui.socialshare

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a714e0` | `SocialShareDataSource::FieldName` | SocialShareDataSource | Column-name lookup for the social-share data source rows (Provider, Status, Enabled). | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a71e00` | `SocialShareDataSource::DtorBody` | SocialShareDataSource | Destructor body of SocialShareDataSource: releases the SocialShareData row array and runs the destructor of the embedded GuiUserDataTable holder. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a720f0` | `SocialShareDataSource::Ctor` | SocialShareDataSource | Constructor of SocialShareDataSource: builds its data source and fills the initial rows. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a721a0` | `SocialShareDataSource::ScalarDeletingDtor` | SocialShareDataSource | Scalar deleting destructor of SocialShareDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a71c60` | `SocialShareDataSource::RebuildRows` | SocialShareDataSource | Rebuilds the social-share row array (a header row plus rows from string-table indices 1 and 2) and notifies the owner through vtable slot +0x10. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a71eb0` | `— ⚠reject` | SocialShareDataSource | Cell accessor of the social-share data source (row, column) returning the row fields for Lua/UI display. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

