# Game rules and membership

Snapshot: 2026-10-13. Native client reference: FreeRealms.exe 1.910.1.530630, queried through the reviewed map at `lab/native-map/run/map.sqlite`. This note records client facts separately from revival policy (R). The game-rule subsystem is a client-side content/expression system; it is not evidence for an undocumented server packet schema.

## Recovered client facts

### Account membership and messages

| Fact | Client provenance | What is established |
|---|---|---|
| Account-info callback values | `ClientAccountManager::OnAccountInfo`, `00719330` (confirmed) | On success, three supplied values are copied to client fields `+0x22280` (member), `+0x22284` (maximum characters), `+0x22288` (admin). They are callback values, not a recovered binary message. |
| Membership activation | `PacketMembershipActivation` constructor `0089bab0`; dispatch case `0x64` at `009471a0`; bounded reader `008d5ee0` via `008f6e30`; setter `00a12470` | This is **opcode 100 / 0x64**, with a six-byte body: opaque signed i16 followed by i32. The i32 is stored at game-state `+0x464`, and its nonzero form drives the membership-dependent state and `MembershipHandler:OnMembershipActivated`. The i16 meaning is unknown. A different dispatcher case `0xca` is unrelated here; the `202` attribution in the quest guide conflicts with the constructor and dispatcher bytes. |
| Subscription info | `PacketMembershipSubscriptionInfo` constructor `0089c200`; dispatch `009471a0` case `0xbd`; reader `008d6600` via `008f7300` | Opcode 189 / 0xbd carries an opaque i16 and two boolean bytes. Dispatcher stores the bytes at game-state `+0x468` and `+0x469`. Their names/meanings and any expiry semantics are unresolved. |
| VIP info | `PacketClientMembershipVipInfo` constructor `008b1970`; dispatch `009471a0` case `0xba`; reader `008f7290` | Opcode 186 / 0xba updates two dwords on a resolved actor and raises a membership/VIP UI event. This is separate from the account membership dword at `+0x464`; payload meanings are not assigned here. |
| Membership check for equipment | equip action `00bdbc50` (native pseudocode inspected); getter `008a6990`; caller Lua binding `00c10bf0` | Before sending family 42 equip/remove requests, the client rejects a member-only item or member-only equipment slot when the account membership dword at game-state `+0x464` is zero. This is the concrete local authorization check found in the item flow. |
| Member quest cap selection | getter `00a10890`; activation setter `00a12470`; self-data quest-limit fields described in `Gameplay/Quests and Objectives.md` | The client selects one of two quest limit dwords at player `+0x1bed0` / `+0x1bed4` based on the activation value at `+0x464`. The limit numbers are server-supplied; their historical values are not established here. |
| Quest-limit notice | dispatcher `00a91bf0`, family 47 subtype 15 | A one-byte classification selects `QuestLimitMember` or `QuestLimitNonmember` UI text; the notice does not contain the numeric cap. |
| VIP rank requirement | requirement type `ClientRequirementCheckVipRank`, ctor `00bc4670` and pool path `00bc4ec0`; Marketplace field list `00997840` | The client names a VIP-rank requirement type and exposes `VipRankRequirement` on store bundles. VIP rank is distinct from the account member boolean; no VIP-rank values/rule ids are embedded in this module. |

### Member-only client content and UI

These fields prove that the client can display or classify member-only content. Unless explicitly stated above, the mapped evidence does not prove the server-side denial behavior.

| Content | Client provenance | Evidence boundary |
|---|---|---|
| Store bundles / coin shop | `StoreBundleFieldNameForIndex` `00997840`; coin-store item columns `00cf07f0`; tooltip builder `00baec20` | Fields include `IsMembersOnly`, `MembersOnlyPrice`, `Member Purchase Price`, `MemberPrice`, and nonmember price. The item tooltip shows a localized membership requirement; the coin-store result text includes `Fail_NotAMember`, but no status number or captured purchase exchange was established. |
| Housing | `HousingDataSource::ColumnHeaderText` `00cbb310`; `HousingInstanceDataSource::ColumnHeaderText` `00cbb210`; Lua surface includes `IsFixtureMembersOnly` | Lot, instance and fixture member-only presentation exists. No mapped native local lot-entry refusal was identified. Server entry enforcement is R. |
| Mounts | mount record byte offset `+0x28`; `BaseClient.MountList` column 6 via `00cc3910`; trial dispatch and timer `00b056a0` | Mount records carry a member-only flag. Trial messages use opcode 168 subtypes 10, 11 and 13. Trial duration/cooldown policy is not recovered, and no local member-only mount-use denial was identified. |
| Pets | `PetListDataSource` column `Members Only` via `00cca5c0`; summon binding `00c066a0`; trial timer `00b589c0` | Member-only data is exposed in the list. The panel-level denial is reported by the existing guide, but no native address for that script predicate was recovered. Pet trial timer/lockout notices are client-handled; eligibility, length and cooldown values remain R. |
| Minigames and activities | data-source column functions `00c45760`, `00c45860`, `00c45a00`, `00c45af0`, `00c477c0`; activity request class `01086990` | Goal, group, minigame, portal and activity-request data expose member-only fields. No client-side launch denial or numeric minigame rule id was established by those readers. |
| Quests and goals | quest source `00cf7a10`; objective source `00cb7710`; goal formatter `00c42440` | Quest/objective data exposes member-only state; a minigame goal formatter has a `MembershipRequired` presentation state. These fields establish UI/presentation categories, not a universal server rule. |
| Jobs and expired trials | job profile columns `00c8f2b0`; trial-upsell dispatch at `009471a0` case `0x91`, reader `008d61c0`; `ValidateJobForStartScreen` client Lua | For a minigame start screen, the client hides a job whose trial expired unless the player is a member. The membership condition is direct client Lua behavior; server-side job eligibility and trial-expiry policy remain R. |

