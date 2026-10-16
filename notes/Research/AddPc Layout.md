# AddPc Layout (W77)

Research for Build Plan 1.3, 2026-10-15. **Researched; no implementation or native multiplayer acceptance.** Only this note was written. Code, saves, shared notes, binaries and the native client were left untouched.

## Result and evidence boundary

35/1 is already the correct spawn path. Replace its presentation placeholders using the same authenticated identity, active profile and inventory as opcode 12. Reuse `Inventory::profile_attachments`; do not send the self-data equipment triples as attachments. Preserve both opcode-125 tail bytes after movement validation, including updates that change only those bytes.

**Additional correction:** AddPc's four-float field called `Rotation` by Legacy is consumed as a direction. With the AddPc object at stack `+0x304`, native `0092ff2c` loads `+0x3c4` (object `+0xc0`, X), `0092ff3b` loads `+0x3cc` (object `+0xc8`, Z), and `0092ff44` executes `fpatan`: yaw = `atan2(X,Z)`. `players.rs:330-335` currently writes a half-angle yaw quaternion. For yaw pi/2 that supplies X=Z=0 and cannot represent the requested facing. Use `[direction.x, direction.y, direction.z, 0]`, or `[sin(heading), 0, cos(heading), 0]` while retaining the heading API. W is wire-present but not used by this yaw calculation. This supersedes the quaternion interpretation in [[Player Visibility]].

Confidence marks used below:

- **C:** wire width/order or stated operation checked against native instructions in this task. An unknown map label does not lower a directly checked byte fact.
- **L:** semantic name/order corroborated by the pinned Legacy source; not independently established as the original server's meaning.
- **U:** meaning unresolved; preserve its position, do not assign invented gameplay meaning.
- **(R):** proposed reconstruction/server policy, not an original fact. All default choices below have this status.

Sources:

- **Native:** `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` (rehashed). Addresses are executable VAs, not packet offsets. Read-only map: `lab/native-map/run/map.sqlite`; `lab/native-map/ACTIVE` resolves there. Labels provide leads; `nmq dis` provides the instruction checks.
- **Legacy root:** `lab/references/FreeRealms-Legacy`, revision `9e082025391177d5a33ee18c714ddd06b457a695` (checked), AGPL-3.0. Paths abbreviated **P** = `src/Sanctuary.Packet/BasePlayerUpdatePacket/PlayerUpdatePacketAddPc.cs`, **N** = `src/Sanctuary.Packet.Common/NameData.cs`, **A** = `src/Sanctuary.Packet.Common/CharacterAttachmentData.cs`, **T** = `src/Sanctuary.Packet.Common/PlayerTitleData.cs`, **M** = `src/Sanctuary.Packet/PlayerUpdatePacketUpdatePosition.cs`. Consulted for field hints; no C# implementation copied.
- **Rust root:** all `src/...` below are under `prototype/osfr-tools/`. [[Self Data Layout]], [[Movement Validation]], [[Native NPC Motion and Animation]], [[../Reviews/Native SelfData 011719]] and the newest [[../Reviews/Native Round5 050310]] were read. The world guides still have placeholder narratives; the Character and Customization guide documents active-profile equipment. No missing feature is inferred from those placeholders.

## Existing route and reuse points

`bin/osfr-gateway.rs:450-469` joins the registry with only identity GUIDs and pose. `players.rs::PlayerRegistry::diff_messages` calls `build_add_pc` for new visibility and `build_position_update` for an existing visible subject. The gateway runner's `commit_outcome` (`bin/osfr-gateway.rs:1211-1249`) updates the registry only when position or rotation changed. `players.rs::update_pose` independently suppresses equal-pose updates. Both comparisons must include the two movement bytes for state-only relays to work.

`gateway.rs:510-521` already parses both bytes into `MovementUpdate`, checks the packet GUID against `CharacterState.guid`, and requires complete consumption at 535. `movement.rs::MovementState` already stores `state` and `unknown` (36-48); `accepted_state` preserves them (340-359). `Gateway::handle_at` commits that validated state (960-1049). The values are lost at the registry boundary, not at parsing. Rejected/ignored raw reports must not be forwarded.

`gateway.rs::build_self_message` (1397-1440) overlays `CharacterIdentity` and `Gameplay::state()` on typed `SelfData`. It updates model, appearance strings/colors and literal name from identity, inventory from gameplay, active job through `apply_profile_state` (1443 onward), and transforms from `CharacterState`. It does **not** refresh `SelfData.name.fields` or account flags there. Never take another character's template name IDs or template membership as authenticated presentation.

Useful existing types/helpers:

