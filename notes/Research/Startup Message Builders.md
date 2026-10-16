# Startup Message Builders

Research snapshot: 2026-10-13. Scope: the Rust gateway's 46 private startup replies in `lab/references/rust-gateway-reference`; this is a compatibility inventory, not production-server recovery.

## Capture inventory

All 46 files are tunnel messages with outer id `5`, one flag byte, a little-endian `u32` payload length, then the inner packet. The listed size includes the seven-byte outer header. Captured totals are 627,566 bytes. The private inventory report records each file's SHA-256 without copying packet data into the project: [`capture-inventory.json`](../../../lab/reports/startup-builders-20261013T191103Z/capture-inventory.json). Commands and check outcomes are recorded in [`command-log.md`](../../../lab/reports/startup-builders-20261013T191103Z/command-log.md).

The first eight replies are the login phase. The remaining 38 are the ready phase. Packet names below are client RTTI names or accepted guide labels where available. `u16/u32` describes the observed inner family/sub-opcode header; the 209 family uses a `u32` subtype in the client reader. The unclassified 193/2 interpretation remains provisional.

| # | Phase | Inner opcode | Bytes | Purpose | Character/account state vs content | Layout provenance |
|---:|---|---|---:|---|---|---|
| 00 | login | 41/132 (`u16/u16`) | 20 | Overworld-combat flag | Session/actor state; captured body is nine bytes, while the guide only settles the flag behavior. Remaining body fields are unresolved. | [[Build Guides/Gameplay/Combat and Abilities]]; dispatcher `00aa36c0`, setter `009058d0` |
| 01 | login | 2 | 10 | ClientGateway login reply | Session result; one status byte. | [[Build Guides/Network/Gateway and Transport]]; `PacketLoginReply` RTTI, body reader not mapped |
| 02 | login | 166 | 14 | Initialization environment | Deployment constant; captured string is `live`. Client compares it with its configured environment. | [[Build Guides/Network/Login and Account]]; `PlayClient::HandleInitializationParameters` `0090e6a0` (confirmed; disassembly checked) |
| 03 | login | 43 | 46 | Zone details | World content: name, type, tutorial flag, sky, zone id and geometry id. Zone id is the existing gateway's one patched field. | [[Build Guides/Network/Gateway and Transport]]; `PacketSendZoneDetails` ctor `008cebc0`; field walk in `gateway.rs::derive_zone_layout`; native receive reader not mapped |
| 04 | login | 144 | 26 | Client game settings | Account/client settings; 17-byte body fields are not identified by a native reader. | [[Build Guides/Network/Gateway and Transport]]; `PacketClientGameSettingsCtor` `0089bc20`; reader not mapped |
| 05 | login | 35/37 (`u16/u16`) | 263,908 | Item definitions | Static item catalogue content; length-prefixed payload, definition count and serialized definitions. | [[Build Guides/Gameplay/Items and Inventory]]; `0092f460`, `00a3db70`, `008e3760`, `00919e10`, receive cache `00a3c290` |
| 06 | login | 193/2 (observed) | 71 | Unidentified account/world initialization packet | Capture-only values; no accepted packet class or client reader was found. Meaning and variable fields unresolved. | No matching Build Guide reader or packet catalogue row found. Requires targeted native-map recovery. |
| 07 | login | 12 | 203,850 | Send self to client | Character and account state: identity, appearance, position, profiles, inventory and other player data. Existing gateway patches GUID/position/rotation, then identity and gameplay spans. | [[Build Guides/Gameplay/Character and Customization]], [[Build Guides/Gameplay/Items and Inventory]], [[Build Guides/Network/Gateway and Transport]]; opcode dispatch `009471a0`, self handling `00944fe0`, item/self path `0090a630`; constructor `0089b240`; full reader layout remains incomplete |
| 08 | ready | 67/1 (`u16/u16`) | 98,815 | Quick-chat phrase table | Static server content: counted keyed rows (id, parent, display string id, optional say string id, animation id). | [[Build Guides/Gameplay/Social (Friends, Guilds, Chat)]]; dispatch `00b966e0`, counted reader `00b90750` (likely; disassembly checked) |
| 09 | ready | 57 | 10,644 | Point-of-interest definitions | World content: POI keys, names, zone/location, heading and display metadata. | [[Build Guides/World/Zones and Terrain]], [[Build Guides/Network/Gateway and Transport]]; receive handler `00b98b70` (likely; disassembly checked), packet ctor `00b97ad0` |
| 10 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #1 | Static actor/world content: actor fields, transform and strings; no local-player state identified. | [[Build Guides/World/Actors and NPCs]], [[Build Guides/Network/Gateway and Transport]]; `PlayerUpdatePacketAddNpc_ReadBody` `00921a10` (confirmed) |
| 11 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #2 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 12 | ready | 35/2 (`u16/u16`) | 362 | Add NPC #3 | Static actor/world content; longer variable strings/fields account for the larger frame. | Same 35/2 reader `00921a10` |
| 13 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #4 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 14 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #5 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 15 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #6 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 16 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #7 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 17 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #8 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 18 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #9 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 19 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #10 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 20 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #11 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 21 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #12 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 22 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #13 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 23 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #14 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 24 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #15 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 25 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #16 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 26 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #17 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 27 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #18 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 28 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #19 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 29 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #20 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 30 | ready | 35/2 (`u16/u16`) | 356 | Add NPC #21 | Static actor/world content; variable strings/fields account for the larger frame. | Same 35/2 reader `00921a10` |
| 31 | ready | 35/2 (`u16/u16`) | 353 | Add NPC #22 | Static actor/world content; variable strings/fields account for the larger frame. | Same 35/2 reader `00921a10` |
| 32 | ready | 35/2 (`u16/u16`) | 341 | Add NPC #23 | Static actor/world content. | Same 35/2 reader `00921a10` |
| 33 | ready | 38/1 (`u16/u16`) | 19 | Hit points | Character state: current and maximum HP (`i32`, `i32`). | [[Build Guides/Gameplay/Combat and Abilities]]; family reader `009392c0`, fields described in guide |
| 34 | ready | 38/13 (`u16/u16`) | 20 | Mana/energy | Character state: current, maximum/secondary value and one byte flag. | [[Build Guides/Gameplay/Combat and Abilities]]; family reader `009392c0`, field reader `0089d360` |
| 35 | ready | 36/5 (`u16/u16`) | 51 | Ability-set definition | Character/job ability content; body is 38 bytes, with full field meanings unresolved in the startup evidence. | [[Build Guides/Gameplay/Combat and Abilities]], [[Build Guides/Network/Gateway and Transport]]; `AbilityPacketSetDefinition` ctor `00a2ccf0`; client reader/layout requires follow-up |
| 36 | ready | 38/16 (`u16/u16`) | 104 | Add effect tag #1 | Character status/content row: effect id, icon, duration, remaining time and name are client UI fields; additional bytes need field-level reconciliation. | [[Build Guides/Gameplay/Combat and Abilities]]; family reader `009392c0`, row applier `00a19e70` |
| 37 | ready | 38/16 (`u16/u16`) | 104 | Add effect tag #2 | Character status/content row; same schema as #36. | Same 38/16 reader/applier |
| 38 | ready | 209/1 (`u16/u32`) | 2,165 | Journal definitions | Static quest/journal content: four tables with key plus 7, 10, 3 and 9 `u32` columns. | [[Build Guides/Gameplay/Quests and Objectives]], [[Build Guides/Network/Gateway and Transport]]; parser `00a45d80`, body reader `00a45d10` (likely; disassembly checked) |
| 39 | ready | 209/2 (`u16/u32`) | 17 | Journal set | Character quest-progress state: `u32` count followed by `u32` pairs; captured packet represents an empty set. | [[Build Guides/Gameplay/Quests and Objectives]]; parser `00a45bf0`, key/value reader `00a45680` (likely; disassembly checked) |
| 40 | ready | 165/1 (`u16/u16`) | 39,254 | Coin-store item lists | Server catalogue plus account availability/state; two lists of 19-byte entries. The three dwords and three flags are read, but their meanings remain unknown. | [[Build Guides/Gameplay/Coin Store and Purchases]]; handler `00c75fc0`, record reader `00c71510` (speculative label; disassembly checked) |
| 41 | ready | 93 | 236 | Welcome-screen data | Account/promotion content: welcome flag, content rows, claim-code rows and trailing values. | [[Build Guides/Network/Gateway and Transport]]; parser `00939200`, applier `00ad0790` (likely; parser disassembly checked) |
| 42 | ready | 14 | 9 | Unidentified marker | No body beyond inner opcode; purpose and appropriate startup phase are unresolved. | No matching Build Guide reader or packet catalogue row found. |
| 43 | ready | 38/26 (`u16/u16`) | 12 | Done-sending-preload-characters marker | Session sequencing marker; one captured trailing byte has unknown meaning. | [[Build Guides/Network/Login and Account]], [[Build Guides/Network/Gateway and Transport]]; family reader `009392c0`, ctor `0089e8c0` |
| 44 | ready | 38/7 (`u16/u16`) | 59 | Update stat | Character/account stat state; 48-byte body not tied to a reviewed native reader in current guide. | [[Build Guides/Gameplay/Combat and Abilities]]; family dispatch `009392c0`; field reader remains open |
| 45 | ready | 87 | 221 | Message of the day | Constant server text content in two strings; no player text. | [[Build Guides/Network/Gateway and Transport]]; `PacketMOTD` ctor `008ea4e0`; client body reader not mapped |