### Generic game-rule data and other requested toggles

- `RuleSetData` stores a `RuleSetID` and expression-tree pointer (`013c5920`, getters/setters `013c5b50`/`013c5b60`, evaluator/debug formatter `013c5a90`). `RuleSetDB_LoadRuleIndex` (`0129f6d0`) opens local `./data/archetypes` and `ruleIndex`; `RuleSetDB_GetOrLoadRuleSet` (`0129fea0`) resolves local content. No numeric rule ID or generic game-rule packet was recovered for the membership gates above.
- `ClientRequirementCheckCountry`, `ClientRequirementCheckLanguage`, `ClientRequirementCheckCurrentProfile`, and `ClientRequirementCheckVipRank` are named requirement types (for example, VIP type constructor `00bc4670`). Do not substitute VIP for the account member boolean.
- Chat input has a direct client gate in Lua: `ChatLog`/`GameDock` refuse typed text unless the account is not underage, its open-chat byte is enabled, or the client is in the China environment. The bytes are read from self data by `00944fe0` (`+0x460` underage, `+0x461` open-chat, membership at `+0x464`). This is a client input gate; server moderation and recipient policy are separate and R.
- The server-to-client chat channel map identifies channel 10 as `WorldMembersOnly` (`00b11600`); quick-chat channel names are mapped at `00b978b0`. This identifies a member-only channel, but the client receive map does not enforce account membership. Server recipient membership checks remain R. `WorldTrade` is likewise a channel name, not an account permission flag.
- A server-driven per-key card-trade toggle does exist: command 348/version 1 `TradeCommand_SetTradeAllowed` (`nmq command 348`). Its reader is `014a68e0`; `014a6720` reads a count followed by `(u32 key, u32 flag)` pairs and normalizes nonzero flags to true; handler `014a6250` emits one keyed event per entry; writer `014a65b0`/`014a63e0` emits the same tree. Key meanings and connection-specific outer framing are unknown. The shared integer serializer `013c23f0` supports compact integers or four-byte network order according to global `DAT_01cbdd7c`; no trade command capture was used to identify which mode is active in this session. This card-trade flag tree is distinct from a global direct-player-trade permission.
- No global PvP-enabled or general chat-enabled flag/message was found in the inspected map. Player-trade refusal strings exist, but their flows are not traced. Do not infer a permission value from those strings or from the absence of a global toggle.
- Mount and pet trial timers/lockouts are server-driven client updates, not evidence of membership expiry. The mapped membership activation and subscription messages expose no expiry timestamp. The exact mount/pet trial durations and cooldowns are not recovered.
- Reward opcode 50 subtype 6 is named `RewardPacketNonMemberRewardDeferred` (`00b88e10`), but `RewardFamilyPacketDispatch` (`00b8a640`) uses it to accumulate deferred XP/coin bank totals. The name alone does not establish a member authorization gate.

### Startup capture values

The login capture contains one opcode-12 self-data packet and no opcode-100 or opcode-189 packet. The self-data parser/serializer in `prototype/osfr-tools/src/startup_selfdata.rs` reads/writes account values in this order: underage byte, open-chat byte, membership-status u32, member-nag byte, country/language dwords, login-count dword, grandfathered byte, then two vehicle-loadout dwords (`parse_self` around 1069-1082; `build_self` around 1194-1205). The inspected private capture has `underage=0`, `open_chat=0`, raw `membership_status=1`, `show_member_nag=0`, `grandfathered=0`. The meaning of raw membership-status value 1 is not established; it is not used to infer membership here. The ignored capture test parses these scalar flags and rebuilds the whole packet byte-for-byte without asserting, logging or embedding identity/chat data.

`startup_messages.rs` currently emits login reply, initialization parameters, zone details, HP/mana, journal and preload-complete packets. The four zone bytes retained there are unnamed captured zeros, not identified game rules. The membership message encoders in `src/game_rules.rs` follow the native readers; opcode 100 and 189 are absent from the captured startup message sequence, so their byte tests are native-layout tests, not capture-parity claims.

## Revival policy (R)