- `character_store.rs::CharacterState:25-30`: GUID, zone, position XYZ, direction XYZ only; no appearance or transient movement flags.
- `identity.rs::CharacterIdentity:78-93`: real names, model, head, hair, colors, skin, face paint, customization. `identity::valid_name:95-107` supplies existing name bounds.
- `startup_selfdata.rs:64-120`: `Identity`, `Appearance`, `Transform`, `CharacterName`, `AccountState`, `JobState`; these provide the crosswalk, not a reason to store the entire large self packet in each registry row.
- `profiles.rs::ProfileState:15-20`: `active`; `GameplayState.profiles` and `.inventory` come from `Gameplay::state` (1442-1453).
- `inventory.rs::Inventory::profile_attachments:340-365` returns exactly `i32 count + attachment rows` for the selected profile, excludes item ID 0, and uses the client-visible definition filter. Its private `write_attachment:991-1009` already writes the required three strings and three dwords, including tint fallback. `profile_equipment:303-316` instead writes `(slot,item_id,category)` triples: wrong format for AddPc.
- `startup_world.rs::NpcAttachment:552-560`, `read_npc_attachment:692-700`, `write_npc_attachment:727-734` describe the identical attachment row. `NpcSpawn` itself is a different 35/2 body and must not replace AddPc.

Graph check: project `cartoonmmo`, generation `2026-10-16T01:56:50Z`; inbound trace reaches registry join/update and runner `commit_outcome`. Six relevant source paths had `no_recorded_issue/metadata_match`. The graph is not watched in this session; the route claims above also use direct source reads.

## Framing and offset notation

All scalars are little-endian; no alignment padding. Offsets below start at the **inner opcode**, including its 4-byte family/subtype header. The existing server tunnel is `05 00 01 | inner_length:u32 | inner`; add 7 for full application-packet offsets. AddPc has no opcode-12-style body-length field.

`str(v)` = signed `i32` nonnegative **byte** length followed by exactly those bytes, no terminating NUL. C: `008e7290` and the inline string reads in `0091aa10` check negative lengths and remaining bytes. Our Rust strings are UTF-8; the native length itself does not prove a character encoding.

Define byte lengths `n1,n2,h,r,s,f,c` for first name, last name, head, hair, skin tone, face paint, customization. Let:

- `N = n1+n2`.
- `A = sum(24 + model_bytes + texture_bytes + tint_alias_bytes)` across all attachment rows; excludes the attachment count.
- `V = N+A+h+r`; `W = V+s+f+c`.
- `G` = guild row count; `D = W+12*G`.
- `E` = effect-tag count; `Z = D+72*E`.

Inner length is **`195+Z`**. Empty variable values give 195 inner bytes / 202 tunnel bytes. Native `0092fec3` constructs subtype 1 via `0091d000`; `0092fee3` calls bounded parser `00925380`, which calls `0091aa10` and rejects unread trailing bytes when the dispatch passes strict flag 0. The body reader returns at `0091af49`.

## AddPc field-by-field layout

Every row includes native reader provenance. `C/L` means native shape, Legacy semantic name. Signedness of ordinary dwords follows Legacy hints; the native readers primarily copy their bits. Native object offsets are included to disambiguate field crosswalks.

