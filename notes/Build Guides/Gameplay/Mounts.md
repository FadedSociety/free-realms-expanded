---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Mounts

Part of [[Build Guides]] › Gameplay. Related: [[Pets]], [[Movement and Pathfinding]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the native-map labels for a function, taken from the accepted records. Where a function has no accepted name or the label says too little, the line says "read from pseudocode this pass": the behaviour was read directly from the stored decompilation, and the confidence is then my own, usually `likely`. Source tags: *client* = shipped executable (labels, shard findings, pseudocode, RTTI class names), the shipped `Resources` files, the asset manifest, or client locale text; *video* = a reviewed frame record (frame key given); *wiki* = the fan wiki. Precedence is client, then video, then wiki. *(R)* marks reconstructed server policy that is ours, not original behaviour. Packet class names below are the client's own RTTI vtable names (for example `PacketMountResponse::vftable`) and the sub-opcodes are the constants the client constructors pass, not names from any emulator or catalogue; several shard notes cite a community packet catalogue, and those lines were not used. No original server code exists and no mount traffic has been captured, so every statement about what the server sent or decided is a hypothesis built from what the client requires. No player names, chat or player-chosen ride names are recorded. Shard S0203 (review pending) was not used. Related: [[Items and Inventory]], [[Pets]], [[Movement and Pathfinding]], [[Animation]], [[Actors and NPCs]], [[Coin Store and Purchases]], [[Minigame Framework]], [[Combat and Abilities]], [[Character and Customization]], [[Gateway and Transport]], [[Scripting (Lua)]], [[Housing]], [[Kart Racing]].

### Overview

The game and its text call mounts "rides"; the code, the packet family and a few locale lines call them "mounts" (client locale: "Mount", "Mounts", "Select Mount", "My Rides", "Rides", "Obtain a mount."). A ride is a creature or vehicle the player owns, calls up beside the character and sits on. It is a native client feature: one packet family (opcode `0xa8`, 168), a Lua module, two UI data sources and a per-character mount list.

- **What a ride is.** Rides are permanent: no cooldown, not inventory objects, cannot be sold or traded, and duplicates give no benefit (wiki). The client agrees on the shape: the roster is a separate list of mount records sent with the player (see Data and formats), and a ride is called by its mount id, not by an item instance. The client also has an item path (an item of the right category spawns a ride, see Spawning), so how the wiki's "not in your inventory" fits is open.
- **Getting one.** Marketplace in Station Cash (Rides category with sub-lists for Balls, Tigers, Hoverboards, Unicorns, Horses, Dragons, T-Rex and Pegasus; video), in-world stables (client locale: "head to one of the in-world stables to check out the T-Rex and Wyvern rides"; NPC titles "Stable Hand" and "Stable Guide"), Mystery Chests, Trading Card Game virtual-reward cards, quest and event rewards, and one collection prize (wiki; client locale; video). Client locale also has an achievement ladder, "Obtain a mount.", "Obtain 5 mounts.", "Obtain 15 mounts.", "Obtain 25 mounts."
- **Summoning.** The player clicks the stable icon on the game dock ("Rides" label on the dock bar in video, key `pLF-dUhMFWs/00466`), picks a ride from the paged "Rides" menu and the ride appears and the character mounts ("automount", see Spawning). Wiki: clicking the icon again (it changes to a saddle with a red slash) removes the ride. Help text in the client locale: "After ya buy yerself a ride, just click that stable icon on yer game dock and add a ride."
- **Riding.** The character sits in a seat on the ride and uses the ride's locomotion set instead of its own (stand, walk, run, swim, jump, glide and so on). Wiki: up to 150 percent of normal speed, no stacking with other speed boosts, no emotes, no weapon use, no tool belt use. The client has no speed number for rides that I could find (Open questions).
- **Mount kinds.** The shipped `AnimationTypes.xml` defines 29 mount types, each a prefix for a full animation set (saddled ground and air, seated, surf, T-Rex, hydra, bat, wolf, bicycle, chopper, broom, hoverboard, flying carpet, tank, jet, and others, listed under Data and formats). The wiki sorts rides as mechanical, non-winged, exotic or winged.
- **Gliding.** Winged rides (bat, dragon, feathered drake, pegasus, phoenix) can be taught to glide with a Training Saddle of Gliding, earned from a five-step daily quest series from the Ride Trainer in Seaside (client locale; wiki; video).
- **Trials and membership.** The client has a timed "Mount Trial" (locale "Mount Trial will end in %timeLeft minutes." and "Mount Trial has ended.") and a members-only flag on each mount record. Wiki: most rides need membership to buy and use, and an owned ride stays usable after membership lapses.
- **Upgrades.** Each mount record has "IsUpgradable" and "IsUpgraded" flags, a Lua `Mount.Upgrade(mountId, upgradeItemDefinitionId)` call and a server "upgrade successful" reply. What an upgrade changes is unknown (Open questions). A video advert reads "Upgrade Now for Power Mount" (video).
- **Where rides stop working.** Wiki: not in a minigame, dungeon or duel. The client forces a dismount when a Flash minigame starts and when an ability starts (client, below).
- **Not covered here.** Pets ([[Pets]]), karts and other race vehicles ([[Kart Racing]], [[Demo Derby]]), and housing furniture seats. Housing seating items are furniture, not mounts, although a Lawn Chair is sold as a ride (mount type `lawnchair`).

### How the client implements it

#### Main classes and objects

| Class or object | Role | Evidence |
|---|---|---|
| `MountBasePacket` family (`0089f9c0`, 168) | Family constructor: writes `0xa8` into the packet id slot and the sub-opcode into the next slot. All mount packet classes call it. | confirmed (S0219, S0332) |
| `MountHandler` (receive switch `00b06100`, 2942 bytes) | The only receive switch for the family. Reads a u16 and a u8 header (`00b04c20`) and switches on the sub-opcode: 2, 4, 5, 9, 10, 11, 13, 15. Its only mapped caller is the thunk `00a10980`; the link from the main packet dispatcher to that thunk was not traced. | confirmed (S0333) |
| `MountClientInfo` store (`HashListMap<int,MountClientInfo,16,-1>`, ctor `00b03cd0`, insert `00b050d0`, remove `00b04b70`, remove all `00b05260`) | The client's mount list for the local player: 16 buckets, key = mount id (`id and 0xf` picks the bucket), 0x40-byte nodes. | confirmed RTTI, likely members (S0332) |
| Mount client object (`00b05cb0`, created by the client-data constructor `00a1de60`) | Holds the store, the automount id (`SetAutoMountId`, `00b03c90`, stored at +0x64), and the trial end time (64-bit at +0x58/+0x5c; +0x60 holds the received value and then the last announced remaining value). The request timestamp used by the debounce lives on the local player object (+0x68/+0x6c, set by `00b04240` and by each Lua call). The data sources keep pointers back to the handler. | read from pseudocode this pass; `00b03c90` confirmed |
| `ProxiedCharacter` mount state | Each character actor (rider or mount) keeps a pointer at +0x1870 to a 0x20-byte mount-state block, created by `Mount` (`00972d30`, confirmed) and freed by `RemoveRider` (`009710b0`, confirmed). The first two dwords are the mount's guid pair. A pending-mount group of five values sits at +0x17a8..+0x17b8 (`SetPendingMountInfo`, `0095ad80`, confirmed). | S0259, S0260, pseudocode |
| `MountListDataSource` (`BaseClient.MountList`, ctor `00cc4200`) | UI list of owned rides (columns below). Keeps its own filtered copy of the store, rebuilt lazily by `00cc3c70`. | confirmed (S0415) |
| `CurrentMountDataSource` (`BaseClient.CurrentMount`, ctor `00cc42c0`) | UI view of the ride the player is on: columns `CurrentMountGuid`, `Name`, `ImageSetId`. | confirmed ctor; columns read from `00cc39c0` |
| Lua module `Mount` (registrar `00bff480`, table `018282a0`) | `Spawn` (`00bff6f0`), `Despawn` (`00bff8e0`), `Dismount` (`00bffa60`), `EnableUpgradableMountsFilter` (`00bff440`), `Upgrade` (`00bffbc0`). Table decoded from the binary this pass. | `00bff480` confirmed; `Upgrade` confirmed; others unknown or likely |
| Mount type table (`MountTypeTableLoad` `007a7e40`, builder `007a2cd0`) | Loads the `MountTypes` section of `AnimationTypes.xml` into a list at +0x4968 of the animation-parameter object (record: id at +0x10, animation slot prefix string, description string). | confirmed (S0173, S0174) |
| `Mountable` actor-definition block (reader `007e4000`) | Per-ride asset data: a minimum occupancy and a list of seats. | confirmed (S0185) |
| Movies | `gamedock.gfx` (the dock with the stable icon) and the category icons `icon_category_mounts_*` (plain and "flying" variants) are in the asset manifest. No separate rides movie name was found; the Rides panel is Lua-driven. | asset manifest |

#### Spawning a ride and the automount handshake

1. **Lua entry.** `Mount.Spawn(id)` (`00bff6f0`, label unknown, behaviour read from pseudocode) takes one integer. It first asks `00a2e540` whether the last mount or dismount request was less than 1000 timer units ago (the label for `00a2e540` says: it returns 1 only when the elapsed low dword is non-negative and under 1000; the timer is the 64-bit timer `007027f0`, assumed to be milliseconds). If so, it does nothing except fire the Lua event `GameDock:SetMountButtonCallback` with argument 2, to reset the dock button.
2. **Otherwise** it stores the timestamp (player-side field +0x68/+0x6c), builds and sends a `PacketDismountRequest` (168/3) first, builds a `PacketMountSpawn` (168/6, constructor `00b038c0`, confirmed in S0332) with the ride id, calls `SetAutoMountId(id)` (`00b03c90`, confirmed), logs "Lua Spawn and automount: id %d" to `ClientMountLog.txt` and sends the spawn through the tunnel wrapper `00b05ea0`. Sends happen only when the gateway pointer at client +0x280 is non-null. So a spawn always asks the server to drop the current ride first (likely).
3. **Server spawns the ride actor.** The mount is a separate world actor (a proxied character). Nothing on the client builds it from the spawn packet; it arrives through the normal actor and player-update paths, and the mount list entry gets its guid (field +0x20 of the record, see Data). This ordering is inferred (speculative).
4. **Automount.** When the mount actor's state update runs (`0096de80`, a `ProxiedCharacter` vtable slot; label name `UpdateStateAndMountLog` from S0258), the client checks that the actor's model is mountable (the `Mountable` block has at least one seat) and that the local player exists. It then calls `ListEntryMatchesIdAndGuid` (`00b047b0`, likely) with the automount id and the actor's guid; on a match it adjusts the player's facing (likely) and sends `SendMountRequest` (`00b04290`, likely) with the actor's guid. The log lines are "Mountable npc ready, automount %d, guid %I64u" and "BaseClient::SendMountRequst: target %I64u". The packet is `PacketMountRequest` (168/1, `008b3730`, confirmed). Read from pseudocode this pass.
5. **Click to mount.** A controller handler (`008bb3a0`, class `InitiateMountControllerHandler`, label unknown) does the same for a stored target guid: when the local player has no mount state (or the default pair) and the target actor exists, it calls `SendMountRequest` with that guid and a flag of 1. This is the path for mounting a ride that is already standing in the world (such as a mountable NPC or a ride the player clicked). Read from pseudocode.
6. **Mount response.** The server's `PacketMountResponse` (168/2) names the rider and the mount by guid. `00b06100` looks both up in the proxied-character table (`00984960`). If either is missing it logs "failed. Source or target NULL". Otherwise it logs "PacketMountResponse: rider %s (%llu), mount %s (%llu), queue position %d" and acts on the result field. Result 1 stores one value on the rider (+0x17bc) and calls `Mount` (`00972d30`, confirmed) with the seat. Result 0 either runs the local dismount (`00973050`) or logs a failure, depending on a predicate (`00957f90`, unnamed). Other values do nothing. If the rider is the local player, the dock datasource flag is cleared (`00cc3a00`) so the button refreshes.
7. **Spawn from an item.** `ProcessItemRecordListActions` (`00a1ec00`, likely) groups the incoming item records by category and, for one bucket, calls `SpawnMountByItemDefinitionId` (`00b06db0`, confirmed), which sends 168/8 with the first record's definition id and a flag byte of 1 (log "SpawnMountByItemDefinitionId: item %d"). That is how buying or claiming a ride item mounts it at once. Which category value is the mount bucket is not proven (the [[Items and Inventory]] guide lists the case values).
8. **Re-mount after item use.** In `UseItemSelectedInUi` (`00bdc060`, likely) and its sibling `00bdc420`, for items outside the equipment, style card and a few other categories that pass a predicate (`01082360`, meaning unresolved), the client looks up the current mount's id by guid (`ListLookupIdByGuid`, `00b047f0`, likely), keeps it at the owner object's +4 and sends a dismount. `ClearPreviewAndFlushMount` (`00bdc3c0`, speculative) later calls `SendPendingMountSpawn` (`00bdc330`, likely), which sends 168/6 with that id and resets the field to -1. The effect is a dismount and a re-spawn of the same ride around some item uses; the exact trigger is unresolved.

#### Riding, attachment and animation

- **Attaching the rider.** `Mount` (`00972d30`, confirmed) logs "Mount, rider %s (%I64u), mount %s (%I64u), rider state %X", returns early if the rider already has mount state, checks that both actors are ready and that the requested seat index is below the seat count of the mount's model (`mountModel+0x4e8`), allocates the 0x20-byte mount-state block (`0095ac30`), registers three animation slots from it (`EnsureAnimSlotRegistered`, `00786c30`, likely), (under a further check, `008c4930`) stores the rider's guid on the mount actor (+0x17a0/+0x17a4), then calls `TransitionIndexedActorState` (`009722d0`, likely). `UpdateSeatAttachment` (`00971250`, likely) validates the seat against the model seat count (logging an error on mismatch) and composes the seat transform; `GetSeatTransform` and `GetSeatTransformSecondary` (`00961b70`, `00961c40`, likely) return the seat matrices used to place the rider. All client-side; no packet is involved.
- **Pending mount.** A player who is already riding when they come into view is handled through `SetPendingMountInfo` (`0095ad80`, confirmed): the base player update dispatcher (`0092f460`, the accepted label does not mention mounts) calls it from one branch with five values. The per-frame `ProxiedCharacter::Process` (`00973200`, confirmed) later calls `Mount` with the stored values once the mount actor and its model exist, then clears the pending guid. Which player-update subtype carries the five values is not traced; `0092f460` also calls `RemoveRider` in two sibling branches. Read from pseudocode this pass.
- **Dismount.** `Dismount` (`00973050`, label unknown) logs "Dismount, rider %s (%I64u), rider state %X", marks the mount state, calls `RemoveRider` (`009710b0`, confirmed) if the mount actor still resolves, restores the rider's movement state and clears the dock flags (`00b04310`, likely). The server's `PacketDismountResponse` (168/4) reaches it; the client's own `SendDismountRequest` (`00b06cc0`, confirmed) does not dismount locally and waits for that reply.
- **Triggers that send a dismount.** `HandleDismountInput` (`00b06d70`, likely): looks up local action `0x27` and sends 168/3 when its condition fails; reached from `UpdateActionAndRelatedState` (`009607c0`, likely) when the character's mount state matches a condition that is not resolved. `StartFlashMiniGameUi` (`009bd650`, confirmed) always sends 168/3 when a Flash minigame starts, and `HandleMiniGameStart` (`009beb70`, confirmed) may. `AbilityProcessor::DispatchTargetModeAndSend` (`00a35600`, likely) sends one first when the player is mounted and an internal flag (+0x70) is set. `AbilityProcessor::SendStartAbilityRequest` (`00a35150`, likely) refuses to start while `00a2e540` says a mount or dismount went out within 1000 timer units. `Mount.Dismount` and `Mount.Despawn` (`00bffa60`, `00bff8e0`) both send 168/3 under the same debounce; they differ only in the argument they pass to the button-reset event (1 for Dismount, 3 for Despawn), their log line, and that `Despawn` requires one Lua argument while `Dismount` takes none, so the original server must have told them apart by state, or not at all (Open questions).
- **Animation.** `AnimationTypes.xml` has an animation type `Mounts` (id 18) and a `mountTypeId` attribute on each mount animation slot; a slot is named `<prefix>_<set name>`, for example `saddleground_loc_walk` (id 7202). The loader (`AnimationParametersLoadFiles` `007aa2d0`, confirmed) reads the types, then `MountTypes`, then the tree. The rider and the ride play separate clips: the asset manifest has `<ride>_mount_loc_*.gr2` and `human_m_<ride>_mount_loc_*.gr2` pairs (for example `horse_mount_loc_run` and `human_m_horse_mount_loc_run`). `AnimationTransitionRules.xml` contains one rule whose comment says the stand-to-stand transition covers going from a normal stand to a mounted stand and back.
- **Seats.** The `Mountable` block (`007e4000`, confirmed) holds a MinOccupancy value (valid 1 to 99; the loader logs "Mountable: invalid MinOccupancy = %d" otherwise) and a list of seats, logging "Mountable: no seats specified" if empty. Each 0x74-byte seat element holds strings and two direction vectors. This is asset data, not sent; the server needs only agree on ride and seat indexes (S0185). A "queue position" appears in the response log, which fits a multi-seat design, but no shipped multi-seat ride is identified (Open questions).

#### Trials, upgrades and the list

- **List.** `PacketMountList` (168/5) replaces the whole store: `00b06100` parses it, then `00b05600` empties the store and copies every parsed record into it, then clears the list and current-mount data source flags so the UI rebuilds (`00cc39a0`, `00cc3a00`). The same mount list is also part of the self packet at login (next paragraph).
- **Login.** The `SendSelfToClient` body (message id 12, reader `00944fe0`) contains a counted list of mount records, read with `008e74c0` right after the pet section and installed by the same copy routine `00b05600` (read from pseudocode this pass; the Pets and Character guides cover the surrounding body). So a client learns its rides at login without any 168 packet.
- **Trial.** `StartTrial` (168/10) records an end time (current time plus a received value) at +0x58 and fires the Lua event `MountHandler:StartTrial`. `UpdateMountTrialTime` (`00b056a0`, confirmed) compares the end time with now, and on a changed, positive multiple of 60 it fires `MountHandler:TrialTime` with the remaining value (so the unit is probably seconds, matching the minute-based locale text). `EndTrial` (168/11) fires `MountHandler:EndTrial` with five values and resets the end time to the "none" value. `TrialFailed` (168/13) builds a red system message with hour and minute pieces (the `TimeUnitHour` and `TimeUnitMinute` strings), which looks like a "cannot try another yet" notice (likely). Promotional data sources `BaseClient.MemberUpgrade.PetTrial` and `...MountTrial` are named in the packet dispatcher (S0286, likely).
- **Upgrade.** `Mount.Upgrade(mountId, upgradeItemDefinitionId)` (`00bffbc0`, confirmed) sends `PacketMountUpgradeRequest` (168/14, `00bff390`, confirmed) and logs "Upgrading Mount mountId = %d, upgradeItemDefinitionId = %d". The reply `PacketMountUpgradeResponse` (168/15) makes `00b06100` show a green "UpgradeSuccessful" system message and then, unless the 1000-unit debounce is active, dismount, send 168/6 with the returned id and call `SetAutoMountId` ("Automount after upgrade: id %d"). So after an upgrade the client re-summons the ride at once.
- **Filter.** `Mount.EnableUpgradableMountsFilter(flag)` (`00bff440`, label unknown) stores a byte on the list data source (+0x2b1) so the rebuilt list keeps only records whose "IsUpgradable" flag is set.
- **Set automount.** `PacketSetAutomount` (168/9) just stores an id with `SetAutoMountId`. Whether it is a server-chosen ride to bring back at login or after a zone change is unknown.

### Data and formats

**Client resource files (read by the client, not sent).** From the shipped client baseline `Resources` folder (`lab/runs/client-baseline-*/client/Resources`).

| File | Contents | Source |
|---|---|---|
| `AnimationTypes.xml` | Animation type `Mounts` (id 18). A `MountTypes` table of 29 rows (`id`, `animSlotPrefix`, `description`; ids "must be unique and strictly increasing"). A `Mounts` branch of the animation tree with 22 to 48 slots per mount type, each named `<prefix>_<set>` and tagged `mountTypeId`. The set names include `loc_stand`, `loc_walk`, `loc_run`, `loc_backpedal`, turns, banks, swim variants, `loc_jump` and `loc_fall` (up to five variants), `loc_land`, glide deploy, fall, jump, land and retract, `loc_fidget_01` to `03`, `amb_loop_01`, `amb_oneshot_01`, `com_stand` and `com_h2h_attack_01` (45 distinct set names in all). Not every type has every set. | client resource |
| `ContentPacks.txt` | Row 25, "Rides/Mounts", flagged unlocked in all 13 environment columns. What a content pack gates is covered elsewhere; here it only shows rides are a content pack. | client resource |
| `VehicleItemClasses.txt` | 124 rows `VEHICLE_ID^ITEM_CLASS_ID^` (vehicle ids 2 to 69) loaded by `VehicleItemClassManager::LoadVehicleItemClasses` (`0104aea0`, confirmed). No evidence ties it to rides; it belongs with karts and other vehicles ([[Kart Racing]]). | client resource |
| `Models.txt` | Two rows name hoverboard mount models (`hoverboard_clockwork_mount.adr`, `hoverboard_precursor_mount.adr`). | client resource |
| `CodeStringMappings.txt` | Locale hooks `Mounts_ToolTip`, `Mounts_PS3`, `mountTooltip`, `PS3_STR_Mount`, `PS3_STR_Select_Mount`, `membership_mount_deluxe`. | client resource |

**Mount types (`MountTypes` rows).** The animation prefix is also the name used in the asset manifest.

| Id | Prefix | Description in the file | Id | Prefix | Description in the file |
|---|---|---|---|---|---|
| 1 | saddleground | Quadruped, saddled, walks on ground | 16 | chicken | Chicken |
| 2 | saddleair | Winged, saddled, flies in air | 17 | mech | Bipedal Robot |
| 3 | seated | Mount where you simply sit down (like a chair) | 18 | chopper | Chopper (Motorcycle) |
| 4 | surf | Like a surf board or skateboard | 19 | broom | Broom |
| 5 | trex | Tyrannosaurus Rex | 20 | pogostick | Pogostick |
| 6 | panthera | Panthera | 21 | reindeer | Reindeer |
| 7 | ball | Ball | 22 | bear | Bear |
| 8 | hydra | Hydra | 23 | hover | Combiner Ride Hoverer and Helicopter |
| 9 | tractor | Tractor | 24 | lawnchair | Lawnchair |
| 10 | bike | Bike | 25 | cybercycle | Combiner Ride Cybercycle |
| 11 | bat | Bat | 26 | flyingcarpet | Flying Carpet |
| 12 | wolf | Wolf | 27 | tank | Combiner Ride Tank |
| 13 | kangaroo | Kangaroo | 28 | jet | Combiner Ride Jet |
| 14 | flyingsaucer | Flying Saucer | 29 | combinermech01 | Combiner Ride First Combined Mech |
| 15 | octopus | Octopus | 0 | (none) | empty row |

The description for id 1 reads "Qudraped" in the file; the table above corrects the spelling. Ids 23, 25, 27, 28 and 29 are the five "Combiner" types, which matches the locale lines "Collect all 5 Mechanical Combiner Rides!" and "Combiner Rides". Pegasus, horse, tiger, dragon, unicorn and similar rides have no type of their own; presumably they use `saddleground` or `saddleair` (speculative). The mount type of each ride is not stored in any client table I found; it must come from the ride's actor definition or from the server (Open questions).

**Asset manifest (client assets, not sent).** About 3,190 manifest entries have "mount" in the name (excluding wall-mounted furniture): roughly 2,000 animation clips (`.gr2`), 860 icon textures, 157 audio files, 35 actor definitions (`.adr`, for example `hoverboard_surfboardshort_mount.adr`, `trex_air_mount.adr`, `drake_m_mount.adr`, `batmount_m_basic.adr`, `wolfmount_m_basic.adr`, `robgoblin_mechmount.adr`) and effect files. Icons come as `icon_item_mount_<type>_<variant>_64.dds` with a matching `_mask_` texture (a tint mask), plus category icons `icon_category_mounts_*` including "flying" and "flying grey" variants. Clip counts per prefix range from 9 (pegasus) to 75 (bike, cybercycle).

**The mount record on the wire** (reader `008e74c0`, the entry reader of `PacketMountList` and of the login body; widths and order read from the pseudocode and the label this pass; the label for the reader is accepted, field names are inferred from the data-source columns in `00cc3910`):

| Order | Width | Client field | Meaning |
|---|---|---|---|
| 1 | i32 | +0x00 | Mount id; the store key. Default -1. |
| 2 | i32 | +0x04 | Text id of the ride name (the data source shows "<mount name: string %d not found>" when the id is missing from the string table). |
| 3 | i32 | +0x08 | `ImageSetId`, the icon. |
| 4 | u64 | +0x20/+0x24 | Guid of the spawned ride actor; the "none" pair when not spawned (likely: drives `IsCurrentlySpawned`). |
| 5 | u8 | +0x28 | `MembersOnly`. |
| 6 | i32 | +0x0c | Tint value (the `Tint` column resolves it with a tint-alias lookup; likely). |
| 7 | i32 length, then bytes | +0x10 | A string, presumably the tint alias; unconfirmed. |
| 8 | u8 | +0x29 | `IsUpgradable`. |
| 9 | u8 | +0x2a | `IsUpgraded`. |

An underrun zeroes each field and sets the reader's error byte. `IsCurrentlyMounted` is computed from the guid and the rider's mount state, not sent (read from pseudocode, likely).

**UI data sources (client).** `BaseClient.MountList` columns, in index order: `Id`, `ImageSetId`, `Name`, `IsCurrentlySpawned`, `IsCurrentlyMounted`, `Tint`, `MembersOnly`, `IsUpgradable`, `IsUpgraded` (the column-name function `00cc3910`; the accepted label for `00cc4200` names only the registration). `BaseClient.CurrentMount` columns: `CurrentMountGuid`, `Name`, `ImageSetId` (function `00cc39c0`).

### Network

**Envelope.** All mount traffic is packet family `0xa8` (168). The body starts with a u16 family id followed by a u8 sub-opcode (reader `00b04c20`, read this pass); shard S0332 reached the same reading: the payload opens with a 16-bit family opcode and a one-byte sub-opcode, so a serializer that writes only one byte will not match. Senders serialise a small body and wrap it in a `PacketTunneledClientPacket` (`00b05ea0`, `00b05fb0`, `00bff5c0`) only when the client transport at client +0x280 exists. The dismount request goes through the generic sender `009bb570` (not traced). The datagram framing and byte order are unverified: no mount datagram has been decoded. Guids are two 32-bit words, read as one 8-byte value.

**Server to client (handled by `00b06100`).**

| Sub | Class (RTTI) | Body after the 3-byte header | Client action |
|---|---|---|---|
| 2 | PacketMountResponse | guid (rider), guid (mount), i32 seat, i32 result, i32, i32, f32 (reader `00b04c80`, NaN-checked) | See Spawning step 6. Seat bound-checked against the mount model; result 1 mounts, 0 undoes; `+0x2c` is stored on the rider (+0x17bc); the last two fields are passed on to `Mount`. |
| 4 | PacketDismountResponse | guid (rider), i32 (reader `00b04dc0`) | Stores the i32 at rider +0x17c0, runs `Dismount`. |
| 5 | PacketMountList | i32 count, then that many mount records (readers `00b05e00`, `00b05830`, `008e74c0`) | Replaces the store and rebuilds the UI lists. |
| 9 | PacketSetAutomount | i32 (reader `00b053d0`) | `SetAutoMountId(id)`. |
| 10 | PacketMountStartTrial | i32, i32 (reader `00b05440`) | Sets the trial end time; fires `MountHandler:StartTrial`. Which of the two values is the duration is not proven. |
| 11 | PacketMountEndTrial | guid, 6 x i32 (reader `00b054b0`) | Fires `MountHandler:EndTrial`; clears the trial end time. |
| 13 | PacketMountTrialFailed | 3 x i32 (reader `00b05520`) | Red system message with a time left in hours and minutes (likely). |
| 15 | PacketMountUpgradeResponse | i32 (reader `00b05590`) | Green "UpgradeSuccessful" message, then dismount, re-spawn and set automount of that id. |

The field widths come from the readers; the names for sub-opcode 2 are my reading of the log text and the code (the log prints the first guid as the rider, the second as the mount, and field 4 as "queue position"). Class bindings for 2 and 4 are `likely` in the accepted records because the readers carry no vtable of their own. Sub-opcodes 10, 11, 13 and 15 are classed as client-received because the handler logs their events and `SetAutoMountId` only stores (S0332).

**Client to server (senders).**

| Sub | Class (RTTI) | Body | Sender |
|---|---|---|---|
| 1 | PacketMountRequest | constructor stores three values (target guid as two words, an i32) at +0x10, +0x14, +0x18 and two zero bytes at +0x1c/+0x1d (`008b3730`, confirmed). Writer not located. | `PlayClient::SendMountRequest` (`00934970`, confirmed), `SendMountRequest` (`00b04290`, likely) from `0096de80` (guid, 0, 0) and from `008bb3a0` (guid, a handler value, flag 1) |
| 3 | PacketDismountRequest | no payload field initialised (`009aa410`, confirmed) | `SendDismountRequest` (`00b06cc0`), `Mount.Despawn`/`Dismount`/`Spawn`, minigame start, ability start, item-use paths |
| 6 | PacketMountSpawn | i32 mount id (constructor sets +0xc to -1; writer `00b05a80`) | `Mount.Spawn` (`00bff6f0`), `HandleMountPackets` after an upgrade, `SendPendingMountSpawn` (`00bdc330`) |
| 8 | PacketMountSpawnByItemDefinitionId | i32 item definition id, u8 flag (1) (writer `00b05ae0`) | `SpawnMountByItemDefinitionId` (`00b06db0`) |
| 14 | PacketMountUpgradeRequest | i32 mount id, i32 upgrade item definition id (writer `00bff500`) | `Mount.Upgrade` (`00bffbc0`) |

Sub-opcodes 0, 7 and 12 have no class in the client. The packet `PacketClientTrialMountUpsell` (opcode `0xc9`, 201, constructor `0089be40`, confirmed, no payload) is a separate message; the dispatcher builds and destroys it, so it is received (likely) and presumably tells the client to show a trial-ride upsell (speculative).

**Related traffic outside the family.**

| Message | What it carries for rides |
|---|---|
| SendSelfToClient (id 12) | The mount list at login (see above). |
| Base player update (`0092f460`) | Pending-mount values for a rider already seated when they appear; subtype not traced. |
| Item record lists (`00a1ec00`) | Routes mount items to 168/8. |
| `CommandPacketSetActiveVehicleGuid` (command 31, `00a10340`, `00a1e580`) | Two dwords naming an "active vehicle"; a link to rides is not shown ([[Kart Racing]]). |

**Lua `Mount` module (5 entries, decoded this pass):** `Spawn`, `Despawn`, `Dismount`, `EnableUpgradableMountsFilter`, `Upgrade`. Events the client raises for rides: `GameDock:SetMountButtonCallback` (with 1, 2 or 3), `MountHandler:StartTrial`, `MountHandler:TrialTime`, `MountHandler:EndTrial`, and the generic `NotificationHandler:SystemMessage`.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Persist the mount list.** Per character: the owned mounts, each with mount id, name text id, image set id, tint and tint alias, members-only flag, upgradable and upgraded flags (the record above). Keep the spawned mount's actor guid in session state and put it in the record's guid slot (R for session-only).
2. **Content the client lacks.** Which mount ids exist, their prices, availability channel, members-only rule, mount type (animation prefix), seat count, glide ability and speed. None is in the client. Derive them from our own design, using the wiki ride list (113 pages) as the roster and no emulator tables.
3. **Login.** Include the mount list in the self packet (message 12) in the position the reader expects, after the pet section. Send a 168/5 list whenever ownership or flags change (purchase, claim, upgrade, trial end).
4. **Spawn (168/6, 168/8).** Treat the ride id as a claim: check ownership (or trial state), the members-only rule, and the wiki restrictions (not in a minigame, dungeon or duel) (R). The client sends a dismount (168/3) before every spawn, so a spawn arrives with no ride active. Create the ride actor in the world with its actor definition (a model with a `Mountable` block), then make sure the client has the guid before it needs it: the automount only fires when the actor's guid matches the list entry for the automount id the client set when it sent the spawn (hypothesis from `0096de80`). Sending the new guid in a 168/5 list entry is the minimum the client's code requires; whether the original server did so or sent 168/2 directly is unknown.
5. **Mount request (168/1) and response (168/2).** On the request, validate that the target actor is the player's own ride (or a mountable NPC in reach) and the seat is free. Reply with 168/2: both guids, seat index below the model's seat count, a result of 1 to accept or 0 to refuse, and the remaining fields. Send it only after both actors exist for the client, or the client logs a failure and ignores it. A refusal (result 0) makes the local player dismount.
6. **Dismount (168/3).** Replace the rider's mount state, despawn the ride actor if it was summoned, and reply with 168/4 (rider guid and an i32). The client does not dismount until it gets the reply. `Despawn` and `Dismount` send the same packet; use the player's state to decide whether the ride actor is removed (R).
7. **Debounce and ordering.** The client already holds back repeats for about one second. The server should still handle duplicate and out-of-order requests: spawn, mount, dismount and spawn in quick succession, a dismount while not mounted, and abilities or minigame starts that arrive right after a dismount.
8. **Rider movement.** A rider's position is the ride's. How movement and speed are applied to a mounted player is not recovered from the client; the wiki gives up to 150 percent of normal speed, not stacking (wiki, R). See [[Movement and Pathfinding]]. The server must keep the rider and ride in sync for other players, including late joiners: send the pending-mount values in the player-update path so a rider who was already seated appears mounted.
9. **Weapons, emotes and tool belt.** Block emotes, weapon use and tool-belt use while mounted (wiki, R), and end the ride on minigame entry and ability use as the client already requests with 168/3 ([[Combat and Abilities]], [[Minigame Framework]]).
10. **Trials.** For a trial ride, send 168/10 with the length (two i32; confirm which is the duration), let the client count down, and send 168/11 when it ends; send 168/13 when a new trial is refused (time left in hours and minutes, R for the numbers). Include the `MountTrial` promotional upsell where the dispatcher expects it (S0286). Trial length and cooldown are not in the client (R).
11. **Upgrades.** Handle 168/14 (mount id, upgrade item definition id): check ownership of both, consume the upgrade item, set `IsUpgraded`, send a 168/5 list and then 168/15 with the mount id. The client then dismounts and re-summons that id. What the upgrade does to the ride is unknown (R).
12. **Gliding.** The Glide Training quests, the Training Saddle of Gliding item and the glide flag per ride are server and content side; the client list has no glide column (Open questions). Content per [[Quests and Objectives]] and the Seaside NPC page.
13. **Store and rewards.** Rides are sold in the Rides category and given by chests, cards and events; the purchase confirmation should reach the item-record path (168/8) so the new ride mounts at once ([[Coin Store and Purchases]], [[Items and Inventory]]). Prices and discounts are wiki and video data, with conflicts (R).
14. **Errors.** Every refusal should end in a visible client effect (a result 0, a system message, the dock button reset), never silence.

### Video and wiki evidence

**Video frames reviewed:** frames showing the Rides menu, Rides shop, ride advertisements, riders and the Ride Trainer. Hundreds of frames show a character on a ride with player chat and names; only ride type and context are recorded, and no ride names chosen by players.

- **Rides menu (video).** The in-game Rides menu is a paged list headed "Rides" with a "Main Menu" button and a "1/2", "2/2" page counter. Page 2 of 2 lists Bicycle, Hydra Ride, Tractor, Beach Ball, Fire Pegasus, Bat, Wolf, Kangaroo, Robgoblin Mech, Glacierator 5000 and The Fire Hawk (`yLjpOTxwUj8/00052`). Another view lists Octopus, Lawless Gleam Chopper, Valiant Gleam Cycle and The Hero Copter, with a gold badge on the first three entries (`lPWru_CDw5I/00184`); a gold winged badge also sits on an Octopus entry (`yWuHe_jYyc4/00172`). What the badge means is not read (it could be membership, glide training or an upgrade).
- **Rides shop (video).** The Station Cash Marketplace has a Rides category with sub-lists Balls, Tigers, Hoverboards, Unicorns, Horses, Dragons, T-Rex and Pegasus (`3udDqx4LYVA/00690`). A Rides shop frame lists Fairweather Hoverboard (Item Of The Day at **275 Station Cash with 550 struck through**), Eight Ball, Futuristic Bicycle, Precursor Hoverboard, Gloamstone Hoverboard, Skeletal T-Rex, Pogo Stick, Holiday Tree Board and Polar Bear (`fjZluIfbUIo/00100`). The T-Rex list shows six colour variants at **450 regular, 360 member (a 20 percent discount)** (`3udDqx4LYVA/00690`). An advert reads "RIDES IN THE SC MARKETPLACE! Mount up and get around in style! From 450 - 550 Each" (`e4XXSnnive0/00236`). Another reads "Upgrade Now for Power Mount" (`4kAFL0X0KkA/00642`; client build string v1.828 in the frame).
- **Riding (video).** Players appear on winged mounts gliding over scenery, on a skeletal dinosaur, a polar bear, a motorcycle-type mount and board-type rides; a yellow ringed launch pad appears in the Glide Training frames (`fjZluIfbUIo/00118`). A crowd of players on horse-type mounts fills the Pet Party plaza (`_rHV7eHti_U`).
- **Ride Trainer (video, wiki).** The Ride Trainer NPC Troy Ridewrangler stands in Seaside; frames show the quest panel text "Bounce through the fiery hoop above the launch pad outside of Sanctuary while on your untrained glide-trainable ride." (`fjZluIfbUIo/00052`).
- **Client locale on rides.** The Glide Training series has five steps, named by town: Sanctuary, Merry Vale, Snowhill, Lakeshore, Seaside, each "bounce off a launch pad and through the fiery hoop" and each rewarded with a "Proof of ... step" item, collected by the trainer after all five. A Glide Training Certification allows buying more Training Saddles in the Marketplace; the saddle "will PERMANENTLY grant the Gliding skill to this ride." Glide-trainable rides are bat, dragon, feathered drake, pegasus and phoenix. There are help lines "How can I change my ride?", "How do I use my ride?" and "How can I buy a ride?". Locale lines also mention a "Premium Ride Trainer" and "Deluxe Ride" with no recovered detail.
- **Wiki roster.** The Rides category has 113 pages. By infobox type: 45 mechanical, 39 non-winged, 12 exotic, 11 winged and 6 with no clean type. Page categories include 21 Unreleased, 9 Unobtainable, 21 Seasonal, 9 Mystery Chest, 4 Trading Card Game virtual rewards, 6 Combiner Rides and 13 Hoverboards. By family: 13 hoverboards, 10 T-Rexes, 7 tigers, 7 horses, 6 dragons, 5 pegasi, 5 surfboards, 4 bicycles, 4 turkeys.
- **Wiki prices.** 62 pages give a Station Cash regular price, from 149 to 999. The commonest are 549 (14 pages), 550 (13) and 499 (6). Of the 52 pages with two prices, 51 show the member price at 90 percent of regular (10 percent off). Rides are sold in the Marketplace; some are limited to events (Thanksgiving, Snow Days, Super Spooktacular, Mystery Chests) or come from cards and the Mysterious Merchant.
- **Wiki rules.** Rides give up to 150 percent of normal speed, do not stack with other boosts, cannot be used in a minigame, dungeon or duel, disable emotes, boomboxes, illusions, weapon display, weapon effects and tool-belt use. The ride picker was a small side-scrolling strip until January 10, 2013, then a full-screen list like the pet selector. Membership became required for rides on July 1, 2010, but owned rides stay usable after membership lapses. The Polar Bear was free from the Snow Days gift boxes.
- **Wiki gliding.** The Glide Training quest series offers five quests, each doable one day after the previous; the Training Saddle of Gliding is used once on one eligible ride. Phoenix is a Treasure War virtual-reward ride (a repeat claim pays 2,500 coins, client locale).

### Open questions and contradictions

- **No captured mount traffic.** All layouts come from the client's readers and writers. The datagram framing, byte order, the writer for 168/1 and the writer for 168/3 (header only) are unverified.
- **Meaning of 168/2 fields.** The log calls field 4 "queue position" but the code tests it for 1 and 0 as an accept or refuse code. The third field is the seat index (the code bounds-checks it against the model's seat count). The fifth is stored on the rider and the fourth and sixth are passed on to `Mount`; their meaning is unresolved. The call's argument order is read from pseudocode in which Ghidra shows the stack arguments out of order, so treat the field-to-parameter mapping as likely, not confirmed.
- **Seats and queueing.** The Mountable block allows up to 99 occupants and a seat list, the response log says "queue position", and the locale has a "Tandem Mount Test" string. No shipped multi-seat ride is identified, and nothing in the client says how rider two joins.
- **Spawn ordering.** I inferred that the server creates the ride actor and updates the list entry's guid before the client sends 168/1. Nothing proves the original order, or that the client could also receive 168/2 unprompted.
- **`Despawn` versus `Dismount`.** Both Lua calls send an identical 168/3. The server must have used its own state to tell them apart.
- **`SetAutomount` (168/9).** Nothing in the client says when the server sends it.
- **Trial packet fields.** The two values in 168/10, the six in 168/11 and the three in 168/13 are read but not named. The unit of the trial time is assumed to be seconds because of the multiple-of-60 test.
- **Upgrade.** `IsUpgradable` and `IsUpgraded` exist, with a request that names an upgrade item. Whether "Power Mount" in the advert is this upgrade, and whether upgrading relates to the five Combiner rides (five Combiner mount types exist, and the locale says to "Collect all 5 Mechanical Combiner Rides!"), is unknown. The combiner link is an inference from names only.
- **Glide.** The client's mount list has no glide column, and no client field marks a ride glide-trained. The five glide-trainable types come from locale text and the wiki. The assets include "flying" category icons, which hint that the client groups winged rides, but no code was traced.
- **Speed.** The 150 percent figure is wiki only. No client constant for ride speed, and the movement controller, were not traced to a mount term.
- **Mount type per ride.** The 29 mount types are animation sets. The wiki has 113 rides and no table maps a ride to its type; horses, pegasi, dragons, tigers and unicorns share or reuse sets. It is not in the client files read.
- **Inventory versus mount list.** The wiki says rides are not inventory objects and cannot be traded, while the client has an item-definition spawn (168/8) and a mount-item category in `00a1ec00`. Which item category is the mount bucket is not proven.
- **Prices.** Wiki member price is 10 percent off for 51 of 52 pages, but the video shows a 20 percent member discount on T-Rex rides, and an advert gives 450 to 550 each against a wiki range of 149 to 999. Prices changed over time; no price is in the client.
- **Restrictions.** Wiki says rides cannot be used in dungeons or duels. The client was seen to dismount only at minigame start and at ability start; the dungeon and duel rule is unproven in code.
- **Mounted combat.** Locale has a "Mounted Combat" label (two ids) and "Skullz bike rams the player while mounted"; they look like ability or effect names. Nothing was traced.
- **Gold badge in the Rides menu.** Unread (see Video).
- **Glide Training length.** Client locale has both a five-step series (one town per step) and older text for a "3-step program" with three rings in one area; which version shipped last is unknown.
- **Vehicles.** `CommandPacketSetActiveVehicleGuid`, `VehicleItemClasses.txt` and the subsystem tag `net.packet.vehicle` on the family constructor (S0332 notes the inconsistent tag with `net.mount`) suggest an overlap with the vehicle system; any link to rides is untraced.
- **Pending-mount subtype.** Which player-update subtype carries the five pending-mount values, and what the five are, is not traced.
- **Label gaps.** The accepted labels for `00bff6f0`, `00bffa60`, `00bff440`, `00973050`, `008bb3a0`, `0095a710` and the unclassed readers `00b04e70`, `00b04ee0`, `00b05000`, `00b051b0`, `00b05220` are unnamed or `unknown`; this page used their behaviour, not their names. None of the mount addresses cited here appears in the known-wrong labels list.

### Key functions

| Address | Name | Confidence |
|---|---|---|
| `00b06100` | HandleMountPackets | confirmed |
| `0089f9c0` | MountBasePacket::MountBasePacket | confirmed |
| `008b3730` | PacketMountRequest::PacketMountRequest | confirmed |
| `00934970` | PlayClient::SendMountRequest | confirmed |
| `00b04290` | SendMountRequest | likely |
| `009aa410` | PacketDismountRequest::PacketDismountRequest | confirmed |
| `00b06cc0` | SendDismountRequest | confirmed |
| `00b06d70` | HandleDismountInput | likely |
| `00b038c0` | PacketMountSpawn constructor (label unknown) | unknown |
| `00bdc330` | SendPendingMountSpawn | likely |
| `00b06db0` | SpawnMountByItemDefinitionId | confirmed |
| `00bffbc0` | Mount::Upgrade | confirmed |
| `00bff390` | PacketMountUpgradeRequest::Ctor | confirmed |
| `00bff480` | RegisterMountBindings | confirmed |
| `00bff6f0`, `00bff8e0`, `00bffa60`, `00bff440` | Mount.Spawn, Despawn, Dismount, EnableUpgradableMountsFilter (names from the decoded table; labels unknown or likely) | unknown |
| `00b03c90` | SetAutoMountId | confirmed |
| `00b047b0`, `00b047f0` | ListEntryMatchesIdAndGuid, ListLookupIdByGuid | likely |
| `00b056a0` | UpdateMountTrialTime | confirmed |
| `00b04c20`, `00b04c80`, `00b04dc0` | Header reader, 168/2 body reader, 168/4 body reader | likely |
| `00b05e00`, `00b05830`, `008e74c0` | Mount list body reader, list loop, mount record reader | likely, unknown, accepted record |
| `00b05600` | Copy parsed list into the store (label unknown) | unknown |
| `00b03cd0`, `00b050d0`, `00b04b70` | Store constructor, insert, remove | confirmed RTTI, likely members |
| `00972d30` | Mount | confirmed |
| `00973050` | Dismount (label unknown) | unknown |
| `009710b0` | RemoveRider | confirmed |
| `00971250`, `009722d0` | UpdateSeatAttachment, TransitionIndexedActorState | likely |
| `00961b70`, `00961c40` | GetSeatTransform, GetSeatTransformSecondary | likely |
| `0095ad80` | SetPendingMountInfo | confirmed |
| `00973200` | ProxiedCharacter::Process | confirmed |
| `0096de80` | UpdateStateAndMountLog (ProxiedCharacter slot 1) | see S0258 |
| `008bb3a0`, `008bb440` | InitiateMountControllerHandler (mount, reset) | unknown, confirmed |
| `00a2e540` | One-second request debounce (label unknown) | unknown |
| `00a35150`, `00a35600` | Ability start gate and dismount-before-ability | likely |
| `009bd650`, `009beb70` | Dismount on minigame start | confirmed |
| `00944fe0` | SendSelfToClient body reader (mount list inside) | speculative |
| `00cc4200`, `00cc42c0`, `00cc3c70` | Mount list and current-mount data sources, list rebuild | confirmed, confirmed, likely |
| `007a7e40`, `007a2cd0`, `007aa2d0` | MountTypes loader, MountTypes writer, animation parameter loader | confirmed |
| `007e4000` | MountableDefinitionRead | confirmed |
| `00bdc060`, `00bdc3c0` | UseItemSelectedInUi, ClearPreviewAndFlushMount | likely, speculative |
<!-- NARRATIVE END -->

## Mapped functions (generated)

78 accepted functions: 28 confirmed, 32 likely, 1 speculative, 17 unknown. Network-linked (`wire`): 39.

Subsystems: `net.mount` (60), `game.mount` (18)

### `game.mount`

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a2cd0` | BuildMountTypesXmlDoc | confirmed | __fastcall: ECX = object; its MountType list head is at this+0x4968 (next at record+0xc; record fields: +0x10 id, +0x18 animSlotPrefix string, +0x28 descriptio… |
| `007e4000` | MountableDefinitionRead | confirmed | Thiscall (ECX = owner, one stack arg = reader). Tag 4 stores a dword in owner+0x48c, the minimum occupancy. Tag 1 appends a seat element through 007e3ca0 (0x74… |
| `0095ad80` | SetPendingMountInfo | confirmed | Logs the rider name and GUID plus mount GUID to ClientMountLog.txt, then stores five incoming values at offsets 0x17a8 through 0x17b8. |
| `00b03c90` | SetAutoMountId | confirmed | Logs the string SetAutoMountId: mount %d with param_2 into the log file ClientMountLog.txt through FUN_00704db0, then stores param_2 into this+0x64. |
| `00b056a0` | UpdateMountTrialTime | confirmed | Compares stored timer words at +0x58/+0x5c with current time; on a changed positive multiple of 0x3c, emits MountHandler:TrialTime and stores elapsed time at +… |
| `00b04240` | StoreTimerAt68 | likely | Calls the 64-bit millisecond timer FUN_007027f0 with a stack output buffer and copies the two returned dwords into this+0x68 and this+0x6c. |
| `00b042e0` | SetSubObjectByte2B1AndRefresh | likely | Stores the byte param_2 at ([this+0x70])+0x2b1, then calls FUN_00cc3c70 and FUN_00cc39a0 in that order with ECX set to [this+0x70] (the second call re-loads th… |
| `00b04310` | ClearTwoSubObjectFlags | likely | Calls FUN_00cc39a0 with ECX = [this+0x70], loads ECX = [this+0x74] and tail-jumps (e9, verified in the raw bytes) to FUN_00cc3a00, which clears a flag byte and… |
| `00b047b0` | ListEntryMatchesIdAndGuid | likely | Walks the singly-linked list whose head is at this+0xc (next pointer at entry+0x30) and returns 1 as soon as an entry has the guid pair (*param_2 at entry+0x20… |
| `00b047f0` | ListLookupIdByGuid | likely | Walks the same singly-linked list (head at this+0xc, link at entry+0x30) and returns the id stored at entry+0 of the first entry whose guid pair matches (*para… |
| `00b048a0` | NodeDtorCallWrapper | likely | Wrapper that sets up an SEH frame with the security cookie and then calls the record destructor FUN_00b04830 with its own this still in ECX (bytes 51 push ecx,… |
| `00b04980` | NodeInitWithIStringAndGuid | likely | Initialises the 0x40-byte mount list record: writes -1 at +0, zeroes +4/+8/+0xc, installs SoeUtil::IString<char>::vftable at +0x10 with the shared empty buffer… |
| `00b04a10` | NodeCtorWithKeyAt38 | likely | Constructor of one mount-list record: calls the field initialiser FUN_00b04980, then stores the dword at *param_2 into the record at +0x38, which is the key fi… |
| `00b04a90` | NodeScalarDeletingDtor | likely | Scalar deleting destructor of the mount-list record: calls the destructor body FUN_00b04830 and, when the low bit of param_2 is set, calls _free(object) and re… |
| `00b06d70` | HandleDismountInput | likely | Looks up local action 0x27, invokes a virtual action handler, tests the resulting condition, and sends a dismount request when the test fails. |
| `00bdc330` | SendPendingMountSpawn 🌐 | likely | When the dword at this+4 differs from -1 it constructs PacketMountSpawn (00b038c0: mount family 0xa8 opcode 168, sub-opcode 6) with that dword as the packet pa… |
| `0095a710` | — | unknown | Queries the current context mount list, resolves each entry's GUID, compares four float coordinates with the input, tracks the lowest squared distance, and ret… |
| `00b04830` | — | unknown | Destructor body for a record holding a SoeUtil::IString<char> at +0x10: writes the IString vtable at +0x10 and, when the count field at +0x1c is greater than 0… |

### `net.mount`

**MountBasePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03710` | MountBasePacket::MountBasePacket 🌐 | confirmed | Family constructor without a sub-opcode: installs BasePacket::vftable, writes the mount family opcode 0xa8 (168) into the packet id slot [1] at +4, installs Mo… |

**PacketDismountRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa410` | PacketDismountRequest::PacketDismountRequest 🌐 | confirmed | Constructor: calls the MountBasePacket base constructor FUN_0089f9c0(3) (which writes the mount family opcode and sub-opcode 3) and installs PacketDismountRequ… |
| `009ac9d0` | DeletingDtor_009ac9d0 | likely | Scalar deleting destructor for a BasePacket-derived mount packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object thr… |

**PacketDismountResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b04710` | PacketDismountResponse::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |
| `00b046a0` | PacketDismountResponse::PacketDismountResponse 🌐 | likely | Constructor: calls FUN_0089f9c0(4) (family opcode 0xa8, sub-opcode 4), installs PacketDismountResponse::vftable, stores the invalid-guid pair DAT_01be9138/DAT_… |

**PacketMountEndTrial** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b040a0` | PacketMountEndTrial::PacketMountEndTrial 🌐 | confirmed | Constructor: calls FUN_0089f9c0(0xb) (family opcode 0xa8, sub-opcode 11), installs PacketMountEndTrial::vftable, stores the invalid-guid pair DAT_01be9138/DAT_… |
| `00b04120` | PacketMountEndTrial::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PacketMountList** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b05b80` | PacketMountList::Constructor 🌐 | confirmed | Constructs PacketMountList with mount-family sub-opcode 5 and initializes its packet-info list. |
| `00b05bf0` | PacketMountList::Destructor | likely | Sets PacketMountList vtable, destroys all packet-info list entries, restores list/base vtables, and returns. |
| `00b05c90` | PacketMountList::ScalarDeletingDtor | likely | Calls PacketMountList cleanup and frees the packet allocation when delete flag bit 0 is set. |

**PacketMountResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b045b0` | PacketMountResponse::PacketMountResponse 🌐 | confirmed | Constructor: calls FUN_0089f9c0(2) (family opcode 0xa8, sub-opcode 2), installs PacketMountResponse::vftable, stores the invalid-guid pair DAT_01be9138/DAT_01b… |
| `00b04640` | PacketMountResponse::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PacketMountSpawn** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03f80` | PacketMountSpawn::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |
| `00b038c0` | — 🌐 | unknown | Constructor: calls FUN_0089f9c0(6) (family opcode 0xa8, sub-opcode 6), installs PacketMountSpawn::vftable and initialises the payload dword at +0xc to 0xffffff… |

**PacketMountSpawnByItemDefinitionId** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03970` | PacketMountSpawnByItemDefinitionId::PacketMountSpawnByItemD… 🌐 | confirmed | Constructor: calls FUN_0089f9c0(8) (family opcode 0xa8, sub-opcode 8), installs PacketMountSpawnByItemDefinitionId::vftable, zeroes the payload dword at +0xc (… |
| `00b03fe0` | PacketMountSpawnByItemDefinitionId::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PacketMountStartTrial** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03a20` | PacketMountStartTrial::PacketMountStartTrial 🌐 | confirmed | Constructor: calls FUN_0089f9c0(10) (family opcode 0xa8, sub-opcode 10) and installs PacketMountStartTrial::vftable; no payload field is initialised. |
| `00b04040` | PacketMountStartTrial::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PacketMountTrialFailed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03b20` | PacketMountTrialFailed::PacketMountTrialFailed 🌐 | confirmed | Constructor: calls FUN_0089f9c0(0xd) (family opcode 0xa8, sub-opcode 13) and installs PacketMountTrialFailed::vftable; no payload field is initialised. |
| `00b04180` | PacketMountTrialFailed::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PacketMountUpgradeResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03bd0` | PacketMountUpgradeResponse::PacketMountUpgradeResponse 🌐 | confirmed | Constructor: calls FUN_0089f9c0(0xf) (family opcode 0xa8, sub-opcode 15), installs PacketMountUpgradeResponse::vftable and initialises the payload dword at +0x… |
| `00b041e0` | PacketMountUpgradeResponse::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PacketSetAutomount** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b03810` | PacketSetAutomount::PacketSetAutomount 🌐 | confirmed | Constructor: calls the MountBasePacket family constructor FUN_0089f9c0(9) (which writes the family opcode 0xa8 and sub-opcode 9) and installs PacketSetAutomoun… |
| `00b03f20` | PacketSetAutomount::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns… |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00934970` | PlayClient::SendMountRequest 🌐 | confirmed | Opens ClientMountLog.txt, constructs PacketMountRequest, and sends it when the gateway pointer at this+0x280 is non-null. |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b06100` | HandleMountPackets 🌐 | confirmed | Reads the mount-family header and dispatches sub-opcodes 2,4,5,9,10,11,13,15; parses responses/lists and updates mount, trial, notification and upgrade client … |
| `00b06cc0` | SendDismountRequest 🌐 | confirmed | Logs SendDismountRequest, captures two timer words into the packet, constructs PacketDismountRequest, and serializes/sends it when the gateway is available. |
| `00b06db0` | SpawnMountByItemDefinitionId 🌐 | confirmed | Logs the item definition id, constructs a mount-spawn-by-item packet, writes the id and flag, and serializes/sends it when the gateway is available. |
| `009aa470` | PacketDtor_009aa470 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from seven MSVC unwind funclets plus the dismou… |
| `00b03770` | PacketDtor_00b03770 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns. No member is destroyed and nothing is freed (th… |
| `00b037c0` | PacketDtor_00b037c0 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b03870` | PacketDtor_00b03870 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b03920` | PacketDtor_00b03920 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b03a80` | PacketDtor_00b03a80 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b03ad0` | PacketDtor_00b03ad0 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b03b80` | PacketDtor_00b03b80 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b03c30` | PacketDtor_00b03c30 | likely | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b04290` | SendMountRequest 🌐 | likely | Logs SendMountRequest: target %I64u with the two dwords of the target guid to ClientMountLog.txt through FUN_00704db0, then forwards the four arguments (param_… |
| `00b04c20` | ReadU16AndU8Header 🌐 | likely | Shared header reader used by the mount packet bodies. Reads the 16-bit value at the reader cursor (+8) and stores it into this+4, then reads the 8-bit value at… |
| `00b04c80` | ReadTwoGuidsFourIntsFloat 🌐 | likely | Mount packet body reader. Calls FUN_00b04c20 for the header, then reads an 8-byte value into +0x10/+0x14, a second 8-byte value into +0x18/+0x1c, four 32-bit v… |
| `00b04dc0` | ReadGuidAndInt 🌐 | likely | Mount packet body reader: calls FUN_00b04c20 for the header, reads an 8-byte value into +0x10/+0x14 and one 32-bit value into +0x18, with the usual 0/error-byt… |
| `00b04e70` | ReadTwoInts 🌐 | likely | Mount packet body reader: calls FUN_00b04c20 for the header, then reads two 32-bit values into +0xc and +0x10, storing 0 and raising the reader's error byte at… |
| `00b04ee0` | ReadGuidAndSixInts 🌐 | likely | Mount packet body reader: calls FUN_00b04c20 for the header, reads an 8-byte value into +0x10/+0x14 and six 32-bit values into +0x18, +0x1c, +0x20, +0x24, +0x2… |
| `00b05000` | ReadThreeInts 🌐 | likely | Mount packet body reader: calls FUN_00b04c20 for the header, then reads three 32-bit values into +0xc, +0x10 and +0x14, with 0/error-byte/cursor-clamp fallback… |
| `00b05e00` | BoundedListBodyReader_00b05e00 🌐 | likely | Reads the common packet header, decodes a list into the packet object, and returns whether reader status and bounds are valid. |
| `00b039d0` | PacketDtor_00b039d0 | speculative | Destructor body of a MountBasePacket-family packet class: writes BasePacket::vftable into *param_1 and returns; no member destruction and no free. |
| `00b051b0` | — 🌐 | unknown | Mount packet body reader: calls FUN_00b04c20 for the header, then reads one 32-bit value into the target object's +0xc, storing 0 and raising the error byte at… |
| `00b05220` | — 🌐 | unknown | Second copy of the single-dword mount body reader: calls FUN_00b04c20 for the header and writes one 32-bit value into the target object's +0xc, with the 0/erro… |
| `00b052f0` | — 🌐 | unknown | Checks reader/object pointers, calls the two-GUID/four-integer/float body reader, and returns success only when the read flags and bounds permit it. |
| `00b05360` | — 🌐 | unknown | Checks reader/object pointers, delegates GUID-plus-integer decoding, then applies the shared read-status and bounds condition. |
| `00b053d0` | — 🌐 | unknown | Checks reader/object pointers, delegates one-integer decoding into the target, then applies the shared read-status and bounds condition. |
| `00b05440` | — 🌐 | unknown | Checks reader/object pointers, delegates two-integer decoding, then applies the shared read-status and bounds condition. |
| `00b054b0` | — 🌐 | unknown | Checks reader/object pointers, delegates GUID-plus-six-integer decoding, then applies the shared read-status and bounds condition. |
| `00b05520` | — 🌐 | unknown | Checks reader/object pointers, delegates three-integer decoding, then applies the shared read-status and bounds condition. |
| `00b05590` | — 🌐 | unknown | Checks reader/object pointers, delegates a second one-integer body reader, then applies the shared read-status and bounds condition. |
| `00b05830` | — 🌐 | unknown | Drains the source list, advances a bounded counter/state, then invokes two operations once per remaining count unless its stop flag is set. |
| `00b05a80` | — 🌐 | unknown | Serializes a 32-bit list length/value into a writer buffer, bounded by remaining writer capacity, and advances the cursor. |
| `00b05ae0` | — 🌐 | unknown | Serializes a 32-bit list value and a one-byte field into the writer buffer, bounded by remaining capacity, advancing after each write. |
| `00b05ea0` | — 🌐 | unknown | Builds a tunneled-client-packet envelope, serializes a selected payload with its length/metadata, and finalizes the BasePacket wrapper. |
| `00b05fb0` | — 🌐 | unknown | Builds a tunneled-client-packet envelope, serializes a selected payload plus one byte, and finalizes the BasePacket wrapper. |

## Source shards

[[S0173|S0173]] (1), [[S0185|S0185]] (1), [[S0247 00934740-00939200|S0247]] (1), [[S0254 00954ec0-0095a710|S0254]] (1), [[S0255 0095a840-00960520|S0255]] (1), [[S0269 009a6cf0-009aceb0|S0269]] (3), [[S0332 00b006d0-00b05260|S0332]] (48), [[S0333 00b052f0-00b0a290|S0333]] (21), [[S0374 00bd9880-00be0090|S0374]] (1)