### State and content summary

Character/session-bearing messages include 41/132, login status, self (12), hit points, mana, ability set, effect tags, journal set and 38/7. Zone details and 35/2 actor spawns are world content. The item catalogue, quick-chat table, POIs, journal definitions, coin-store lists, welcome payload and MOTD are server content or account presentation. The startup capture does not establish original production values or server rules.

Captured-only or incomplete layouts: 193/2 has no accepted map reader; 14 has no mapped purpose; 144, 36/5, 38/7 and the full self body lack a complete client-reader field map. The mapped 165/1 reader consumes entries but does not establish the meanings of their fields. Do not promote captured values in those fields to recovered server policy. Any retained unknown value in a future builder needs a named `// (R) unknown field, value from capture` constant.

## Builder implementation status

`prototype/osfr-tools/src/startup_messages.rs` currently builds seven packets from typed inputs: login result (index 01), initialization environment (02), zone details (03), hit points (33), mana (34), journal set (39), and the preload-complete marker (43). Zone and marker fields without recovered meanings are named capture-valued constants and tagged `(R)` in code. The ignored private oracle compares these seven builders with their original files; ordinary tests vary zone name/zone id and journal entry count/content.

This does **not** meet the requested 46-message replacement yet. The unimplemented capture indices are `00, 04-32, 35-38, 40-42, 44-45` (39 messages). In particular, no complete builder exists for self data (index 07), so synthetic character name/GUID/position and inventory-size coverage is still absent. The definition catalogue, NPC/quick-chat/POI content, effects, journal definitions, coin store, welcome data, MOTD and other incompletely mapped packets need structured inputs and client-reader layouts before exact builders can be added. The capture oracle must not be treated as a production content source. Do not switch the gateway until all 46 packet builders and synthetic character-state coverage are reviewed.

## Gateway handoff

No gateway edits are part of this worker scope. After every message builder passes the private byte oracle, the lead's exact switch points are:

1. In `prototype/osfr-tools/src/gateway.rs`, `Gateway::handle_at`, the `Event::Login` branch at the current `templates_for(Kind::Login, &state, gameplay.as_ref())` call: replace with `startup_messages::build_login(...)`.
2. In the `Event::Ready` branch, replace only the `templates_for(Kind::Ready, &state, gameplay.as_ref())` fallback with `startup_messages::build_ready(...)`; preserve the existing `kart_ready_stream()` route.
3. Remove `Gateway::templates_for` after its identity, inventory, zone and gameplay patches have moved into the builder inputs. Then remove `Template`, `Manifest.templates`, `load_reference` message bytes and the now-unused layout offsets only after private-gate and actual-client acceptance.

## Method and evidence limits

Applied reverse-engineering and Ponytail guidance. Native-map labels were read from `lab/native-map/run/map.sqlite`; likely/speculative/unknown reader candidates used above were checked with `python -I scripts/native_map/nmq.py dis`. No game launch, external web search, asset work, source copying, or player name/chat was used. The receipt and exact command log are in `lab/reports/startup-builders-20261013T191103Z/`.