| Inner offset | Type | Meaning / confidence / native read | Server source or proposed default |
|---:|---|---|---|
| 0 | u16 | Family 35; C, `0091aa1b` -> `008d6830`; Legacy `BasePlayerUpdatePacket.cs:16-19` | (R) protocol constant 35, existing route |
| 2 | u16 | Subtype 1; C, same helper; constructor `0091d029`; P:11 | (R) protocol constant 1 |
| 4 | u64 | Character GUID -> object +0x10; C/L, `0091aa43-0091aa4e`, P:80 | Authenticated `CharacterState.guid`, never account GUID |
| 12 | i32 | FirstNameId; C/L, `008e72bc`, N:20 | `SelfData.name.fields[0]` only if correct for this identity; otherwise (R) 0 for literal-name mode |
| 16 | i32 | LastNamePrefixId; C/L, `008e72e2`, N:21 | Same rule for `.fields[1]` |
| 20 | i32 | LastNameSuffixId; C/L, `008e7309`, N:22 | Same rule for `.fields[2]` |
| 24 | str | First name; C/L, `008e7329-008e7345`, N:24 | `CharacterIdentity.first_name` / current self `.name.first` |
| 28+n1 | str | Last name; C/L, `008e7367-008e7384`, N:25 | `CharacterIdentity.last_name` / current self `.name.last` |
| 32+N | i32 | Model -> +0xa0; C/L, `0091aa7e`, P:84 | `CharacterIdentity.model` / current self `.identity.model` |
| 36+N | i32 | Chat bubble foreground -> +0xa4; C/L, `0091aaa5`, P:86 | (R) 0 until character chat-style state exists; preserves stub, visual default unverified |
| 40+N | i32 | Chat bubble background -> +0xa8; C/L, `0091aacc`, P:87 | (R) 0, same reason |
| 44+N | i32 | Chat bubble size -> +0xac; C/L, `0091aaf3`, P:88 | (R) 0, same reason |
| 48+N | 4*f32 | Position XYZW -> +0xb0; C/L, `0091ab06` -> `008e2410`, P:90 | `CharacterState.position`, (R) homogeneous W=1, matching self data |
| 64+N | 4*f32 | Direction XYZW -> +0xc0; C, `0091ab13` -> `008e2410`; yaw consumer `0092ff2c-0092ff44`; P:91 calls it Rotation | Accepted `CharacterState.rotation` XYZ, (R) W=0; do not encode a quaternion |
| 80+N | i32 | Attachment count; C/L, `0091ab20` -> `009131fa`, P:93 | Existing `Inventory::profile_attachments(active)` count |
| 84+N | rows, A bytes | Attachments; C/L, `00913215` -> `008e86f0`, A:16-21 | Remaining bytes from same helper, active profile only; row detail below |
| 84+N+A | str | Head -> +0xe0; C/L, `0091ab3d-0091ab5d`, P:95 | Identity `.head` / self `.appearance.head` |
| 88+N+A+h | str | Hair -> +0xf0; C/L, `0091ab7f-0091ab9f`, P:96 | Identity `.hair` / self `.appearance.hair` |
| 92+V | i32 | Hair color -> +0x134; C/L, `0091abcd`, P:98 | Identity `.hair_color` / self equivalent |
| 96+V | i32 | Eye color -> +0x138; C/L, `0091abf6`, P:99 | Identity `.eye_color` / self equivalent |
| 100+V | i32 | Unknown3 -> +0x130; C/U, `0091ac1d`, P:101 | (R) 0: no authoritative meaning/source. It **is read**, despite P:34's comment |
| 104+V | str | Skin tone -> +0x100; C/L, `0091ac3d-0091ac5d`, P:103 | Identity `.skin_tone` / self equivalent |
| 108+V+s | str | Face paint -> +0x110; C/L, `0091ac7f-0091ac9f`, P:104 | Identity `.face_paint` / self equivalent; absent = zero length |
| 112+V+s+f | str | Model customization -> +0x120; C/L, `0091acc1-0091ace1`, P:105 | Identity `.model_customization` / self equivalent |
| 116+W | f32 | Max movement speed -> +0x13c; C/L, `0091ad0d`, P:107; spawn consumer stores actor +0x3ec | No current general character-stat source. (R) propose 10.0 to match gateway's configured base at `gateway.rs:988-991`; native tuning gate, not recovered speed |
| 120+W | u8 bool | Underage -> +0x140; C/L, `0091ad4a`, P:109 | Server-owned account age flag / self `.account.underage` only when account-correct; otherwise (R) false until supplied |
| 121+W | u8 bool | Member -> +0x142; C/L, `0091ad76`, P:110 | Authenticated account `.flags.membership` (`bin/osfr-gateway.rs:792,810`); same nonzero status mapping hinted by Legacy `Entities/Player.cs:603-604` |
| 122+W | u8 bool | Referee -> +0x143; C/L, `0091ada2`, P:111 | (R) false: role policy unfinished; never infer from client input or grant admin rights from this display flag |
| 123+W | i32 | Temporary appearance -> +0x144; C/L, `0091adcf`, P:113 | (R) 0: no active model override in current session state |
| 127+W | i32 | Guild count; C/L, `0091ade5` -> `008fc7ff`, P:115 | (R) G=0: guild runtime is not routed |
| 131+W | G*(i32,u64) | Guild key/GUID; C/L, `008fc833`, `008fc86d`, P:119-120 | No rows while G=0; future authenticated guild state only |
| 131+D | i32 | Active profile/job -> +0x16c; C/L, `0091ae09`, P:123 | `GameplayState.profiles.active` / self `.jobs.active_profile_id`; do not guess from weapon or model |
| 135+D | i32 | Title ID -> +0x170; C/L, `0091ae1c` -> `008e866a`, T:14 | (R) 0: titles not routed |
| 139+D | i32 | Title type -> +0x174; C/L, `008e868e`, T:15 | (R) 0, same reason |
| 143+D | i32 | Title name ID -> +0x178; C/L, `008e86b3`, T:16 | (R) 0, same reason |
| 147+D | i32 | Title icon ID -> +0x17c; C/L, `008e86dc`, T:17 | (R) 0, same reason |
| 151+D | i32 | Effect-tag count; C, `0091ae29` -> `0090aa4a`; P:127 writes 0 | (R) E=0: no reviewed public persistent-effect snapshot; self list is a different keyed wrapper |
| 155+D | E*72 bytes | Effect tags; C/U, `0090aa65` -> `008e93e0` | No rows for E=0. Complete structural detail below; do not copy self-data's 81-byte payloads |
| 155+Z | u64 | Mount GUID -> +0x190; C/L, `0091ae53`, P:129 | (R) 0: mounts not routed |
| 163+Z | i32 | Mount seat -> +0x198; C/L, `0091ae84`, P:131 | (R) -1 unmounted, following Legacy `Entities/Player.cs:610-611`; sentinel not independently proved by native consumer |
| 167+Z | i32 | Mount queue position -> +0x19c; C/L, `0091aeac`, P:132 | (R) -1, same unmounted rationale |
| 171+Z | f32 | Name vertical offset -> +0x1a0; C/L, `0091aec0` -> `008d53b0`, P:134 | (R) 0: no mount/nameplate lift |
| 175+Z | i32 | Wield type -> +0x1a4; C/L, `0091aee4`, P:136 | Equipped weapon definition's existing `Definition.wield_type` (`inventory.rs:95,807`); (R) 0 if no weapon. The getter/selection is not yet exposed by inventory; no job-to-wield table should be invented |
| 179+Z | f32 | VIP rank -> +0x1a8; C/L, `0091aef8` -> `008d53b0`, P:138 | (R) 0: no VIP progression source |
| 183+Z | i32 | VIP icon -> +0x1ac; C/L, `0091af1c`, P:140 | (R) 0, same reason |
| 187+Z | i32 | VIP title -> +0x1b0; C/L, `0091af30` -> `008c9d20`, P:141 | (R) 0, same reason |
| 191+Z | i32 | Unknown17 -> +0x1b4; C/U, `0091af3d` -> `008c9d20`, P:142 | (R) 0: meaning/source unknown |