- Default new/unconfigured accounts to an active member, following the current Build Guide policy. Explicit account configuration can override this. Membership expiry is an optional local UNIX timestamp; no client packet inspected here supplies one.
- A single `GameRuleSet::may` function handles member-only resource categories, expired-job trial continuation, member/nonmember quest limits, per-key card-trade flags, and configurable PvP/direct-trade/chat/open-chat/trial switches. Defaults for unconfigured switches and missing card-trade keys are permissive (R), not recovered client values.
- A member-only `GameAction` currently checks active membership uniformly. Item equipment and expired job-trial visibility are directly supported by client evidence. Housing, mounts, pets, store, quests, member-only chat channels and minigame/zone denial are policy hooks for the integration layer; their UI fields alone do not prove original server enforcement.
- `OpenChat` uses the account-level underage/open-chat fields and optional China environment exception described by the client Lua. `CardTrade` applies explicit entries delivered by command 348; unknown keys fall through to the R global trade switch.
- Quest limits are optional because historical numerical values are unknown. When configured, the client-backed selection model uses the member or nonmember limit. Mount/pet trial start gates accept a caller-supplied remaining cooldown; exact duration/cooldown rules remain config policy.
- Do not infer PvP or direct-player-trade permission from the absence of messages or from similarly named chat channels. Global switches exist only so an integration can apply explicit revival configuration in one place.

## Wiring for lead integration

1. Map trusted `AccountFlags.membership` from `src/account.rs` / `src/account_store.rs` to `MembershipState`; do not derive it from self-data `membership_status=1` without evidence. Choose where account configuration overrides the R member default.
2. Add the one-line module registration `pub mod game_rules;` to `src/lib.rs` in a lead-owned integration change; this task does not edit shared module registration.
3. When extending startup self data, wire account values to `AccountState` in `startup_selfdata.rs`. That existing record also carries the client-facing underage/open-chat/status/nag/grandfathered fields, whose meanings should remain distinct from the trusted membership policy. Current `startup_messages.rs` has no membership/rule update builder.
4. Pass `GameRuleSet` to coin-shop purchase validation, housing-lot/fixture checks, mount use, pet summon/trials, quest acceptance, and any future PvP/trade/chat handlers. Preserve the client’s member-only flags in their content rows; a UI flag alone does not authorize an action.
5. If enabling quest caps, source both values from explicit config or verified character self data and test each path. Do not invent the historical limits or trial duration/cooldown.

## Artifacts and checks

- `prototype/osfr-tools/src/game_rules.rs`: typed membership/chat flags, quest limits, card-trade flag-tree entries, R configurable switches, centralized access decision, callback value encoding, and membership/trade-flag field encoders. The trade helper encodes only count/key/value fields, not class/base/routing framing.
- `prototype/osfr-tools/tests/game_rules.rs`: direct `#[path]` include so shared `lib.rs` stays untouched; covers member/nonmember bytes, opcode layouts, compact and network-order trade flag fields, policy gates and the optional private startup capture round trip.
- Capture test command uses `OSFR_GATEWAY_REFERENCE` to point to `lab/references/rust-gateway-reference`; it reports only scalar account flags and a byte equality assertion.

Commands and results:

| Command | Result |
|---|---|
| Private-toolchain `rustfmt.exe --edition 2024 prototype\osfr-tools\src\game_rules.rs prototype\osfr-tools\tests\game_rules.rs` with `CARGO_HOME` and `RUSTUP_HOME` set under `lab\tools` | Exit 0; only owned Rust files formatted. |
| `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W54'`; `& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test game_rules` | Exit 0; 7 passed, 0 failed, 1 private-capture test ignored by default. Existing warning: `housing.rs:567` dead-code method `instance_index`. |
| Set the same target plus `$env:OSFR_GATEWAY_REFERENCE='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\references\rust-gateway-reference'`; run `& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test game_rules '--' '--ignored' 'captured_self_account_flags_round_trip_without_disclosing_identity'` | Exit 0; 1 capture round-trip passed, 7 filtered out. No identity/chat values were printed. |

Read-only native-map checks used `python -I scripts/native_map/nmq.py show` for the account callback, membership constructors/readers/dispatcher, membership setter, quest-limit getter, item equip check and trade-command reader/writer/handler; `python -I scripts/native_map/nmq.py command 348` returned `TradeCommand_SetTradeAllowed`, version 1; `python -I scripts/native_map/nmq.py dis 014a6720 96` confirmed the key/flag reads. The dispatcher pseudocode for `009471a0` was also searched directly in the SQLite map to distinguish case `0x64` (membership activation) from case `0xca`.

Native-map facts above were queried read-only using `python -I scripts/native_map/nmq.py`; unknown or contradictory opcode attribution was checked against `009471a0`'s bytes/pseudocode and its `0x64`/`0xbd` cases. The TradeAllowed command's speculative labels were checked against `014a6720` and serializer bytes/helpers before the compact field encoder was added. No decompiled code or raw assets are stored in this repository.
