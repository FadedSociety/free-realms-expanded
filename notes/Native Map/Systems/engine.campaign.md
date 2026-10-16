# System: engine.campaign

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `012516f0` | `CWCampaignMap::CriticalSection::ScalarDeletingDtor` | CWCampaignMap::CriticalSection | Scalar deleting destructor of the CWCampaignMap critical-section helper. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `012516a0` | `CWCampaignMap::CriticalSection::Dtor` | CWCampaignMap::CriticalSection | Destructor body of the CWCampaignMap critical-section helper; it is called from an EH unwind funclet, so it is a cleanup path. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