### Attachment row, relative offsets

Let `m,t,a` be the three string byte lengths. Row length = `24+m+t+a`. Native entry reader `008e86f0` is shared by PC/NPC paths. Its speculative map label was checked against all 344 bytes.

| Offset | Type | Meaning / provenance | Server source |
|---:|---|---|---|
| 0 | str | Model name; C/L, `008e8713-008e872f`, A:16 | `Definition.model` |
| 4+m | str | Texture alias; C/L, `008e8751-008e8771`, A:17 | `Definition.texture` |
| 8+m+t | str | Tint alias; C/L, `008e8793-008e87b3`, A:18 | `Definition.tint_alias` |
| 12+m+t+a | i32 | Tint ID; C/L, `008e87e1`, A:19 | Item `.tint`, existing helper falls back to `.default_tint` when 0 (R existing policy) |
| 16+m+t+a | i32 | Composite effect ID; C/L, `008e880a`, A:20 | `Definition.effect` |
| 20+m+t+a | i32 | Slot; C/L, `008e8836`, A:21 | Active `EquippedItem.slot`; not item ID/category |

Equipment join: `EquippedItem(profile_id,slot,item_id)` -> `InventoryItem.id` -> `.definition_id` -> loaded `Definition`. `profile_attachments` already performs this join. Do not use `CharacterIdentity.attachments` unconditionally: those are creation clothing, and can differ from the active job. Native SelfData 011719 explicitly observed that difference. Legacy `src/Sanctuary.Game/Entities/Player.cs:519-568` corroborates active-profile resolution and a weapon flair override; that override is not implemented by our current attachment helper and remains a gap.

There is no separate gender field, five-element customization-ID array, inventory item-ID list or per-job XP/rank list in this AddPc layout. Reuse their resulting model/appearance/attachment/profile values; do not append opcode-12-only fields.

### Effect-tag row, relative offsets (72 bytes)

These fields are structurally complete even though minimal AddPc emits count 0 (R). All values below have **no AddPc server source while E=0**; future values require authoritative effect state. Native `008e93e0` is map-labeled speculative; its full 803 bytes were checked. `startup_world.rs::NpcEffectTag` and `write_effect_tag:736-757` already match this wire sequence. Semantic names in that Rust struct are crosswalks, not independent native proof.

| Offset | Wire type | Meaning / confidence | Native successful read |
|---:|---|---|---|
| 0 | i32 | Instance ID crosswalk; C/U | `008e940e` -> object +8 |
| 4 | i32 | Definition ID crosswalk; C/U | `008e9432` -> +0xc |
| 8 | i32 | Unnamed field 3; C/U | `008e9454` -> +0x10 |
| 12 | i32 | Unnamed field 4; C/U | `008e9476` -> +0x34 |
| 16 | i32 | Unnamed field 5; C/U | `008e9498` -> +0x38 |
| 20 | f32 bits | Unnamed field 6; C/U (NaN check) | `008e94bf` -> +0x3c |
| 24 | i32 | Unnamed field 7; C/U | `008e94f3` -> +0x40 |
| 28 | u8 bool | Unnamed flag 8; C/U | `008e9516` -> +0x30 |
| 29 | u64 | Unnamed field 9; C/U | `008e9540` -> +0x28/+0x2c |
| 37 | i32 | Clock-relative scalar 10; C operation / U units/meaning | `008e9574`; subtracted from the helper's 64-bit result into +0x18/+0x1c |
| 41 | i32 | Clock-relative scalar 11; C operation / U units/meaning | `008e95a6`; same operation into +0x20/+0x24 |
| 45 | i32 | Unnamed field 12; C/U | `008e95d7` -> +0x44 |
| 49 | i32 | Composite-effect ID crosswalk; C/U; AddPc consumer tests object +0x48 | `008e9600` -> +0x48 |
| 53 | u64 | Unnamed field 14; C/U | `008e9625` -> +0x50/+0x54 |
| 61 | i32 | Unnamed field 15; C/U | `008e964e` -> +0x58 |
| 65 | i32 | Unnamed field 16; C/U | `008e9670` -> +0x5c |
| 69 | u8 bool | Unnamed flag 17; C/U | `008e9697` -> +0x60 |
| 70 | u8 bool | Unnamed flag 18; C/U | `008e96be` -> +0x61 |
| 71 | u8 bool | Unnamed flag 19; C/U | `008e96ed` -> +0x62 |

