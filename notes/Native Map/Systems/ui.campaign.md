# System: ui.campaign

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01250cb0` | `Engine::FRTWCampaignScreenUpdate::DebugName` | Engine::FRTWCampaignScreenUpdate | Debug or log label of the campaign screen update action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250cc0` | `Engine::FRTWCampaignScreenUpdate::ScalarDeletingDtor` | Engine::FRTWCampaignScreenUpdate | Scalar deleting destructor of the campaign screen update action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250df0` | `Engine::FRTWCampaignScreenLoaded::TypeName` | Engine::FRTWCampaignScreenLoaded | Action type name 'CampaignScreenLoaded' used to dispatch the campaign-screen-loaded notice. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250e10` | `Engine::FRTWCampaignScreenLoaded::DebugName` | Engine::FRTWCampaignScreenLoaded | Debug or log label of the campaign-screen-loaded action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250e20` | `Engine::FRTWCampaignScreenLoaded::ScalarDeletingDtor` | Engine::FRTWCampaignScreenLoaded | Scalar deleting destructor of the campaign-screen-loaded action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0124dfe0` | `UI::StartCampaignMode::OpenMenuScreen` | UI::StartCampaignMode | Opens the UI::MenuScreen when the campaign-mode start guard passes. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250c20` | `Engine::FRTWCampaignScreenUpdate::ctor` | Engine::FRTWCampaignScreenUpdate | Constructs the CampaignScreenUpdate action (handler base, then the campaign-screen class). | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250c90` | `Engine::FRTWCampaignScreenUpdate::TypeName ⚠downgrade` | Engine::FRTWCampaignScreenUpdate | Action type name 'CampaignScreenUpdate' used to dispatch the campaign screen update. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250d80` | `Engine::FRTWCampaignScreenLoaded::ctor` | Engine::FRTWCampaignScreenLoaded | Constructs the CampaignScreenLoaded action (handler base, then the campaign-screen-loaded class). | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01250f80` | `Engine::FRTWCampaignScreenUpdate::RefreshIfActive ? ⚠downgrade` | Engine::FRTWCampaignScreenUpdate | Slot [2] of Engine::FRTWCampaignScreenUpdate::vftable, 31 bytes. Calls FUN_012e0ba0, then FUN_0124d010; if that returns nonzero it calls FUN_0131faa0; returns 1. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `012510d0` | `— ⚠reject` | Engine::FRTWCampaignScreenLoaded | Populates the campaign screen with placeholder TEST campaign and mission data (40 entries with random chapter, index and colour) rather than server or shipped campaign content. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