The self-data effect **payload** is 81 bytes because `00a18cc0` wraps this record with nine more bytes; its map key adds another four outside the payload. AddPc calls `008e93e0` directly through the list reader. Therefore neither 81-byte payloads nor keyed 85-byte rows can be spliced into AddPc. [[Self Data Layout]]'s prose calling the base record 68 bytes is stale; its 81-byte payload total remains compatible with the 72+9 walk above.

## Opcode 125 and its two relay bytes

Strict inner length = 36, full server tunnel = 43. Native `008e5940` reads all these fields, sets position W=1 internally at `008e5b23`, and direction W=0 internally at `008e5b31`. No timestamp or zone appears on wire. Legacy M:24-32 corroborates order; its `Quaternion` type name does not establish quaternion semantics.

| Inner offset | Type | Meaning / provenance / confidence | Relay source |
|---:|---|---|---|
| 0 | u16 | 125; C/L, `008e5966`, M:24 | (R) protocol constant |
| 2 | u64 | Actor GUID; C/L, `008e598f`, M:26 | Authenticated subject's character GUID |
| 10,14,18 | 3*f32 | Position X,Y,Z; C, `008e59c1`, `008e59f5`, `008e5a29` | Accepted/clamped `MovementState.position`, never raw rejected report |
| 22,26,30 | 3*f32 | Direction X,Y,Z; C, `008e5a5d`, `008e5a91`, `008e5ac6`; yaw setter `0095dae0` | Accepted/sanitized `MovementState.direction`; do not introduce quaternion conversion |
| 34 | u8 | State bitfield -> object +0x30; C/L, `008e5af9`, M:31 | Validated state's `.state`, retained exactly as a byte |
| 35 | u8 | Opaque state/service byte -> object +0x31; C/U, `008e5b1b`, M:32 | Validated state's `.unknown`, retained exactly as a byte |

State facts, without invented walk/run/jump enums:

- C: sender `00934e6f-00934e97` builds bit 3 from actor +0x18b8 and bit 4 from +0x650; `00934e99-00934ed7` combines bits 7/6/5/1/0 from cached booleans. Bit 2 is not set by that ordinary branch; other native sender branches use it.
- C: receiver `0090de90` uses bits 7,6,5,2,1,0 for motion/controller choices. Bit 0 selects zero versus `GetEntitySpeed` for its motion speed argument. These operations do not by themselves name a locomotion mode.
- C: receiver `0090e0f4-0090e0f9` writes bit 4 to actor +0x650; `008b3180` scales actor +0x3ec by 0.25 when set, otherwise 1.0. `nmq bytes 0175d738 4` = `0000803e`; `0175b420` = `0000803f`. Do not replace server movement authority with a client speed claim.
- C: sender `00934e81-00934e87` copies client-service byte +0x3e4 as wire byte 35. Its gameplay meaning is U. No name/enum is justified. Retaining it is distinct from granting it authority.
- Existing [[Movement Validation]] reports captured values 0/1 in byte 35 and multiple state-byte values; those aggregate capture counts were not rerun here. The reconstructed exhaustive byte-pair check below is fresh, not a real-trace claim.

When a validated update changes only state/unknown, the server shall relay it to already-visible viewers. When movement is ignored or rejected, the server shall retain the last accepted public pose/state. A server teleport may change pose while preserving the latest accepted flags, or explicitly reset flags under documented (R) transition policy; no arbitrary client state may cross to another session. State bytes stay transient; no save-schema migration is required for this task.

## Minimal function-level Rust change list (lead/build handoff only)

No code changed in W77. A `players.rs`-only patch cannot obtain the required data; these integration seams are mandatory and remain lead-owned.

1. **`players.rs::PlayerSession`, `join`:** accept/store a small authenticated public presentation snapshot (identity fields, active profile, already-built active-profile attachments, speed, account display flags, wield type). Store the two transient movement bytes, initially 0 (R) until an accepted report. Do not clone all `SelfData` or inventory into every visibility row. Reuse the existing identity type/attachment wire helper; no second item catalogue or attachment resolver.
2. **`build_add_pc`:** replace existing placeholders at their present positions. Append the helper's attachment count+rows once. Correct direction float4 as above. Keep unsupported guild/title/effect/VIP fields explicit (R) defaults. Use checked count/length conversion at the snapshot boundary; existing `put_string` can panic on oversized lengths and is only suitable after validation. Keep output tunnel format unchanged.
3. **`update_pose`:** accept the two validated bytes and include them in its equality early return. `build_position_update` writes those stored bytes instead of `[0,0]`. Keep authenticated GUID and accepted pose; no raw-packet relay. Preserve the existing visibility set/diff policy.
4. **`diff_messages`:** same routing; classify a state-only accepted change as a position delivery. No new spatial system, packet family or dispatcher.
5. **Lead integration in `Gateway` / `SharedWorld`:** expose the existing accepted `MovementState` through a read-only accessor, compare the previous and staged values in `commit_outcome`, and update registry state even when XYZ/direction did not change. Build presentation from the same identity/profile/inventory inputs as `build_self_message`, after staged changes succeed. A narrow gameplay accessor can call private `Inventory::profile_attachments` and expose the selected weapon's existing wield value; its item/slot selection must be checked against the existing equip path. Pass that snapshot at `join_player`. With no gameplay profile, use the same explicit self-data fallback profile rather than defaulting AddPc to job 0.
6. **Appearance changes after joining:** initial snapshots are insufficient for live job/equip changes. Refresh registry presentation after a committed job/equip change and fan the existing appropriate update messages to viewers. If implementation is restricted to initial AddPc only, record live-change broadcasts as an open acceptance gap; do not claim complete multiplayer outfit sync. Do not use duplicate AddPc as a guessed update protocol without a native check.

Security review (secure-code-guardian + security-reviewer, scoped to this proposed network change): hostile GUID/pose/flags, count/length overflow, stale/template identity leakage, and cross-session presentation are the threat model. Existing per-session GUID check and movement validation must remain before fanout. Bound strings/counts and total output size, validate finite pose/speed, and return errors rather than panic for malformed input. Do not add permissions from appearance flags, relay rejected movement, or change loopback/admin tooling. `notes/Design/Security Model.md` was absent during this read; this local analysis does not satisfy Build Plan 1.0 or authorize remote exposure.

Own-diff review: **0 executable changes, 0 secrets/player names/chat added.** Two existing integration defects (lost flags and suppressed state-only updates) and the quaternion error are documented, not fixed. Active network tests, native control and blanket scans were not run. The embedded offline oracle below validates bounded length handling; it is not a production parser/security sign-off.

## Runnable reconstructed wire check

This original stdlib-only oracle is embedded here to respect the one-file ownership limit. It reconstructs the independently inspected schema, round-trips every field including nonempty nested rows, checks formula offsets and strict consumption, rejects every truncated sample, and checks all 65,536 state-byte pairs. It contains no player name or chat string; the two literal-name fields are empty. Asset strings are synthetic bytes, not game assets. Native rendering remains a separate gate.

Run from `project` without creating another file:

```powershell
@'
from pathlib import Path
text = Path('notes/Research/AddPc Layout.md').read_text(encoding='utf-8')
code = text.rsplit('<!-- W77-CHECK -->', 1)[1].split('```python', 1)[1].split('```', 1)[0]
exec(compile(code, 'W77-embedded-check', 'exec'))
'@ | python -I -
```

<!-- W77-CHECK -->
```python
import hashlib
import math
import struct

ATT = [('model', 's'), ('texture', 's'), ('tint_alias', 's'),
       ('tint', 'i'), ('effect', 'i'), ('slot', 'i')]
GUILD = [('key', 'i'), ('guid', 'Q')]
EFFECT = [(f'f{i}', t) for i, t in enumerate(
    ['i'] * 5 + ['f', 'i', 'B', 'Q'] + ['i'] * 4 + ['Q', 'i', 'i', 'B', 'B', 'B'], 1)]
PC = [('family', 'H'), ('subtype', 'H'), ('guid', 'Q'),
      ('name_ids', '3i'), ('first', 's'), ('last', 's'), ('model', 'i'),
      ('bubble', '3i'), ('position', '4f'), ('direction', '4f'), ('attachments', ATT),
      ('head', 's'), ('hair', 's'), ('hair_color', 'i'), ('eye_color', 'i'),
      ('unknown3', 'i'), ('skin', 's'), ('paint', 's'), ('custom', 's'), ('speed', 'f'),
      ('underage', 'B'), ('member', 'B'), ('referee', 'B'), ('temporary', 'i'),
      ('guilds', GUILD), ('profile', 'i'), ('title', '4i'), ('effects', EFFECT),
      ('mount', 'Q'), ('seat', 'i'), ('queue', 'i'), ('name_offset', 'f'),
      ('wield', 'i'), ('vip_rank', 'f'), ('vip_icon', 'i'), ('vip_title', 'i'), ('unknown17', 'i')]

def encode(schema, values):
    out = bytearray()
    for key, kind in schema:
        value = values[key]
        if isinstance(kind, list):
            out += struct.pack('<i', len(value))
            out += b''.join(encode(kind, row) for row in value)
        elif kind == 's':
            out += struct.pack('<i', len(value)) + value
        else:
            out += struct.pack('<' + kind, *(value if isinstance(value, tuple) else (value,)))
    return bytes(out)

def decode(schema, packet):
    offset = 0
    def take(n):
        nonlocal offset
        if n < 0 or n > len(packet) - offset:
            raise ValueError('length')
        data = packet[offset:offset+n]
        offset += n
        return data
    def count():
        n, = struct.unpack('<i', take(4))
        if n < 0 or n > len(packet) - offset:
            raise ValueError('count')
        return n
    def row(fields):
        result = {}
        for key, kind in fields:
            if isinstance(kind, list):
                result[key] = [row(kind) for _ in range(count())]
            elif kind == 's':
                result[key] = take(count())
            else:
                fields_read = struct.unpack('<' + kind, take(struct.calcsize('<' + kind)))
                result[key] = fields_read[0] if len(fields_read) == 1 else fields_read
        return result
    result = row(schema)
    if offset != len(packet):
        raise ValueError('trailing bytes')
    if schema == PC and (result['family'], result['subtype']) != (35, 1):
        raise ValueError('opcode')
    return result

def sample(fields, populated):
    result = {}
    for number, (key, kind) in enumerate(fields, 1):
        if isinstance(kind, list):
            result[key] = [sample(kind, True)] if populated else []
        elif kind == 's':
            result[key] = bytes([64 + number]) if populated and key not in ('first', 'last') else b''
        else:
            value = number % 2 if kind == 'B' else number + 0.5 if kind.endswith('f') else number
            result[key] = (value,) * int(kind[0]) if kind[0].isdigit() else value
    return result

empty = sample(PC, False)
empty.update(family=35, subtype=1)
assert len(encode(PC, empty)) == 195
full = sample(PC, True)
full.update(family=35, subtype=1, head=b'\xc3\xa9', seat=-1, queue=-1)
full['guilds'].append({'key': 99, 'guid': 101})
wire = encode(PC, full)
assert len(encode(EFFECT, full['effects'][0])) == 72
assert len(wire) == 195 + 27 + 6 + 24 + 72 == 324
assert struct.unpack_from('<i', wire, 32)[0] == full['model']
assert struct.unpack_from('<4f', wire, 64) == full['direction']
assert struct.unpack_from('<i', wire, 111)[0] == 2  # head byte length
assert struct.unpack_from('<i', wire, 188)[0] == full['profile']
assert struct.unpack_from('<Q', wire, 284)[0] == full['mount']
for packet in (encode(PC, empty), wire):
    assert encode(PC, decode(PC, packet)) == packet
    tunnel = b'\x05\x00\x01' + struct.pack('<I', len(packet)) + packet
    assert len(tunnel) == 7 + struct.unpack_from('<I', tunnel, 3)[0]
    assert tunnel[7:] == packet
assert decode(PC, wire) == full
bad = [wire[:i] for i in range(len(wire))] + [wire + b'\0']
for offset, value in [(80, -1), (80, 2**31-1), (111, -1), (160, -1), (208, -1)]:
    mutated = bytearray(wire)
    struct.pack_into('<i', mutated, offset, value)
    bad.append(bytes(mutated))
for packet in bad:
    try:
        decode(PC, packet)
    except ValueError:
        continue
    raise AssertionError('malformed packet accepted')
for state in range(256):
    for unknown in range(256):
        values = (125, 0x102030405060708, 1., 2., 3., 0., 0., 1., state, unknown)
        packet = struct.pack('<HQ6fBB', *values)
        assert len(packet) == 36 and packet[34:] == bytes((state, unknown))
        assert struct.unpack('<HQ6fBB', packet) == values
for heading in (0., math.pi/2, math.pi, -math.pi/2):
    facing = (math.sin(heading), 0., math.cos(heading), 0.)
    assert math.isclose(math.atan2(facing[0], facing[2]), heading, abs_tol=1e-7)
print('AddPc: 195/324 inner bytes; nested row 72 bytes; 2 byte-exact round trips PASS')
print(f'Malformed AddPc: {len(bad)} rejected PASS; opcode 125: 65536 byte pairs PASS; facing: 4 PASS')
print('Reconstructed populated AddPc SHA256:', hashlib.sha256(wire).hexdigest())
```

This check proves the reconstructed layout's internal consistency, not that the production stub now sends these values. The production regression must compare built AddPc to an independent expected sample/decoder, not repeat `build_add_pc`'s writer line for line as current `tests/players.rs::expected_add_pc` does.

## Validation record and remaining gates

Commands run from `project` (all read-only except writing this note):

- `Get-FileHash -LiteralPath '../lab/working/OSFR.Client/OSFR Client/FreeRealms.exe' -Algorithm SHA256`: hash above, matches documented original.
- `git -C '../lab/references/FreeRealms-Legacy' rev-parse HEAD`: pin above.
- `python -I scripts/native_map/nmq.py packet 'AddPc|UpdatePosition'`: 35/1 AddPc catalogue matches. `command AddPc` has no match; `command 125` is an unrelated MatchCommand ID, **not** the standalone packet reader. Use the packet vtables/dispatch and `008e5940` instead.
- `nmq show`/read-only SQLite `fn.c`/`label` inspection: `0091aa10`, `008e5940`, `008e7290`, `009131c0`, `008fc7c0`, `008e8640`, `0090aa10`, `008e86f0`, `008e93e0`, `00925380`, `0092f460`, `0090de90`, `00934a20`, `008b3180`.
- `python -I scripts/native_map/nmq.py dis` instruction checks: `0091aa10 700`, `0091accc 638` (start misaligned; corrected with `0091acca 36`), `0091af42 10`, `0091d000 90`, `0092fe80 320`, `0092ff2c 31`, `00925380 97`, `008e7290 279`, `009131c0 145`, `008e86f0 344`, `008fc7c0 240`, `008e8640 167`, `0090aa10 145`, `008e93e0 803`, `008e5940 506`, `0090e0f0 24`, `00934e60 170` (first byte misaligned; flag claims begin at valid boundary `00934e6f`), `008b3180 58`, `00930080 120` (speed assignment at `009300c4-009300cd`), `0090df60 270` (start misaligned; motion branches read from `0090df62` onward), `0095dae0 80`. Read ranges extending beyond a return were ignored for that function.
- `python -I scripts/native_map/nmq.py bytes 0175d738 4` / `bytes 0175b420 4`: quarter/unity float bytes above.
- Embedded `python -I -` check: exit 0, output below. The initial extraction command selected its own marker literal and executed an empty expression (exit 0, no output); corrected to `rsplit` and rerun. Only the printed successful run is counted.

```text
AddPc: 195/324 inner bytes; nested row 72 bytes; 2 byte-exact round trips PASS
Malformed AddPc: 330 rejected PASS; opcode 125: 65536 byte pairs PASS; facing: 4 PASS
Reconstructed populated AddPc SHA256: 7deb3b603b1d01a2b667743b44a669c920695b8039db1761a93f371284c18ac7
```

Final in-memory note audit (`python -I -`): `78 field rows each cite a native address; embedded Python compiles; secret-pattern scan clear`. This automatic check verifies citation presence, not semantic correctness; the manual native/Legacy checks above provide the field evidence. Final scoped `git status --short -- 'notes/Research/AddPc Layout.md'` shows this new note untracked. No commit was made.

Content and Rust were not changed. Cargo/native/bot checks were not run: this worker owns only this note, and `content_check.py` writes a separate report even by default. No content validation pass is claimed. Existing native self data is a reuse baseline, not proof of other-player rendering.

Applied skills: spec-miner for source/route crosswalk and requirements, reverse-engineering for native byte verification, codebase-memory for the bounded inbound trace plus coverage check, secure-code-guardian/security-reviewer for threat boundaries and the own-note review, Ponytail full for existing-helper reuse, and Caveman for concise progress. RTK used normal commands first. No asset reader or asset production was needed, so game-recon, community format research and asset-pipeline were not invoked.

Implementation acceptance tests required from the builder/lead:

1. `players` byte oracle: distinct synthetic model/name IDs/appearance, nonempty attachment and title fields, empty and populated guild/effect lists; exact offsets/length, decode -> encode equality, and strict truncated/negative-count rejection where a decoder is exposed. Keep any real player names/chat out of checked-in fixtures; use empty literal names plus IDs for wire tests, and private native captures for actual names.
2. Active-profile outfit test: profiles with different equipment produce different attachment lists; empty active profile remains empty even if creation/profile-1 clothing exists. Tint fallback and wield type match existing inventory behavior. Do not require catalogue expansion for this change.
3. Registry state test: stationary subject transitions state 0 -> 2 and unknown 0 -> 1; both produce opcode-125 deliveries to existing viewers with unchanged pose. Same complete state produces no duplicate; never send the subject to itself. Every u8 value must survive storage/serialization without bool conversion.
4. Gateway/bot route test on an isolated loopback gateway (`--studio-port 0`): two authenticated distinct GUIDs, accepted state-only packet reaches other viewer; wrong GUID, truncated/extra bytes, non-finite position, rejected movement and stale-teleport reports never leak into relay. A clamped report relays only the committed pose and corresponding accepted flags. No permissions from client bytes. Run focused tests through `scripts/cargo.ps1` when code ownership permits.
5. Lead native acceptance: native client + bot, then two native clients, check real name/appearance/outfit/job, initial facing, moving/stopping/turning, and live job/equipment change. Record run IDs/captures and independently reviewed rendering. W77 ran no client and makes no animation/rendering claim.

Explicit unknowns: chat bubble defaults, Unknown3, Unknown17, exact original movement speed, underage/referee account policy, mount sentinel consumer semantics, VIP/title/guild sourcing, effect-tag semantic fields/time units, state bits' named locomotion meanings and byte-35 meaning. No historical claim is made for (R) constants. Direct active-profile attachment encoding is reusable now; flair shard override, weapon selection/getter and post-join appearance broadcasts still need implementation/native review.

Ponytail: reused native map/disassembler, existing self-data/identity types, inventory attachment join/writer, movement validation/state and registry fanout. Did not build a new parser module, dependency, item resolver, spatial system, saved-state schema, gateway or native harness.
