---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Combat and Abilities

Part of [[Build Guides]] › Gameplay. Related: [[Character and Customization]], [[Items and Inventory]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable in a shard or in this pass; *pseudocode* = a field-level reading of the Ghidra export done in this pass (inference, not a captured fact); *client resource* = a loose text or XML file shipped beside the executable (`Resources/`); *client Lua* = the client's own UI script, decompiled in the lab; *client locale* = the shipped text table; *video* = a reviewed frame record (48 batch files, 5,760 records); *wiki* = the fan wiki; *vault* = a project research note, used here only for facts it derived from the client or from our own native runs. Precedence is client, then video, then wiki. *(R)* marks reconstructed server policy that is ours, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. The older vault pieces 046 to 058 lean on community serializers and on a decompiled server enum; this guide takes only their client-derived facts and drops everything else, and it uses no code, tables or packet class names from the community emulators. OSFR-held data (packet captures and server JSON) is not used. No player names or chat are recorded; enemy and NPC names are. Related: [[Character and Customization]], [[Items and Inventory]], [[Quests and Objectives]], [[Minigame Framework]], [[Matchmaking and Lobby]], [[Rewards and Collections]], [[Ship Combat (Pirates)]], [[Pets]], [[Gateway and Transport]], [[Command System]], [[Scaleform and Flash UI]], [[Scripting (Lua)]], [[Animation]], [[Jobs]].

### Overview

Combat is the part of the game where a player fights creatures with a job's weapon abilities. It happens in battle instances (dungeons and "wandering battle" encounters), in the open world against creatures that attack on sight, in boss fights and in player-versus-player duels. The server decides every result; the client only asks, animates, and shows numbers.

- **Combat jobs.** Six jobs fight: Archer, Brawler, Medic, Ninja, Warrior and Wizard (wiki, Combat page). The client's wield-type table has the same six as job wield types 9 (Ninja), 10 (Wizard), 11 (Archer), 12 (Warrior), 13 (Brawler) and 14 (Medic), all playing swing group 1099 (client resource `WieldTypeAnimationMappings.txt`). Adventurer is a non-combat job; its one combat-related text is a monster-scare trick for players outside a combat job (client locale 1631390737; see [[Adventurer]]). Combat jobs run to level 20 (wiki). They level by playing battles (client tip 4024725827; wiki). Battle difficulty is Tier 1 to 5 with level gates 1, 3, 5, 9 and 13 (wiki; the Sampson panel in video says "Requires a level 9 combat job").
- **Ability bar and keys.** The fight key is `1` and the "super attack" is `2` (wiki; client tutorial text "Click your Shuriken Toss special ability or press the 2 key", locale 2148594758; video "Hit the [2] key to use your super!"). The executable ships a default binding table (24-byte records at `0x1b41988`): `ActionBarSlot1` to `ActionBarSlot9` are action ids 19 to 27 on keys `1` to `9`, `TabTargetSelect` is id 39 on `Tab`, `TriggerClosestInteraction` is id 45 on `X` (client, via the vault note [[Native Combat Controls]]). Abilities come from the weapon in hand: "Abilities are tied to the weapon you're using. Equip a different weapon to use different abilities." (client locale 819381112). Brawler, Archer and Ninja also have four level-gated passive traits at levels 5, 10, 15 and 20; Warrior, Wizard and Medic have four level-gated abilities with five ranks each; Stars raise ranks (wiki, client locale; table below).
- **Targeting.** `Tab` picks the nearest hostile creature in front of the player that is in line of sight; clicking a creature selects it too. A red target bar shows the selected enemy's name and health (video); boss fights show a boss health bar at the top of the screen (video). The client ships combat cursors (`cursor_interaction_combat`, `cursor_combat_target_valid`, `cursor_combat_target_invalid`; client resource `Cursors.txt`).
- **Health and energy.** Two bars. Energy is called mana inside the client code and "energy" in every player-facing string (client locale 285 "You don't have enough energy for that ability!"). Video shows an Archer at job level 14 with 3483/3483 health and 100/100 energy, and a Medic at 10 with 14/110 energy. Health and energy refill stats exist (stat ids 4 and 6, client resource). Potions, food and combat power-ups restore them (wiki).
- **Damage numbers.** Floating numbers rise from the target: red negatives for damage (video values from -16 to -4981, with `!!` on eight of 51 numbers, for example `-4166!!`), a green `+371` for healing (video). The client formats a negative amount as `%d`, a negative critical-style amount as `%d!!` and a gain as `+%d` (client string constants read in `00ae2350`). A miss shows the literal text `MISS!` (client string, `00a2b360`).
- **Statuses.** Players and creatures can be asleep, silenced, stunned, rooted, knocked back, confused, poisoned, afraid, frozen and more (wiki lists Sleep, Silence, Stun, Root, Knockback, Confuse, Poison; the client has a 13-name list, see Data). A blocked cast prints a reason such as "You can't use abilities when you are stunned!" (client locale 284; seen in many video frames).
- **Knockout.** A player whose health runs out is knocked out. A panel reads "You Got Knocked Out! Recover in N seconds" with a countdown (client locale 410188 and 408280; video). After about ten seconds the player recovers with brief invulnerability and full health and energy (wiki). Battle instances limit knockouts: 10 in wandering battles and 15 in dungeons, and one knocked-out group member uses up the whole group's allowance (wiki); the client has "@1 Knockouts Remaining" (locale 419881). In the open world the player can instead pay coins to revive on the spot or revive at a safe location (client Lua `RespawnWindow`; coin price from the wiki via the vault, see Open questions).
- **Bosses, results and the prize wheel.** A boss is marked by a shaft of light and a boss bar (wiki, video). Finishing the main objective opens an exit door; the score screen shows score rows and rewards, and a won battle ends in a spinnable prize wheel whose slots mix item prizes with coins and stars (wiki; client Lua `ScoreScreen:PopulateLootWheel`). The same wheel appears after minigame wins (video: "YOU WIN!" with a wheel of coins and items, 10 frames mention it by name).
- **Duels.** Players can challenge each other. The client has a duel invitation and response in the encounter family and video shows duel fights with large numbers and the energy warning (client; video).
- **What our server already does.** The Rust server runs wolves, the Gloam fights, Mac and the opening cave through the real client, with a 3 m player melee reach, and the real client shows swings, red damage numbers, health, goal progress, removal of the dead enemy and the results screen. That acceptance is for a small reconstructed rule set (one basic attack, flat damage, flat health), not for the original ability, damage or status system. See Evidence.

### How the client implements it

#### Main classes

| Class | Role | Evidence |
|---|---|---|
| `CombatProcessor` (vtable `01813864`, slot 0 `00a2c370`) | Receive switch for the combat family (32): handlers for sub-opcodes 4, 5, 6, 7 and 9; init `00a2a120`; face-the-target helper `00a2a160`; Tab-target selector `00a2ac20`; swing-timer pair `00a2a6c0` (stamp) and `00a2a290` (still in the future?). Also owns the two client senders `00a2c7d0` and `00a2c880`. | dispatcher confirmed; handlers read in S0292 |
| `AbilityProcessor` (vtable `01813ae0`, ctor `00a324e0`, receive switch `00a35cc0` = slot 2) | Ability use. Fields: owner at +4, bar type at +8, slot at +0xc, selected target fields at +0x18 to +0x34, "pending" flag at +0x24, active flags at +0x60 to +0x62, force-dismount byte at +0x70. Owns `HashListMap<int,AbilityDefinition,100,-1>` (100 buckets, key at node +0x100) and a pending-request list at +0x238. | confirmed (dispatcher), likely (senders) |
| `AbilityDefinition` (ctor `01064290`, vtable `01880b3c`) and `EffectDefinition` (vtable `01880b54`, reader `01064c20`) | A definition node and its effect chain (+0xb4, next pointer +200). The text-row parse constructors have no static caller; the server-delivered form arrives as 36/13. | confirmed (ctor), unknown (reader) |
| `AbilityActorClickHandler`, `AbilityPlayerClickHandler`, `AbilityTerrainClickHandler` (ctors `00a2ef20`, `00a2f030`, `00a2f6e0`; dispatch `00a359a0`, `00a35a40`, `00a35920`) | Three handlers the processor registers on the `TargetProcessor` while an ability waits for a target. Event kind 8 (a click) sends the request; kinds `0x400`, `0x800`, `0x1000` copy vectors and flags. | confirmed |
| `TargetProcessor` (ctor `00acb770`, per-frame query `00acbc80`, candidate filter `00acb920`) | Ray and shape queries that turn the mouse into actors, players or terrain points, and the shared base for housing and ability click handlers. | confirmed ctor; unknown bodies |
| `Target` family (`TargetLocation`, `TargetCharacterGuid`, `TargetCharacterBoneId`, `TargetActorBoneId`, `TargetActorBone`, `TargetCharacterBone`) | A type code plus a 16-byte vector and, per type, an id pair and bone ids or names. Used inside launch-and-land packets and projectile parameters. Factory `0101c300` builds six variants from a type 1 to 6; read slot `0101c850`, write slot `0101c930`. | confirmed (S0563) |
| `BaseActionBar` / `ClientActionBar` (ctors `00c454b0`, `00c45510`), `ActionBarSlot` (ctor `0101cd80`) | A bar holds up to 8 buckets of slots (`HashListMap<int,ActionBarSlot,8,-1>`, 0x68-byte nodes, key at +0x60). `ClientActionBar` adds the owner at +0x3a8, the selected slot at +0x3ac and a 600-byte data source at +0x3b0. `Client_GetOrCreateActionBar` (`00a16d50`) keeps bars in a 5-bucket table at client +0x35434, named `BaseClient.AplActionBar`, `BaseClient.ItemActionBar` and `BaseClient.InvalidActionBar`. | confirmed |
| `HandlerActionBar` (Lua) | The UI side: init, show, hide, `setButtonData` (four parameters), `setItemCount`, `setTimer`, `setEnabled`, `setDisabled`, `setTargetingOn`, `setTargetingCanceled`. C++ callers `009cf6a0` to `009d0a70`. These are Lua and Flash calls, not network packets. | confirmed (S0276) |
| `ClientActionBarDataSource` / `ActionBarDataSource` | UI tables: `BaseClient.ActionBar` and the item bar's nine-column table (see Data). | confirmed |
| Character stat map | `HashListMap<int,CharacterStat,71,-1>` at client +0x35afc: stat id key at node +0x14, value dword at +8. Readers `00a12aa0`, `00a12ae0` (int with default) and `00a12b20` (float with default). | likely |
| `ClientEffectTag` map | `HashListMap<int,ClientEffectTag,20,-1>` (20 buckets, key at +0x80) on the client object at +0x6f0: the player's buff and effect-tag rows. Insert/find `00a19e70`, remove `00a14750`, receive-side removal `00a15440`. | confirmed / likely |
| Status flags dword | Actor +0x6f8: one bit per status. Written by the character-state update (35/20) and read by the cast gates. | pseudocode |
| `PlayerHitpointDeltaEvent` (vtable `018024f8`, ctor `008b81e0`, apply `00ae2350`) | The event that makes a floating number. Fields: signed delta, a "critical" byte and the id pair of the source. Sibling events: `PlayerMotionEvent` (`008b82a0`), `PlayerEffectEvent` family (`0095c230` and nine subclasses for particles, sound, model material, lights, camera, rumble, mesh and light-sabre trails), `PlayerProjectileEvent` (`00964e00`) and `ActorProjectileEmitter` (`00964ff0`). | confirmed |
| `ProjectileManager` (ctor `00b83d20`, update `00b84e90`) and `Projectile` plus `Beam`, `Arc`, `ThrowCatch`, `Seek`, `Boomerang` (base ctor `00bf34e0`, per-frame step in slot 2) | Client-side flight models. The manager is ticked from the client update (`00b84160`) and drained on zoning (`00b84050`). | confirmed ctors, speculative bodies |
| Boss registry (`00985200` add, `009852a0` remove) and `BaseClient.Bosses` | The set of actors drawn with a boss bar. A boss flag bit `0x80` sits on the actor. | confirmed (vault [[Native Loss Boss Display]]) |

The table lists 430 functions, but 28 of them are not combat. `ValidAttrModFilter` (14 rows) and `CheckFilterMatch` (1) are trading-card-game engine classes (S0749), `AttributeModifier` (11 rows) is the card game's attribute system (S0743), and two rows in `game.stats` (`00b404b0`, `00b40550`) log store purchases (S0343). See Open questions.

#### Using an ability: key press to server and back

1. **Key to slot.** The input dispatcher `00b02880` turns key actions 19 to 27 into a slot index and calls `008a7eb0` (`DispatchActionBarSlot`). It picks a mode and runs `008a7e40`: mode 1 is the job toolbar, mode 2 the item bar, mode 3 the minigame toolbar, mode 4 a Flash minigame that wants the raw key (`MiniGameFlash:OnKeyPressed`). The Lua dock can also call `Ui.ClickedAbility(index)` or `Ui.ClickedAbility(slot, 2)` for the item bar (client Lua `activateAbility`, `activateItem`).
2. **UI gate (job toolbar).** `009cf430` reads the slot's local flags. If the slot is disabled it prints a reason and stops; if it is already waiting for a target (flag bit 1) it cancels that wait (`00a2e620`); otherwise it calls `AbilityProcessor` `00a36a70(slot, 1)`. The reason is chosen in a fixed order from the player's status dword: knocked out, asleep, silenced, afraid, stunned, frozen, knocked back, popped up, then not enough energy (current energy at `client+0x3e0 + 0x488` against the slot's cost), else "default" (pseudocode of `009cf430`; strings in Data).
3. **Select and arm.** `00a36a70` stores the bar type at +8 and slot at +0xc, fetches the slot's targeting fields (bar type 2 through `00a129e0`, type 3 through `009ad390`, others from the ability set) into +0x18 to +0x20 and calls `00a35600`.
4. **Target mode.** `00a35600` first runs the status gate `00a30870`. That gate tests the status dword against mask `0x1542c6` (nine blocking states), or `0x1542ce` for the job bar (which adds silence; item use is not blocked by silence). On failure it resets (`00a2e620`) and returns. Then it branches on the slot's two targeting selectors: a self cast (first selector 1 with second selector 2 or 4) copies the player's own id and sends at once; the other selectors either need a current target (and call `00a2e6b0` when none is selected), arm the three click handlers and switch the slot's UI to "targeting" (`009d0570`), or arm ground targeting and draw the area ring `aoe_ring.dds` (`00a2f7f0`). Selector meanings are only partly decoded (see Open questions).
5. **Send.** `00a35150` builds `AbilityPacketClientRequestStartAbility` (36/10). It is skipped while the lockout test `00a2e540` is true (a 64-bit stamp at processor `+0x68`, set by 36/11 `MeleeRefresh` as a base value plus the packet's value, that is less than 1,000 ms old or still in the future; pseudocode) and when neither the "pending" nor the "active" flag is set. It picks the nearest type-1 actor (a creature) from a spatial query when no explicit target was clicked, turns the player toward it (`0095dae0`), writes the request through `00a35000` into a tunneled client packet and resets the arming state. Wire layout: `u16` family 36, `u16` sub 10, `i32` bar type, `i32` slot, `i32` target mode, then an 8-byte id when the mode is 0 or four floats otherwise (writer `00a342e0`, pseudocode). With no slot selected and a pulse location pending, the same function sends 36/16 (`ReceivePulseLocation`) with a vector instead (`00a34720`).
6. **Cast start (server to client).** 36/3 `StartCasting` (`00a31aa0`) names the caster, a cast time, an animation id and a composite effect id. The handler resolves the caster actor; when the caster is the local player and the cast time is above a threshold it calls `ActionProgress:SetActionTime` to show the cast bar; a positive animation id builds an animation event (`00959bf0`) on the caster; a composite effect id is looked up and played, and a continuous effect with no animation logs "Casting composite effect id %d on ability id %d is continuous but no animation associated" to `DesignAlerts.log`.
7. **Results.** Hit results arrive as combat-family frames (32/7 and friends) and as `35/5`, `35/35` and `38/1` health updates (next sections). Failure arrives as 36/1 `Failed` with a string id the client prints as a message (`00a31cd0`, `0097be40`). A cast can be cancelled by 36/18 `CastInterrupt`, a projectile detonated by 36/14, an aura stopped by 36/9, and a server-driven cast started by 36/6 `ClientMoveAndCast`.
8. **Bar refresh.** 38/25 (`UpdateActionBarSlot`, handler `009392c0` case `0x19`) replaces one slot on a bar (the item bar through `00a12980`, a minigame bar through `009affd0`). The 36/5 `SetDefinition` packet replaces the whole job toolbar: the handler logs "Received ability set definition", rebuilds the `BaseClient.ActionBar` data source and refreshes tooltips (`00a35cc0` case 5).

#### Basic attack, engagement and Tab targeting

- **Tab.** `00a2ac20` walks the nearby actor list and keeps creatures (kind 1) that are hostile (disposition value 0 at actor +0x178), not knocked out (status bit 7 clear), in front of the player (positive dot product against a constant `0.0`) and visible through a ray cast. Pressing Tab again cycles through them. The chosen id pair is stored locally (`008c6c10`); the server learns about it only inside the next attack or ability request (client, via [[Native Combat Controls]]).
- **Click or fight key on a target.** `008c6470` (called every frame and on movement input) approaches and attacks the stored target. It only runs while the client is in a combat state: either the byte at `[client+0x778]+0xc` or the byte at `client+0x80` is set. It reads the Weapon Range stat (id 3, float, default 5.0) and multiplies by the constant 0.8, so a character without that stat engages at 4.0 units. Inside that distance it sends `CombatPacketSingleAttackTarget` (32/3, `00a2c880`) or, in auto-attack mode, `CombatPacketAutoAttackTarget` (32/1, `00a2c7d0`) and sets a flag; outside it, it walks toward the target. A target whose status bit 14 (`GoingHome`) is set is skipped, and a character whose Weapon Range stat is exactly 0.0 never engages (pseudocode). The body of both packets is the family header and one 8-byte target id.
- **Interaction menu.** Clicking a creature that offers an interaction menu sends `CommandPacketInteractionSelect` (26/10, ctor `00a36f80`) with the chosen entry id. The accepted native fights use this path: menu entry 103 means attack (101 exit, 102 retry, 104 guide start in the same menu; vault [[054 Melee attacks]], [[Native Combat Controls]]). The server supplies the menu with 26/9, so which entries a creature offers is server content.
- **Swing animations.** Every combat job wield type plays swing group 1099 (`com_swing`, fallback group 1000); slot `com_h2h_attack_01` is animation id 1001; the recoil is 1120, special 1011, knock-down 1402, death 1151 and static death 1152 (client resources `CodeAnimationMappings.txt`, `AnimationGroups.xml`, `AnimationTypes.xml`). Entering a combat animation is a single play with 0.2 s blends; leaving eases back over 0.6 s (`AnimationTransitionRules.xml`). The clip length is not recovered.

#### Casting, projectiles and effects

- **Cast time.** The cast time is a float inside 36/3; the cast bar is a UI call, not a timer the client enforces. Abilities have `CastSeconds`, `AuraDuration`, `ManaCost`, `ManaCostPerSecond`, `MaxAoeTargets` and a `Description` field in the definition node (named-field builder `00a309c0`, pseudocode of the string constants).
- **Effect rows.** Each definition has a chain of effects. `00a30d20` builds their named fields: `ChanceToOccur`, `DurationSeconds`, `MaxCharges`, and `Param1`, `Param2`, `Param3`, each with `TunedLow` and `TunedHigh`. The tooltip code (`00a34e30`, `00c20420`) turns them into text rows using `AbilityEffectTypeSettings.txt`. Row `Param1`/`Param2` for type-2 effects is a character stat id, so a buff is "stat X changes by N" (Data).
- **Projectiles.** A projectile is a composite-effect child or a launch event, then simulated on the client. Five motion classes exist (beam accelerates along a direction, arc and throw-and-catch interpolate a 12-float segment, seek ray-casts for a target, boomerang steps toward the target and returns). The server's part is the bracketing frames: 35/62 launch, 36/4 launch-and-land (a `Target` array plus `ProjectileParameters`), 36/14 detonate. The composite-effect definition file lists no PROJECTILE children at all (0 of 5,387 effects; the XML writer knows type 5), so projectile definitions come from elsewhere.
- **Composite effects.** 1,988 definitions in `ActorCompositeEffectDefinitions.xml` map an id to timed particle, sound, material, light, camera, rumble and mesh children. The server asks for them by id (35/16 `PlayCompositeEffect`, 35/36 `TriggerEffectPackage`, ids inside 32/7 and 36/3). `5017` is `PFX_knocked_out_loop`; the character-state handler plays it when the knocked-out bit turns on.

#### Cooldowns, energy and the cast gate

- **Slot refresh time.** The 51-byte slot carries a total refresh time and a cast age; the client keeps a 64-bit clock pair in each slot (`+0x40` snapshot, `+0x48` running) that `00c45450` advances every tick, and derives the remaining time as total minus elapsed (`0101ce90`). `ClientActionBarDataSource` exposes it as `Refresh Time Left` and `Total Refresh Time` columns (client; the mapping of the wire fields onto these columns is partly inferred).
- **Local cooldown feel.** When the client sees its own attack result (32/4, 32/5, 32/6 or 32/7 whose first id pair is the local player), it cancels ground targeting on every slot (`009d09f0`), starts a 500-unit timer on every other slot (`009d0a20`) and starts the used slot's timer from the Weapon Speed stat (id 15, default 1000, `008a8d10`, `009cfd60`). Units are probably milliseconds (pseudocode, not proven).
- **Energy gate.** The client refuses locally when current energy is below the slot's cost. The server must still check, because the client value is a UI aid.
- **Level gate.** A slot carries a required level (+0x24) compared with the player's level (`00c43cd0`); the activation check `00c43fd0` combines level, status mask and an "enabled" byte.

#### Damage, healing and the health bar

- **Local health and energy.** 38/1 carries two `i32`, current then maximum health; the handler stores the current value at `client+0x3e0 +0x484` and drives the player's health bar from `current * 100 / max`. 38/13 carries an `i32` (current energy, stored at `+0x488`), a second `i32` and a flag byte (pseudocode of `009392c0`, readers `008d8210`, `008d8280`).
- **Other actors' health.** 35/5 (`UpdateHitpoints`, reader `008d6f10`) carries an id pair and three `i32`; the handler skips the local player's own actor and queues a health-bar event on every other actor.
- **Attack results.** The five combat frames the client can receive:
  - 32/4 `AttackTargetDamage` (reader `00a2aab0`): two id pairs, an `i32`, a byte. Its handler starts the attacker's cooldown feedback and a motion event; it builds no number and no health-bar event. What it adds beyond 32/7 is open.
  - 32/5 `AttackerMissed` (`00a2a7a0`): two id pairs. The first actor plays a motion event; the second shows `MISS!`.
  - 32/6 `TargetDodged` (`00a2a820`): two id pairs; stores the second pair on the first actor at +0x4c8 and runs the local feedback calls. No text.
  - 32/7 `AttackProcessed` (`00a2a910`): the main result. Three pairs are read, but the second overwrites the first, so the frame is attacker, attacker, target; then `i32` +0x20, `i32` +0x24, `i32` +0x28, two bytes (+0x2c, +0x2d) and two `i32` (+0x30, +0x34). The handler `00a2ba40` (pseudocode) uses +0x20 as the damage taken, +0x24 as the target's maximum health and +0x34 as the target's current health when the client has none. On the target it queues (a) a `PlayerHitpointDeltaEvent` with delta -damage, the byte at +0x2c as the critical-style flag and the attacker id; (b) a health-bar event with `(current - damage)` clamped at 0 over the maximum; (c) a motion event (the hit reaction) when the byte at +0x2d is 0; and (d) a composite effect looked up from +0x28. It stores the target pair on the attacker at +0x4c8, clears the local target when the dead actor was the local player's target, and runs the local cooldown feedback. A second composite effect is queued on the attacker through `00a2a540`.
  - 32/9 `EnableBossDisplay` (`00a2a8a0`): an id pair and a bool; adds or removes the actor from the boss registry.
- **Healing and damage over time.** 35/35 (`HitPointModification`, reader `008d6c50`: two id pairs, a bool, three `i32`, a bool) is dispatched into `BaseClient` slot 65 (`008d3020`). When the flag is set and one of the pairs is the local player, a hitpoint-delta event is built. A positive delta prints `+%d`. This is the only visible path for heals and poison-style ticks.
- **Floating text.** `PlayerHitpointDeltaEvent::Apply` (`00ae2350`) formats the delta and calls `008bb0b0`, which colours the text (red `0xffaa0000` when the flag is clear, a computed colour otherwise) and hands it to the actor's text slot (+0x50 or +0x58). The known Flash path is `00844d80` (`ShowDmgNum`) and `00845010` (`ShowEffect`): both resolve `Main.wndSceneText.swfSceneText`, build GFx parameters (a `%llu` 64-bit amount string, a string and, for damage, a `%d` int) and invoke the movie function. A byte scan of the whole executable finds no call, jump or pointer reaching either function, so what calls them is open. `009d2170` ticks the movie (`GiveTime`) every frame.

#### Statuses and effect tags

- **Status dword.** 35/20 (`UpdateCharacterState`: id pair plus one `i32` of flags) sets the actor's status bits. On every change the handler compares old and new bits and shows floating text for each bit that turned on, using the locale text mapped from the `CharacterStatus...` key (`CharacterStatusAfraid` and eleven more, plus `GoingHome`, which reads "Immune"; table in Data). The bit table is in Data. Bit 7 (knocked out) is special: when it turns on the handler plays composite effect 5017, drops a boss from the boss registry and clears the local target; when it turns off it restores the actor (pseudocode).
- **Blocking mask.** The same bits drive the cast gate (mask `0x1542c6`, with silence added for the job bar). Bits 14 and 18 are in the mask but have no message; they fall through to "You can't use abilities right now!" (locale 286).
- **Effect tags (buffs).** 38/16 adds and 38/17 removes a tag keyed by id. The client keeps decoded rows in the 20-bucket map and feeds `BaseClient.PlayerEffectTags`, a UI table read as (instance id, icon id, duration, time left, name) with zero-icon rows ignored (client Lua `GroupHandler.BuffsDsName`). Removing a Power Hour tag fires `HUD:PowerHourStop`. 35/41, 35/42 and 35/43 are the matching "effect-tag composite effect" frames (add, remove, enable) played on the actor; 35/43's body is not read.
- **Immunities.** The locale names stun, slow, poison, fear, knockback, root and status immunity (ids 70204, 70209, 70214, 436675, 438016, 438557, 438559). The client has no immunity logic; it is server-side.

#### Knockout, revive and the combat flags

1. The server sets status bit 7 with 35/20. The client blocks every ability with "You can't use abilities when you are knocked out!" (locale 280).
2. The server sends the respawn window as `41/125` (`EncounterShowRespawnWindowPacket`, body two `i32`; dispatcher `0090d830`). The handler calls `GroupHandler:SetKO` and `00904300`, which formats `value / 1000` as seconds (a non-positive first value falls back to a client default at `client+0x918`) and calls the Lua event `RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)`. The first value is a millisecond timer and the second the revive cost (vault [[058 Death and revive]]; order inferred from the handler).
3. `isOverworld` is the byte at `client+0x80`. That byte is set by **41/132** (`OverworldCombat`, dispatcher `00aa36c0` case `0x84`, setter `009058d0` = `SetInWorldCombatFlag`, which also calls `CallHandler:SetInWorldCombat`). 41/133 sets the sibling byte at `client+0x81` through `00905a50` (`GameDock:EnableCombatIndicator`). This closes a gap the vault left open: piece 058 could not find the writer of `client+0x80`.
4. When the flag is set the window shows two buttons with "Revive here: N coins" (locale template); otherwise it counts down. The `respawn` event calls `Ui.RevivePlayer()` and a paid choice calls `Ui.OnPaidRevive(b)`. The paid path sends `EncounterParticipantResumePacket` (41/122, `0093be30`, a bool at +0x14); the free path uses the same class without the flag (`0093bd80`). 41/126 hides the window.
5. After recovery the server clears bit 7, restores health and energy (38/1, 38/13) and, for an instance, counts the knockout against the limit.

#### Boss bars, results and the prize wheel

- **Boss bar.** 32/9 puts an actor in the boss registry (`BaseClient.Bosses`), and the HUD `Target` movie shows or hides itself from that table's row count (vault [[Native Loss Boss Display]]). Minigame bosses use a second table, `MiniGame.Bosses` (Name, Health), fed by `39/62` with a `u32` discriminator, 1 update or 2 delete (`00c48c90`).
- **Results.** A won or lost battle ends with `39/18` (`GameOver`, `009ad480`), `39/47` (score rows, `009bb030`) and `39/19` (`StateRemove`, `009bf890` then `ShowMiniGameEndDialog`). The Lua `ScoreScreen` builds pages from non-empty reward, stat and score data; with no pages the movie closes itself and sends the exit request (`41/109`, header 0 and the active instance). Only the player's Close, or the guide's Exit, should end the encounter (vault [[Native Combat Results Lifecycle]]).
- **Prize wheel.** `PopulateLootWheel` reads two data sources, `BaseClient.MiniGame.RewardPreview.Entries` (item prizes) and `BaseClient.MiniGame.RewardPreview` (a coin amount and a star amount), appends a coin slot and a star slot, and shows the wheel only when the game status is a win and at least one row exists. `SpinLootWheel` stops the movie on an index; the visible reward is added to the screen's list. The real grant is a separate reward bundle (family 50), see [[Rewards and Collections]].

#### Progression hooks

- `38/14` updates profile experience, `38/18` the profile rank, `38/15` and `38/24` add and update the profile's ability set, `38/22` adds an ability and `38/20` deletes a profile (handler `009392c0`). `JobLevelUp` is raised on rank gains (two cases), and `AbilityReceived` and `NotificationAbilityUnlocked` strings sit in the same function.
- Ability ranks: 36/8 `UpdateAbilityExperience` (server to client) and 36/7 `PurchaseAbility` (client to server, `00a34c80`, a 12-byte local record) cover spending Stars on ranks.
- Definitions are cached by id. A lookup miss (`00a34d10`) queues the id in a pending list and sends 36/12 `RequestAbilityDefinition`; the answer is 36/13.
- The wire to switch jobs is outside this guide ([[Character and Customization]]); the client side of a swap logs `JobHandler:OnProfileSwapComplete` (`00963410`).

#### Combat jobs

Ids in brackets are job profile ids from the client's own `ItemFilterOptions.txt` (see [[Character and Customization]]); the vault job catalogue in piece 031 is not used because it rests on a recorded server payload and a community profile table. Rank texts are the client's five-rank tooltip numbers; they are not damage (see Open questions). Weapon-tier names are developer comment strings in the client; they list each weapon family's "spammable" and "ultra" abilities.

| Job (profile id, wield type) | Level-gated content (wiki names, client text) | Weapon abilities by tier, spammable / ultra (client) |
|---|---|---|
| Brawler (43, 13) | Traits: 5 Bruising Strikes (weaken, more crits), 10 Savvy (extra crit damage), 15 Toughness, 20 Resilience (health after being hit; wiki also says energy, client says health only) | T1 Glancing Blow, Wallop / Leg Sweep, Ready to Rumble. T2 Smack, Pound / Pummel, Roundhouse Kick. T3 Wild Swing / Sucker Punch |
| Ninja (2, 9) | Traits: 5 Ultimate Power (energy on crit), 10 Shrouded Armor, 15 Ninja's Grace (speed, regen, attack), 20 Dragon's Boon (20% counter, lightning chains up to 3 targets) | T1 Flame Flash, Lightning Strike / Dragonstrike, One-Thousand Storms. T2 Cinder Slash, Twisted Edge / Flame Wave, Shuriken Storm. Ranked job abilities in text: Shuriken Toss (50 to 75, up to three targets), Shadow Minion (10 s), Smoke Bomb (stun 4 s) |
| Archer (35, 11) | Traits: 5 Precision, 10 Marksmanship, 15 Reflexes (run speed, dodge), 20 Lucky Shot (energy) | T1 Barrage, Icy Arrow / Volley, Blizzard Blast. T2 Multi-Shot, Charged Shot / Splitting Arrow, Explosive Shot. T3 Power Shot / Stunning Shot. Firebomb, Ricochet at tier 5 |
| Warrior (32, 12) | Abilities: 1 Kick (133 to 200, stun), 5 Spinning Blade (50 to 75 per hit, two hits), 10 Quake (83 to 125, taunt), 15 Commanding Shout (50 to 75, fear, group buff). Traits named in text: Warrior's Spirit, Warrior's Boon, Crushing Blows | T1 Slash, Sweeping Slash / Cleave. T2 Power Slash, Dual Strike / Quake, Warcry |
| Wizard (12, 10) | Abilities: 1 Zap (83 to 125, up to 3 targets), 5 Fireball (50 to 75 plus 10 s burn), 10 Ice Nova (3 s freeze, +3 to 15% damage taken), 15 Chaos (confuse 5 to 9 s) | T1 Zap, Chill / Lightning Blast, Ice Nova. T2 Burn, Splash / Firestorm, Tsunami |
| Medic (11, 14) | Abilities: 1 First Aid (heal 250 to 375, self and nearby allies), 5 Target Vitals (133 to 200), 10 Vitamins (grow, +1 to 5% crit chance, +10 to 50% crit damage, 5 to 9 s), 15 Shock Paddles (50 to 75, three targets; also revives allies) | T1 Incision, Injure / Triage, Target Vitals. T2 Assist, Bruise / Immunize, Nurse! (summons nurses) |
| Adventurer (1) | None; non-combat | None |

The Warrior gate is enforced by NPCs that turn the player away until "you learn your next ability" (client locale 3158281165, 2023139908). Brawler's Pummel is learned (locale 2232233194, 2757299385).

### Data and formats

**What the client ships and what it does not.** The client has **no ability, effect, damage, enemy, weapon-stat or experience table**. A listing of the preserved `Resources/` folder shows only the files below. Ability and effect definitions arrive from the server (36/13, with the toolbar set in 36/5); stat values arrive with the character; enemy health, damage and behaviour are server-only; names, descriptions and many rank numbers are locale strings. The older `AbilityDefinition` text-row constructors exist in the executable but have no static caller (S0575, see Main classes), so no on-disk ability list is read.

| File | Contents | Source |
|---|---|---|
| `AbilityEffectTypeSettings.txt` | 37 rows: `ID^EFFECT_TYPE^TOOLTIP_STRING_ID^IMAGE_ID^DISPLAY_ORDER^PARAM_1^PARAM_2^PARAM_3`. Effect type 1 = direct damage (row 38), 2 = stat modifier (34 rows), 3 = instant restore (rows 3 and 4). | client resource |
| `CharacterStatDefinitions.txt` | 58 rows: `ID^NAME_ID^IMAGE_ID^DESCRIPTION_ID`; ids 1 to 48 and 59 to 68. Loaded by `0102b8a0` into a 71-slot table (ids up to 70). | client resource, S0566 |
| `CharacterStatSetMappings.txt` | 67 rows in 18 stat sets (`SET^STAT^ORDER`): the stats a panel shows. Sets 2, 4, 5, 8, 9, 12 list Health, Energy, Defense, Star Boost, Coin Boost; sets 1, 10, 11, 14, 15, 17 list only Star Boost and Coin Boost; sets 3, 6, 7, 13 list Star Boost, Talent, Timing, Coin Boost; set 16 adds three soccer stats; set 18 lists four glide stats. Which set belongs to which profile is not recovered. | client resource |
| `ActorCompositeEffectDefinitions.xml` | 1,988 `EffectDefinition` rows (id, name, `defaultLifeTime`, `loadType` always 0) with 5,387 timed children: 2,843 sound, 2,319 particle, 129 light, 46 rumble, 31 model material, 15 camera, 4 mesh. No child is a projectile. By name prefix: `PFX` 1,226, `WFX` 266, `SFX` 136, `BBE` 136, `EFX` 124, `PRJ` 78, `PCX` 10, `MX` 8. | client resource |
| `WieldTypeAnimationMappings.txt` (26), `CodeAnimationMappings.txt` (155), `AnimationGroups.xml`, `AnimationTypes.xml`, `AnimationTransitionRules.xml` | Combat animation ids and timing; see below. | client resource |
| `Cursors.txt`, `CodeStringMappings.txt` | Combat cursors; the `NoCast*` message ids. | client resource |
| `hp_bar_green.dds`, `hp_bar_red.dds`, `hp_bar_green_segment.dds`, `hp_bar_red_segment.dds`, `hp_bar_shields.dds`, `aoe_ring.dds` | Health-bar and area-ring art. | client resource |
| Key table at `0x1b41988` | 24-byte records, 48 read; the combat defaults are in Overview. | client, vault [[Native Combat Controls]] |

**Character stats (client resource, locale names).** A stat is an id and a value dword in the 71-slot map at client `+0x35afc`. Named stats: 1 Health, 2 Movement Speed, 3 Weapon Range, 4 Health Refill, 5 Energy, 6 Energy Refill, 7 Defense, 8 Melee Avoidance, 9 Melee Critical Hit Chance, 10 Melee Critical Hit Multiplier, 11 Melee To-Hit Chance, 12 Weapon Damage Multiplier, 13 Hand-to-Hand Damage, 14 Attack, 15 Weapon Speed, 16 Absorb Damage, 17 Star Boost, 18 Damage Reduction, 19 Damage Addition, 20 Damage Multiplier, 21 Healing Addition, 22 Healing Multiplier, 33 Ability Critical Hit Chance, 34 Ability Critical Hit Multiplier, 35 Luck, 37 Coin Boost. Non-combat stats share the table: 24 Talent, 27 Timing, 39 Kick Power, 40 Footwork, 41 Speed, 42 Toughness, 43 Tackle Power, 44 to 48 fishing, 59 to 64 and 67 glide. Ids 23, 25, 26, 28 to 32, 36, 38, 65, 66 and 68 have no name (65 and 66 are the health and energy regeneration stats that four effect rows point at). The client reads three of them in code: Weapon Range (id 3, float, default 5.0), Weapon Speed (id 15, int, default 1000) and, for the stat window, the set mapping above. No formula combines the others; the names say that to-hit, avoidance, critical chance and multiplier, defense, damage reduction, absorb and addition and multiplier terms existed on the server (a reading of the names, not a recovered rule). The value units are unknown.

**Effect-type rows (client resource, locale tooltips).** The tooltip text uses placeholders such as `#number([*Param2TunedLow*])` that the named-field builders fill (below).

| Row ids | Type | Param 1 (stat id) | Tooltip meaning |
|---|---|---|---|
| 38 | 1 damage | 0 | "Deals N damage." |
| 3, 4 | 3 restore | 0 | "Heals you for N - M health", "Restores N - M energy" |
| 1, 2 | 2 | 1, 5 | health / energy "by N in combat" |
| 6, 7 | 2 | 14, 7 | + Attack, + Defense |
| 8, 9, 10 | 2 | 17, 37, 35 | + % Stars, + % Coins, + % Luck |
| 30, 31, 32, 33 | 2 | 9, 33, 8, 18 | melee critical chance %, ability critical chance %, dodge chance %, damage reduction % |
| 34, 35 | 2 | 22, 2 | heals "N times", run speed "N times" |
| 36, 37, 39, 40, 44 to 47 | 2 | 6, 4, 4, 6, 65, 65, 66, 66 | energy and health regeneration (percent on some rows; the Param 3 column is 1 on rows 36, 37, 45 and 47 and its meaning is not recovered) |
| 41, 42 | 2 | 16, 20 | damage reduction (absorb), "ability damage by 20%" |
| 20 to 29 | 2 | 23 to 32 | minigame skill stats (Matcher, Maker, Talent, Timing); not combat |
| 43 | 2 | 0 | empty text |

Row 42 is the only row with a fixed number in its text. How an effect record picks one of these rows is not recovered; the tooltip builders (`00a34e30`, `00c20420`) do the lookup.

**Ability definition record (36/13 and the set inside 36/5), pseudocode of `00a32930`.** Read after the 4-byte header (`u16` family, `u16` sub), in order: `i32` ability id (`+8`); two `bool`; three `i32`; a `f32` at `+0x1c` that the named-field builder reports as `CastSeconds`; eight `i32` from `+0x20` (the one at `+0x38` is `ManaCost`); an `i32` at `+0x40`; two NaN-checked floats at `+0x44` and `+0x48`; `i32` at `+0x4c`, `+0x50`, `+0x58` (`ManaCostPerSecond`); a `bool` at `+0x5c`; `i32` at `+0x60` (`AuraDuration`), `+0x68` (`MaxAoeTargets`), `+0x70`, `+0x74`; five more floats and dwords up to `+0xa8` with four more `bool`s; and finally the counted effect list through `00a32230` (an `i32` count, then per entry an `i32` key and a body read by `00a30360`) plus a last `bool`. The description is the locale id at `+0x14`, resolved as `Global.Text.<id>` (`0097b520`). Roughly 45 fields; only the named ones above have known roles.

**Effect record body (pseudocode of `00a30360`).** Dwords at `+4` to `+0x14`, a `bool` at `+0x18`, dwords `+0x1c`, `+0x20`, NaN-checked floats `+0x24`, `+0x28`, `+0x2c`, dwords `+0x48` to `+0x5c`, `bool`s `+0x60`, `+0x61`, a counted byte span, floats `+0x64`, `+0x68` to `+0x78`, then more dwords (tail not decoded). The named fields the UI can substitute are `ChanceToOccur`, `DurationSeconds`, `MaxCharges`, `Param1`, `Param2` and `Param3`, each of the three params with `TunedLow` and `TunedHigh` (`00a30d20`). The ability-level fields (`ManaCost`, `CastSeconds`, `AuraDuration`, `ManaCostPerSecond`, `MaxAoeTargets`, `Description`) come from `00a309c0`. The meaning of "Tuned" low and high (rank 1 and rank 5, or a roll range) is not recovered.

**Rank text.** Rank numbers are not computed by the client. The five rank lines of a skill are separate locale entries with the number written in (for Zap: damage 83, 94, 104, 115, 125; Fireball 50 to 75; Ice Nova 3 to 15 percent; Chaos 5 to 9 seconds; see the combat job pages [[Wizard]], [[Brawler]], [[Warrior]], [[Medic]], [[Ninja]], [[Archer]]). Templated lines such as "Increases critical hit chance by #number([*Param2TunedLow*])%." take their number from the definition. Both forms exist, so a definition's `Param2TunedLow` and the rank sentence must agree.

**Toolbar slot record (51 bytes, native read order of `008dae30`, vault [[048 Ability bar]]).** Used by 38/25; the job toolbar and the item toolbar share the layout.

| Offset | Type | Role |
|---:|---|---|
| 0 | bool | empty flag (activation of an empty slot is refused) |
| 1 | i32 | icon id |
| 5 | i32 | icon tint id |
| 9 | i32 | name locale id |
| 13 | i32 | first targeting selector (1 with second selector 2 or 4 is a self cast) |
| 17 | i32 | second targeting selector |
| 21 | f32 | targeting parameter (range or radius; unit not recovered) |
| 25 | i32 | mana (energy) cost |
| 29 | bool | server enable flag |
| 30 | i32 | refresh-related (elapsed or total; unresolved) |
| 34 | i32 | total refresh time |
| 38 | i32 | refresh-related (unresolved) |
| 42 | i32 | displayed quantity (item stacks) |
| 46 | bool | force-dismount flag copied into the cast state |
| 47 | i32 | cast age; `2147483647` selects the "never cast" branch |

Packet 38/25 is `u16 38`, `u16 25`, `i32` bar id, `i32` slot index, then this record (63 bytes). The runtime slot object is larger (0x68-byte nodes) and holds a level requirement at `+0x24`, and a 64-bit clock pair at `+0x40` and `+0x48`. UI columns: the job bar `BaseClient.ActionBar` has Assigned, IconID, Name, Description, Quantity, ManaCost, TintValue, AbilityId; the item bar and the minigame bar have IsEmpty, IconID, Name, IsEnabled, RefreshTimeLeft, TotalRefreshTime, Quantity, IsTargeting, IconTintValue (client Lua `ScriptsBase.lua`). Which wire field feeds `RefreshTimeLeft` is not recovered.

**Ability request (36/10), pseudocode of `00a342e0`.** `u16 36`, `u16 10`, `i32` bar type (1 job, 2 item, 3 minigame), `i32` slot, `i32` target mode, then `u64` target id when the mode is 0 or four `f32` (a position and facing) otherwise: 24 or 32 bytes. Our server parses exactly these two sizes. The self cast sends the player's own id.

**Combat result frames (read order of the reader functions).** 32/7: header, three `u64` ids (attacker, attacker again, target), `i32` damage (`+0x20`), `i32` target maximum health (`+0x24`), `i32` composite effect id (`+0x28`), `u8` critical-style flag (`+0x2c`), `u8` suppress-reaction flag (`+0x2d`), `i32` (`+0x30`), `i32` health (`+0x34`). 32/5 and 32/6: two `u64` ids. 32/4: two `u64` ids, `i32`, `u8`. 32/9: `u64` id, `bool` (13 bytes). 35/5: `u64` id and three `i32`. 35/20: `u64` id and `i32` status flags. 35/35: two `u64` ids, `bool`, three `i32`, `bool`. 38/1: two `i32` (current, maximum health). 38/13: `i32` current energy, `i32`, `u8` flag.

**Status bits (client strings and shifts read in `0092f460`).** The 35/20 handler compares old and new flags. For each bit that turns on it resolves a `CharacterStatus...` key through the string-mapping table and hands the text to the actor's floating-text slot (`008bb0b0`, the same call damage numbers use; pseudocode). The mapped locale text is: Afraid 7928, Asleep 7925, Bound 7926, Rooted 7924, Stunned 7927, Silenced 7929, Frozen 7932, Berserk 7933, Confused 7934, and two that read differently from their names: `Nonattackable` 7930 shows "Invulnerable" and `GoingHome` 7931 shows "Immune". `KnockedBack` and `PoppedUp` have no row in the mapping table. `Visible` 7922 and `Invisible` 7923 also exist in the table but are not used by this handler.

| Bit | Name (`CharacterStatus...`) | Blocks casting | Notes |
|---:|---|---|---|
| 1 | Afraid | yes | |
| 2 | Asleep | yes | |
| 3 | Silenced | job bar only | item use is not silenced |
| 4 | Bound | no | |
| 5 | Rooted | no | |
| 6 | Stunned | yes | |
| 7 | (knocked out) | yes | no name string; plays composite 5017, drops boss registry entry, clears target |
| 8 | Nonattackable | no | |
| 9 | KnockedBack | yes | |
| 13 | Confused | no | |
| 14 | GoingHome | yes | no message; makes the creature untargetable for Tab and attack |
| 15 | (unnamed) | no | on actors flagged as bosses it adds or removes the boss registry entry |
| 16 | Frozen | yes | |
| 17 | Berserk | no | |
| 18 | (unnamed) | yes | no message |
| 20 | PoppedUp | yes | |

Cast gate masks: `0x1542c6` (bits 1, 2, 6, 7, 9, 14, 16, 18, 20) and `0x1542ce` for the job bar (adds bit 3). The "dead" test used by Tab targeting is bit 7 of the dword at actor `+0x6f8`.

**Locale ids used by combat (client locale).** Cast refusals: 280 knocked out, 281 asleep, 282 silenced, 283 afraid, 284 stunned, 285 not enough energy, 286 default "You can't use abilities right now!", 287 recast timer "That ability isn't ready yet...", 288 no target "You need to choose a target!", 3079 "You can't use that ability right now." (`NoCast...` rows of `CodeStringMappings.txt`). Knockout window: 410188 "You Got Knocked Out!", 408280 "Recover in", 408279 "Recover Now!", 419881 "@1 Knockouts Remaining". Immunities: 70204 stun, 70209 slow, 70214 poison, 436620 forced effect, 436622 illusion, 436675 fear, 438016 knockback, 438557 death-prevent plus knockback and root, 438559 status. Tutorial: 75707 to 75719 (the opening robgoblin fight: click or press 1; the leg-sweep special), 10265 and 10268 (Gloamling tutorial: Basic Attack on key 1; "Super Attacks drain a lot of energy! Use an energy potion ... pressing the 6 key"), 419869 "Attack", 419870 "Super Attack", 419871 "Power Up", 422858 "Combat Controls". Damage popups: 401622 "DAMAGE!", 401585 "Damage Dealt".

**Combat animation ids (client resource).** Group 1000 `com_h2h_attack` and slots 1001 to 1003 (`com_h2h_attack_01` to `_03`); group 1099 `com_swing` (fallback 1000) used by the job wield types 9 to 17, 22 to 24, 29 and 30; 1011 `com_h2h_special_01`; 1120 `com_recoil_01`; 1151 `com_death_01`; 1152 `com_death_static_01`; 1200 `com_fidget`; 1300 `com_wieldtype_stand`; 1401 `com_stunned`; group 1402 `com_knock_down`; 1403 `com_get_up`; 1406 `com_dodge`. Slot 1001 is upper-body only, `doNotInterrupt` 1, priority 10. Entering a combat animation is a single play with 0.2 s blends; leaving eases back over 0.6 s. Non-job wield types play 1000 (hand to hand), 1020, 1040, 1060, 1080, 1102 (bow), 1133 (wand); thrown plays none. The recoil, death and knock-down clips' durations are not recovered.

**Boss HUD source (client Lua).** `BaseClient.Bosses` has columns 0 guid (portrait), 1 name, 2 and 3 health current and maximum, 4 and 5 power current and maximum; the `Target` movie shows the first row and hides when there are none (`setHealth(x, 0)` is treated as maximum 100). `MiniGame.Bosses` has Name and Health.

**Effect-tag UI source.** `BaseClient.PlayerEffectTags` rows are read as instance id, icon id, duration, time left, name; a zero icon is ignored. The wire body of 38/16 is a tag id followed by a record whose field order is not decoded in the map (the vault note [[056 Combat status effects]] gives a 93-byte total taken from server-side sources this guide does not use, so treat the size as unchecked against the client).

### Network

All client-to-server traffic below is wrapped by the gateway as a tunneled client packet (`PacketTunneledClientPacket`, client gateway id 5; [[Gateway and Transport]]). Family and sub-opcode are two `u16`. Directions are proven from the sender (C>S) or from the dispatcher case that parses the frame (S>C). The packet catalogue the map validates against has combat-family classes for none of these families, so `wire` is set on only 10 of the 430 rows. Decimal throughout; hex handler addresses in the last column.

| Fam/sub | Name | Dir | Body | Handler or sender |
|---|---|---|---|---|
| 32/1 | AutoAttackTarget | C>S | `u64` target id | `00a2c7d0` (ctor `00a29c20`) |
| 32/3 | SingleAttackTarget | C>S | `u64` target id | `00a2c880` (ctor `00a29cf0`) |
| 32/4 | AttackTargetDamage | S>C | 2 x `u64`, `i32`, `u8` | reader `00a2aab0`, handler `00a2bff0` |
| 32/5 | AttackerMissed | S>C | 2 x `u64` | `00a2a7a0`, `00a2b360` (shows `MISS!`) |
| 32/6 | TargetDodged | S>C | 2 x `u64` | `00a2a820`, `00a2b580` |
| 32/7 | AttackProcessed | S>C | see Data | `00a2a910`, `00a2ba40` |
| 32/9 | EnableBossDisplay | S>C | `u64`, `bool` | `00a2a8a0`, `00a2b910` |
| 36/1 | Failed | S>C | `i32` string id | `00a31cd0` |
| 36/3 | StartCasting | S>C | 2 x `u64`, 4 x 32-bit (the last is a NaN-checked float), `u8` | `00a31aa0`, reader `00a2fc70` |
| 36/4 | LaunchAndLand | S>C | target array, ProjectileParameters, about 20 fields | virtual slot 3 `00a33760`, reader `00a31f30` |
| 36/5 | SetDefinition | S>C | `i32`, then the ability set | `00a32fe0`, applied in `00a35cc0` case 5 |
| 36/6 | ClientMoveAndCast | S>C | 4 x `f32`, `i32` bar, `i32` slot | `00a33090` |
| 36/7 | PurchaseAbility | C>S | 12-byte local record | `00a34c80` via `00a345f0` |
| 36/8 | UpdateAbilityExperience | S>C | six `i32` (key fields) | `00a31bf0`, reader `00a2fef0` |
| 36/9 | StopAura | S>C | `i32` bar, `i32` slot | `00a33120` |
| 36/10 | ClientRequestStartAbility | C>S | bar, slot, mode, id or 4 x `f32` | `00a35150`, writer `00a342e0` |
| 36/11 | MeleeRefresh | S>C | `i32` | `00a31c60` (reader `00a31430`) |
| 36/12 | RequestAbilityDefinition | C>S | `i32` ability id | `00a34d10` via `00a34410` |
| 36/13 | AbilityDefinition | S>C | definition record | `00a34380`, body `00a32930` |
| 36/14 | DetonateProjectile | S>C | `u64` id pair, 3 x 32-bit (animation, effect, float) | `00a31b80`, reader `00a2fe20` |
| 36/15 | PulseLocationTargeting | S>C | `bool`, `f32` | `00a31db0`, reader `00a30170` |
| 36/16 | ReceivePulseLocation | C>S | vector | `00a34720` |
| 36/17 | ExecuteClientLua | S>C | counted string, 3 x 32-bit | `00a31d40`, reader `00a30060` |
| 36/18 | CastInterrupt | S>C | `u64` id pair, `i32` | `00a31b10`, reader `00a2fdb0` |
| 35/5 | UpdateHitpoints | S>C | `u64`, 3 x `i32` | `008d6f10` |
| 35/8 | Animation | S>C | `u64` actor, `i32` animation, `i32`, `bool` | accepted by the client in native runs (our writer) |
| 35/16 | PlayCompositeEffect | S>C | not decoded here | case 0x10 of `0092f460`, ctor `008c4b30` |
| 35/20 | UpdateCharacterState | S>C | `u64`, `i32` flags | `0092f460` |
| 35/35 | HitPointModification | S>C | 2 x `u64`, `bool`, 3 x `i32`, `bool` | `008d6c50`, `BaseClient` slot 65 `008d3020` |
| 35/36 | TriggerEffectPackage | S>C | not decoded here | case 0x24, ctor `008c4c40` |
| 35/41 | AddEffectTagCompositeEffect | S>C | not decoded here | case 0x29, ctor `008c4d50` |
| 35/42, 35/43 | Remove / enable tag effect | S>C | 35/43 body not read | `0092f460` |
| 35/62 | LaunchProjectile | S>C | ProjectileParameters | ctor `008d0b10` |
| 38/1 | Hitpoints | S>C | `i32` current, `i32` maximum | `009392c0`, `0089d2b0` |
| 38/13 | Mana (energy) | S>C | `i32`, `i32`, `u8` | `009392c0`, `0089d360` |
| 38/14, 38/18 | Experience, rank | S>C | profile experience; rank | `009392c0` |
| 38/15, 38/22, 38/24 | Add ability set, add ability, update profile abilities | S>C | not decoded here | `009392c0` |
| 38/16, 38/17 | Add, remove effect tag | S>C | tag id (+ record) | `009392c0`, `00a19e70`, `00a15440` |
| 38/25 | UpdateActionBarSlot | S>C | bar id, slot, 51-byte record | `009392c0` case 0x19 |
| 42/6 | ItemActionBarAssign | C>S | `i32` slot, `i32` item instance (0 clears) | `00c14720` (vault 048) |
| 26/9, 26/10 | Interaction menu, InteractionSelect | S>C, C>S | menu entries; `u64` guid, `i32` entry (103 attack) | `00a36f80`, `00a382b0` |
| 41/109 | ParticipantRequestExit | C>S | header 0, instance | chain from `00c172c0` |
| 41/122 | ParticipantResume | C>S | bool for paid revive | `0093be30` (paid), `0093bd80` (free) |
| 41/125, 41/126 | ShowRespawnWindow, Hide | S>C | two `i32`; none | dispatcher `0090d830`, display `00904300` |
| 41/132, 41/133 | OverworldCombat, combat indicator | S>C | flag | `00aa36c0` cases, setters `009058d0`, `00905a50` |
| 39/18, 39/47, 39/19 | GameOver, score rows, StateRemove | S>C | results lifecycle | `009ad480`, `009bb030`, `009bf890` |
| 39/62 | MiniGame boss row | S>C | `u32` 1 update or 2 delete | `00c48c90` |
| 50/x | Reward bundle | S>C | see [[Rewards and Collections]] | |

Notes. (1) The client never builds 32/4, 32/5, 32/6, 32/7 or 32/9; they are receive-only. 32/2 and 32/8 are not handled by the dispatcher. (2) 36/16 is sent from `00a35150` when no slot is selected and a pulse is armed, instead of 36/10. (3) The accepted native fights send their attack as `26/10` with entry 103, not as 32/1, 32/3 or 36/10 (vault [[054 Melee attacks]]; four 36/10 requests in the same native run produced no processed attack). (4) 36/3 has a received `float` cast time that is `NaN`-checked at `+0x2c`. (5) Direction of 36/6, 36/8, 36/9, 36/11, 36/15, 36/17 and 36/18 was corrected in the shard review: all are server to client because the dispatcher builds them.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Own all combat content.** The client has no definitions. Keep per job and per weapon tier: the ability set (ids, names, descriptions, icons, tints, selectors, energy cost, cooldown, cast time, aura time, area limit), the effect list of each ability, the damage and healing rules, the status effects, and per creature: health, damage, speed, aggro and attack range. Our own job pages and the wiki give names, levels and rank text; the video gives damage-number magnitudes. The vault is clear that no retail damage formula was recovered, so every number is (R).
2. **Login and zone-in.** Send the character stats with the character data, 38/1 and 38/13 for health and energy, the effect tags (38/16), and the job toolbar (36/5 then, on request, 36/13). Send 38/25 slot updates for the item toolbar. Send the combat-area flags with 41/132 and 41/133 when the zone is an overworld combat area.
3. **Ability definitions.** Answer 36/12 with 36/13 for every id the client can ask about; the client queues an unknown id and sends 36/12 once for it (`00a34d10`), and nothing in the client retries. Every locale id in a definition must exist in the shipped table.
4. **Use request (36/10).** Treat the client as a request source only. Check the bar and slot against what the server last sent, the target mode and id, the player's level and job, the weapon, cost and energy, the cooldown, the status flags, range and line of sight, and that the target is hostile and alive. Apply the same status gate as the client (mask `0x1542c6`, plus silence for the job bar) so the two agree. A self cast carries the player's own id; the server resolves the real targets.
5. **Basic attack.** Accept all three entry points the client can use: `26/10` entry 103 (the proven one), 32/1 and 32/3 (target id only, sent when the client is within its `Weapon Range` stat times 0.8 of the target, 4.0 units by default), and 36/10 slot 0. Enforce reach and repeat rate on the server. Native acceptance used a 3 m melee reach and a 600 ms repeat gate (R).
6. **Reply with results.** For a landed hit send 32/7 with the target's maximum health and, for the client's first bar, health (see Open questions about the `+0x34` field), then 35/5 with the new health so the bar is correct, then the swing animation (35/8, group 1099 for the job wield types) and, on death, a removal. Send 32/5 for a miss and 32/6 for a dodge; send 32/4 only if a use for it is found. For damage to the player, send 32/7 for the number and 38/1 for the HUD bar.
7. **Healing and damage over time.** Use 35/35 with the flag set and the local player in one of the id pairs; a positive delta prints `+N`, a negative prints `-N`. Send 38/1 afterwards. This is the only visible path for heals and poison ticks (client).
8. **Energy.** The client refuses locally only when current energy (from 38/13) is below the slot cost, so the server must keep energy authoritative and send 38/13 after every cost, regeneration tick and potion.
9. **Casts, auras and projectiles.** For an ability with a cast time send 36/3 (cast time float, animation id, composite effect id) before the result and 36/18 to interrupt. For auras send 36/9 to stop. For ranged or area abilities use 35/62 or 36/4 for the launch, 36/14 for the landing, and 36/15 then wait for 36/16 for ground-targeted pulses. Drive any server-triggered client behaviour through 36/17 only with Lua function names the shipped scripts define.
10. **Cooldowns.** Send the slot's total refresh time in 38/25 or in the 36/4 timer path; use 36/11 as a lockout; remember the client also starts its own 500-unit timers on other slots and a Weapon Speed timer on the used slot after it sees a result frame addressed to the local player. The server enforces the real cooldown.
11. **Statuses.** Compute stun, sleep, silence, root, knockback, freeze and fear on the server and send the resulting flag dword in 35/20, with the matching 35/16 or 35/36 effect. Rooted, Bound, Confused and Berserk do not stop the client casting, so the server must refuse the cast if the rule says so. Immunities are server logic (client text only). Clear flags when they expire; no client-side expiry of a status flag was traced, so the server must send the clearing 35/20.
12. **Buffs and effect tags.** Send 38/16 with a stable tag id, icon, name and duration; send 38/17 at expiry or removal; send 35/41 and 35/42 for the attached composite effects. Reconnect and disconnect rules are (R): the vault module refreshes rather than stacks a repeated tag from the same source, which is not proven original.
13. **Knockout and revive.** On zero health set status bit 7 (35/20), send 38/1, then in an encounter send 41/125 with the timer in milliseconds and the revive cost, and in the overworld send 41/132 first so the window offers the paid choice. Handle 41/122 (paid: bool set) and the free path once each; restore health and energy (38/1, 38/13), clear bit 7, send 41/126, count the knockout against the instance limit (10 wandering, 15 dungeon; wiki) and share it across the group. Never let a replayed message re-open the window or charge twice (R).
14. **Targets and bosses.** Keep each creature's disposition (0 hostile) and flags correct in its spawn data, since Tab only picks hostile, living, forward-facing, line-of-sight creatures. Send 32/9 true at boss spawn and false on death, on loss, and whenever the fight ends with the boss alive.
15. **Results and exit.** On win or loss send 39/18, 39/47 (at least one score row, else the score movie closes itself) and 39/19; wait for `41/109` with header 0 and the instance before returning the player; ignore the 39/7 acknowledgement. Grant rewards once ([[Rewards and Collections]]).
16. **Progression.** Send job experience and rank changes (38/14, 38/18) and ability additions (38/15, 38/22, 38/24); accept ability purchases (36/7) and answer with 36/8. Level gates (1, 5, 10, 15, 20 for the job skills, wiki) are server content.
17. **Duels.** The client has duel invite and response packets in the encounter family; their bodies and rules are not recovered. Do not implement duels until they are.
18. **Never trust ids.** Validate every id in a request against what the player can target and own; ignore requests during loading and when the player is knocked out. The client does not de-duplicate results, so never send the same result twice.

### Evidence

- **Client.** 430 accepted functions across 42 shards (S0173 to S0749): 177 confirmed, 105 likely, 75 speculative, 73 unknown. Subsystems and their confirmed/likely/speculative/unknown counts: `game.effect` 126 (59/33/8/26), `game.target` 94 (30/15/35/14), `game.projectile` 46 (19/1/19/7), `game.ability` 44 (16/18/3/7), `ui.actionbar` 37 (22/12/1/2), `game.actionbar` 37 (12/13/5/7), `game.stats` 16 (4/6/2/4), `game.encounter` 15 (7/6/0/2), `world.targeting` 15 (8/1/2/4). Extra reads in this pass: the full dispatchers `00a35cc0` (36), `0092f460` (35), `009392c0` (38), `0090d830` and `00aa36c0` (41); `00a2ba40` (32/7 apply), `00a2e540`, `00a35150`, `00a309c0`, `00a30d20`, `00a32930`, `00a30360`, `00ae2350`, `00844d80`, `00845010` (S0203; unlabelled and absent from the table); a byte scan of the whole executable for callers of `00844d80` and `00845010` (none, by direct call, jump or dword pointer); the resource files and Lua named in Data; the preserved client locale (resolved through `reports/locale-id-to-text.json`).
- **Client Lua and locale.** `ScriptsBase.lua` supplies the data-source columns, `RespawnWindow`, `GroupHandler`, `Target`, `ScoreScreen` and `HandlerActionBar`. The locale supplies the cast messages, the knockout window, the tutorial and job text.
- **Video.** 48 batch files, 5,760 frame records. Counts of frames and videos by substring (not curated): "enough energy" 155 frames (42 videos), "Knockouts Remaining" 12 (12), "knocked out" 29 (13), "Recover in" 9 (7), "Take Me There" 147 (27), "boss" 125 (25), "you win" 70 (49), "prize wheel" 10 (5). A scan of standalone signed numbers in `other_text` finds 51 values in 24 videos: 50 negatives from -16 to -4981 (8 with `!!`) and one positive standalone value (+375); a green `+371` appears in another frame. `MISS!` appears in no frame. One frame shows an Archer at job level 14 with 3483/3483 health and 100/100 energy.
- **Wiki.** The Combat page (rev 2013-02-15) and the wiki pages of the six combat jobs (read through the job guides). The Combat page is the only source for the 10 s recovery, the 10 and 15 knockout limits, the tier gates and the status-effect descriptions. The Combat page is a 2013 revision, so its statements may predate later changes.
- **Vault.** Used for their client-derived parts: [[Native Combat Controls]], [[Native Combat Results Lifecycle]], [[Native Loss Boss Display]], [[Native Interaction Acquisition Range]], [[Gloam Native Visual Review]], [[Mac Native Visual Review]], and pieces [[048 Ability bar]], [[053 Aggro and leash]] (Tab conditions only), [[054 Melee attacks]], [[055 Ranged enemy attacks]] (constructor immediates only), [[056 Combat status effects]] (client constructors and Lua only), [[058 Death and revive]]. Not used: piece 031 (job catalogue), every decompiled-server excerpt, community serializer, community catalogue or captured retail row from the community server, and every wiki-derived number inside a reconstructed module. Pieces 051, 052, 057, 059 to 065 (creature, patrol, loot, dungeon and boss content) were not read; they belong to the dungeon and encounter content.
- **Native acceptance (our Rust server with the real client).** Verified on the wire and on screen: in the opening cave 20 attack requests (`26/10`, entry 103), 20 matching 32/7 results at 15 to 78 ms, request distances 1.97 to 2.43 units, swing 1099 on all 20, creature hits with animation 1001; the Gloam fight (run 183551: approach, a `-35` hit, a yellow `-100`, goal 1/1, the creature gone) and the Mac fight (run 193318: `-45`, yellow `-100`, "Defeat Mac 1/1", goal complete), both from a manual review of the retained recordings; the win and loss results lifecycle (39/18, 39/47, 39/19, then 41/109 only after Close), with a first run auto-closing because no score rows were sent and a repair that sends rows (a modal acceptance run was still pending when the vault note was written); boss-bar cleanup with 32/9 false. **Reconstructed (R), not original:** the 3.0 unit player melee reach (a ranged reach of 30 for profile 35 and 20 for profile 12), the 2.5 unit creature contact, the 600 ms repeat gate, flat per-profile damage of 70 to 110 (doubled for non-basic slots), creature health and damage, player health 1000, pursuit speed and the knockout flow. **Never exercised against the real client:** the action-bar `36/10` path producing a result, 36/3 casts, 36/13 definitions, 35/20 status flags, effect tags, healing, the 41/125 window as a captured frame, duels, and any multi-target ability.
- **What the repo's native runs do not prove.** They prove the client accepts our messages, not that the original server sent them or in that order.

### Open questions and contradictions

- **No original server code.** Every server rule above is inferred from the client. The vault's native runs prove client acceptance only.
- **Damage and numbers.** No formula, no rank-to-damage table and no creature table was recovered. Stat names suggest a pipeline (attack, to-hit, avoidance, critical chance and multiplier, defense, reduction, absorb, addition, multiplier) but the order and units are open. Rank text for Zap is 83 to 125 while the wiki's wand pages give Zap 279 (level 1 wand) to 2,609 (level 15 wand); a weapon multiplier is implied but unrecovered. Video shows `-16` to `-4981`.
- **`+0x34` in 32/7.** The apply function subtracts the damage from the client's stored health, or from `+0x34` when the stored health is zero (pseudocode of `00a2ba40`), which reads as pre-damage health; our server writes post-damage health there and follows with 35/5, which corrects the bar. Whether the original sent pre- or post-damage health, and what `+0x30` is, is unknown.
- **What 32/4 adds.** It starts the attacker's cooldown feedback and a motion event but builds no number or bar. Whether the original sent it with, instead of or before 32/7 is unknown.
- **Colours.** `008bb0b0` picks red when its flag is clear and "a computed colour" otherwise; the native-run reviews record `-35` and `-45` (colour not noted) and a yellow `-100`, which fits a player-to-creature hit (inferred from the number matching our reconstructed damage of 100 for those jobs). The rule for the other colours, and for the `!!` form, is not recovered.
- **Who calls `ShowDmgNum` and `ShowEffect`.** Nothing in the executable calls `00844d80` or `00845010` directly (byte scan). Their `%llu` 64-bit amount, string and int arguments cannot come from any 32-bit combat frame above, so a feature we have not found feeds them (candidates: coin or star popups, or a scripted call). The number path we trace is `PlayerHitpointDeltaEvent` to the actor's text slot.
- **Selectors and target modes.** Only self (selector 1 with 2 or 4) and the existence of ground targeting are decoded. What the other selector values mean, how the area ring radius is chosen, and what `MaxAoeTargets` counts are open.
- **36/11 MeleeRefresh.** Pseudocode stores a 64-bit stamp, `now + value`, in the processor, and `00a2e540` blocks sending while the stamp is less than 1,000 ms old or still in the future. The time base (`host+0xb8` is called an id pair in S0292 but is used as a time here) and the unit of the value are unproven.
- **Stat delivery.** How stat values reach the client (the character data block or an update) was not traced in this pass; only the table and readers were.
- **Cooldown units.** Weapon Speed default 1000 and the 500-unit timers are assumed to be milliseconds. The two refresh-related slot fields and the meaning of "cast age" are unresolved.
- **36/17.** The Lua function it calls and its three arguments come from the server; the set of valid names is not listed anywhere we read.
- **Statuses.** Bits 15 and 18 have no name and no message. The wiki's Root is bit 5 in our reading (`CharacterStatusRooted`), and the client has no Poison bit, so poison is probably carried by 35/35 ticks plus an effect tag.
- **Knockout limits and counters.** The client has the string "@1 Knockouts Remaining" but the packet that fills it was not traced. The 10 and 15 limits are wiki. The paid-revive price (1,000 coins, wiki, from the vault) is not in the client; the free path's relocation message is unrecovered.
- **Duels and PvP.** A duel invitation and response exist in the encounter family; nothing else is known.
- **Potions and power-ups.** The wiki says key 3 uses a power-up and the potion belt has four slots; the client tutorial says the energy potion is on the 6 key. The item bar's key mapping is only decoded for keys 1 to 9 as action ids 19 to 27.
- **Exit door.** The wiki says standing near the door for a few seconds opens the score screen (added December 2009); no client code for that was found.
- **Table scope.** 28 rows are not combat (trading-card engine and store logging, see How the client implements it); 8 more `world.targeting` rows are freeform actor placement handlers and 5 `game.encounter` rows are matchmaking queue code. `game.target` and `game.projectile` are mostly speculative (35 and 19) because they come from serialization templates and projectile bodies read by pattern.
- **Speculative labels worth rechecking.** `00a36a70` (select slot) and `00a15440` (tag removal) are unknown in the map though their roles are read above; `00a2d820` was downgraded to speculative.
- **Identical bodies.** `00a31430` and `00a31470`, and `00a32350`, `00a323a0`, `00a32440`, have identical bodies; the reason for the copies is unknown.
- **Wiki versus client, Brawler Resilience.** The wiki says health and energy after being hit; the client text says health only.
- **Warrior gate.** The Warrior ability gates are enforced by NPC dialog ("until you learn your next ability", client locale), not by any combat code we read; how the grant is delivered is not traced (see [[Quests and Objectives]]).

### Key functions

| Address | Role |
|---|---|
| `00b02880`, `008a7eb0`, `008a7e40` | Input dispatcher and action-bar slot dispatch (modes 1 to 4) |
| `009cf430`, `00a36a70`, `00a35600`, `00a30870` | UI gate with refusal reasons, select slot, target-mode dispatch, status gate |
| `00a35150`, `00a35000`, `00a342e0`, `00a34720` | Send start ability, tunneled write, request body writer, pulse location |
| `00a35cc0` | Family 36 receive dispatcher (cases 1, 3 to 6, 8, 9, 11, 13 to 15, 17, 18) |
| `00a31aa0`, `00a33760`, `00a32fe0`, `00a34380`, `00a32930`, `00a30360` | Start casting, launch and land, set definition, ability definition and its body and effect readers |
| `00a309c0`, `00a30d20`, `00a34e30`, `00c20420` | Named fields and tooltip rows |
| `00a34d10`, `00a34c80` | Definition lookup and request, purchase send |
| `00a2e540`, `00a2e620`, `00a2e6b0` | Lockout test, reset, classify target |
| `00a2ef20`, `00a2f030`, `00a2f6e0`, `00a359a0`, `00a35a40`, `00a35920` | Click handlers and their dispatch |
| `00a2c370`, `00a2bff0`, `00a2b360`, `00a2b580`, `00a2ba40`, `00a2b910` | Combat family receive and its handlers (32/4, 5, 6, 7, 9) |
| `00a2c7d0`, `00a2c880`, `008c6470`, `00a2ac20`, `008c6c10` | Attack senders, approach and attack loop, Tab selector, store target |
| `0092f460` | Family 35 receive dispatcher (hitpoints, state flags, composite effects, tags) |
| `009392c0` | Family 38 handler (health, energy, experience, ability sets, tags, slot refresh) |
| `00ae2350`, `008b81e0`, `008bb0b0` | Floating hitpoint number event and text slot |
| `00844d80`, `00845010`, `009d2170` | `ShowDmgNum`, `ShowEffect`, scene-text movie tick |
| `00985200`, `009852a0` | Boss registry add and remove |
| `0090d830`, `00904300`, `00aa36c0`, `009058d0`, `00905a50` | Encounter dispatchers, respawn window, overworld-combat flags |
| `0093be30`, `0093bd80` | Resume packets (paid, free revive) |
| `009ad480`, `009bb030`, `009bf890`, `009b7170` | Game over, score rows, state remove, end dialog |
| `00a16d50`, `00c454b0`, `00c45510`, `0101cd80`, `00c45450`, `0101ce90` | Action-bar table, bars, slot and timer |
| `00a12aa0`, `00a12ae0`, `00a12b20` | Stat readers |
| `00a19e70`, `00a14750`, `00a15440` | Effect-tag insert, remove, receive removal |
| `00b83d20`, `00b84e90`, `00b84160`, `00bf34e0` | Projectile manager, update, tick, projectile base |
| `0101c300`, `0101c850`, `0101c930` | Target factory and serialization |
<!-- NARRATIVE END -->

## Mapped functions (generated)

430 accepted functions: 177 confirmed, 105 likely, 75 speculative, 73 unknown. Network-linked (`wire`): 10.

Subsystems: `game.effect` (126), `game.target` (94), `game.projectile` (46), `game.ability` (44), `ui.actionbar` (37), `game.actionbar` (37), `game.stats` (16), `game.encounter` (15), `world.targeting` (15)

### `game.ability`

**AbilityActorClickHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2ef20` | AbilityActorClickHandler::AbilityActorClickHandler | confirmed | Runs the base click-handler constructor FUN_008c49a0, stores AbilityActorClickHandler::vftable at +0 and the argument at +0xc. |
| `00a359a0` | AbilityActorClickHandler::DispatchEventToProcessor 🌐 | confirmed | Virtual slot [3] of AbilityActorClickHandler::vftable (binary vtable read). Event code is the stack argument at +0x18 (in_stack_00000018); param_3 is a pointer… |
| `00a2d950` | AbilityActorClickHandler::MatchKind | likely | Returns 1 when the first argument is 8 and the second is 0, or when the first argument is 0x400, 0x800 or 0x1000; otherwise returns 0. |
| `00a2efd0` | AbilityActorClickHandler::ScalarDeletingDtor | likely | Stores TargetProcessorEventHandler::vftable (the only vtable step of this destructor chain) and, when bit 0 of the flag argument is set, releases the object wi… |

**AbilityDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01064290` | AbilityDefinition::AbilityDefinition | confirmed | Thiscall with a text pointer (param 2) and a delimiter (param 3). Stores AbilityDefinition::vftable at +0, zeroes about thirty words in the +0x20..+0xf4 range,… |
| `01064600` | AbilityDefinition::AbilityDefinition | confirmed | Fastcall with the object in ECX (no stack arguments). Stores AbilityDefinition::vftable at +0, sets the byte at +0xc to 1 and the byte at +0xd to 0, zeroes the… |
| `010647c0` | AbilityDefinition::~AbilityDefinition | confirmed | Fastcall with the object in ECX (cookie-framed). Stores AbilityDefinition::vftable at +0, then while the head at +0xb4 is non-null: calls 00a2dbb0 on the node … |
| `01064890` | AbilityDefinition_ScalarDeletingDtor | confirmed | Thiscall with a flag byte (ret 4). Calls the AbilityDefinition destructor body 010647c0 on this, then when bit 0 of the flag is set frees this through 00401c00… |

**AbilityPlayerClickHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2f030` | AbilityPlayerClickHandler::AbilityPlayerClickHandler | confirmed | Runs the base click-handler constructor FUN_008b3470, stores AbilityPlayerClickHandler::vftable, writes the argument at +0xc and zeroes the byte at +0x10. |
| `00a2f090` | AbilityPlayerClickHandler::MatchKind | confirmed | Returns true when the argument has any of the bits 0x800, 0x400 or 0x8 set (mask 0xc08), and false otherwise. |
| `00a35a40` | AbilityPlayerClickHandler::DispatchEventToProcessor 🌐 | confirmed | Virtual slot [3] of AbilityPlayerClickHandler::vftable (binary vtable read). Event code is the seventh argument (raw bytes: mov eax,[esp+0x18] at entry). Code … |
| `00a2f100` | AbilityPlayerClickHandler::ScalarDeletingDtor | likely | Stores TargetProcessorEventHandler::vftable (the only vtable step of this destructor chain) and, when bit 0 of the flag argument is set, releases the object wi… |

**AbilityProcessor** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a324e0` | AbilityProcessor::AbilityProcessor | confirmed | Constructor. Stores AbilityProcessor::vftable at +0, the owner pointer at +4 and default fields (dwords at +8..+0x20 set to 0 or -1, bytes cleared at +0x24, +0… |
| `00a355f0` | AbilityProcessor::SendIfFlagAt24 🌐 | confirmed | Virtual slot [0] of AbilityProcessor::vftable (read from the binary). Tests the byte at this+0x24; when it is non-zero it calls 00a35150 with param_2=1. No dto… |
| `00a35cc0` | AbilityProcessor::HandleAbilityPacketBySubOpcode 🌐 | confirmed | Virtual slot [2] of AbilityProcessor::vftable (binary vtable read). Entry: prologue with aligned stack; param_2/param_3 are a buffer pointer and its length. Ca… |
| `00a35150` | AbilityProcessor::SendStartAbilityRequest 🌐 | likely | Gated by 00a2e540, 00a30870 and the bytes at this+0x61/+0x62 and param_2. With this+0xc negative it only runs the +0x25 branch (00a2d4f0 ctor, then 00a2d550 dt… |
| `00a35600` | AbilityProcessor::DispatchTargetModeAndSend 🌐 | likely | Returns a byte (uVar6). Reads the owner object at this+4 and its field +200; takes the 1870 pointer and compares its first two dwords with DAT_01be6870/DAT_01b… |
| `00a31860` | — | unknown | Destructor body. Restores AbilityProcessor::vftable at +0, calls 00a2e620, releases the owned objects at +0x74, +0x78 and +0x7c (each through 00acb430 when the… |
| `00a33760` | — | unknown | Virtual slot [3] of AbilityProcessor::vftable (binary read). Builds an AbilityPacketLaunchAndLand working object through 00a2f450, reads the payload through 00… |
| `00a36a70` | — 🌐 | unknown | Thiscall; param_2 is a slot id (-1 means none) and param_3 is a bar type. If the current slot (this+0xc) or the pending flag (this+0x24) is set, it resets stat… |

**AbilitySet** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fa5c0` | AbilitySet::SerializeForClient | likely | thiscall with one stack argument (reader param_2: cursor +8, end +0xc, error flag +0x10). Reads one dword count from the reader (zero and error flag on overflo… |

**AbilityTerrainClickHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d910` | AbilityTerrainClickHandler::MatchKind | confirmed | Returns 1 when the first argument is 8 and the second is 0, or when the first argument is 0x400, 0x800 or 0x1000; otherwise returns 0. |
| `00a2f6e0` | AbilityTerrainClickHandler::AbilityTerrainClickHandler | confirmed | Runs the base constructor FUN_008b3590 (which stores TargetProcessorEventHandler::vftable, zeroes +4/+8 and then stores TargetProcessorTileEventHandler::vftabl… |
| `00a35920` | AbilityTerrainClickHandler::DispatchEventToProcessor 🌐 | confirmed | Virtual slot [3] of AbilityTerrainClickHandler::vftable (binary vtable read). Takes an event code in its sixth argument (param_6) and a pointer to four dwords … |
| `00a2f790` | AbilityTerrainClickHandler::ScalarDeletingDtor | likely | Stores TargetProcessorEventHandler::vftable (the only vtable step of this destructor chain) and, when bit 0 of the flag argument is set, releases the object wi… |

**TargetProcessorEventHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2ef80` | TargetProcessorEventHandler::TargetProcessorEventHandler | confirmed | Stores TargetProcessorEventHandler::vftable at +0 and nothing else. |
| `00a2f0b0` | TargetProcessorEventHandler::TargetProcessorEventHandler | likely | Stores TargetProcessorEventHandler::vftable at +0 and nothing else. |
| `00a2f740` | TargetProcessorEventHandler::TargetProcessorEventHandler | likely | Stores TargetProcessorEventHandler::vftable at +0 and nothing else. |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a2d790` | ClearStateFlagsAndUnlinkHandles | likely | When *(*(this+4)+0x1a8) is non-zero it unlinks the two container entries at this+0x74 and this+0x7c through FUN_00acb430, clears that sub-object's byte at +0x6… |
| `00a2da50` | ConstructAbilityDefinitionWithKey | likely | Constructs the embedded definition member with FUN_01064600 and copies the first dword of the second argument into this+0x100 (the map key field used by the 10… |
| `00a2f160` | CallResetIfFlagSet | likely | Calls the state reset 00a2e620 when the flag byte at this+0x24 is non-zero and otherwise returns without doing anything. |
| `00a30870` | AbilityValidatePlayerStatus | likely | Fastcall predicate: reads the flags dword at owner->+200->+0x6f8 (owner = this+4). The mask is 0x1542c6 when this+8 == 2, otherwise 0x1542ce. When flags AND ma… |
| `00a30d20` | AbilityEffectNamedFields | likely | Builds a named-field table for one ability effect record: assigns field names including ChanceToOccur, DurationSeconds, MaxCharges, Param1 with Param1TunedLow … |
| `00a34c80` | SendPurchaseAbility | likely | Constructs an AbilityPacketPurchaseAbility through 00a2ce70 and then calls 00a345f0 with a 12-byte local and flag 1 to send the tunneled purchase payload. |
| `00a34d10` | AbilityDefinitionLookupOrRequest | likely | Looks up an ability definition node by id (param_2) in the 100-bucket list at this+0x94 (bucket id mod 100, key at +0x100, next at +0x104). On a miss it scans … |
| `00a34e00` | AbilityDefinitionNamedFieldsByKey | likely | Looks up the ability definition for the id argument through 00a34d10. When found it returns the result of 00a309c0 on that node with the second argument, other… |
| `00a34e30` | AbilityEffectRowsForItem | likely | Looks up the ability definition through 00a34d10, then walks its effect chain at +0xb4 (next pointer at +200). For each effect it calls 00a30d20 to build the n… |
| `00a35810` | StoreVector16AndSend | speculative | If the byte-owner object at this+4 has a non-null +0x1a8 pointer: calls 00acb430 with this+0x74 (accepted label HashListRemoveElement_7c), sets byte +0x61 of t… |
| `00a35860` | StoreVector16FromPtrAndSend | speculative | If the owner object at this+4 has a non-null +0x1a8 pointer: calls 00acb430 with this+0x74 (accepted label HashListRemoveElement_7c), copies the four dwords at… |
| `00a358b0` | StoreSlot10PairAndPositionThenSend | speculative | Thiscall; param_2 is an object pointer. If this->+4->+0x1a8 is non-null: calls vtable slot +0x10 of the object at param_2+0x54 with an out buffer (two dwords),… |
| `00a2d820` | — | unknown | Runs 00a2d790, sets this+0x61, and when *(*(this+4)+0x1a8) is non-zero re-attaches the two entries at +0x74/+0x7c with FUN_00acb3e0, clears the sub-object's +0… |
| `00a2e620` | — | unknown | Runs 00a2d790, clears the byte at this+0x24, copies the globals DAT_01b4ce60/DAT_01b4ce64 into +8/+0xc, copies the host's 64-bit id pair from *(this+4)+0xb8/+0… |
| `00a2e6b0` | — | unknown | Runs 00a2d790 and sets this+0x60, routes by the mode at +8 (2 to FUN_00a128e0(this+8) when the host flag at +0x3e0 is set, 3 to FUN_009aaed0(this+0xc), anythin… |
| `00a309c0` | — | unknown | Builds the named-field table of an ability definition node (arg2). When the list pointer at +0x74 is non-zero it assigns the owner listener (this+4 -> +200), M… |

### `game.actionbar`

**ActionBarSlot** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101cd70` | ActionBarSlot::~ActionBarSlot | confirmed | Destructor body: stores ActionBarSlot::vftable at *this and returns. Called from seven EH unwind funclets and from the scalar deleting destructor 008bf9f0 (acc… |
| `0101cd80` | ActionBarSlot::ActionBarSlot | confirmed | Fastcall constructor, ECX = this: zeroes the fields at +0x0c..+0x38 and +0x24 region, stores ActionBarSlot::vftable at *this, sets the byte at +8 to 1 and the … |
| `0101ce00` | ActionBarSlot::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of ActionBarSlot): stores ActionBarSlot::vftable at *this, then calls _free(this) when bit 0 of the byte argument is set… |

**BaseActionBar** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c453d0` | BaseActionBar_Destruct | confirmed | Base destructor: reinstalls BaseActionBar::vftable and destroys the embedded ActionBarSlot map at this+8 by calling the map destructor 00c44770 (the destructor… |
| `00c454b0` | BaseActionBar_Construct | confirmed | Constructor: installs BaseActionBar::vftable, stores the owner argument at +4 and constructs the embedded slot map at +8 through 00c45310. |
| `00c45490` | BaseActionBar_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the base destructor 00c453d0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c445a0` | — | unknown | Virtual slot 1 setter: finds the slot node for id param_2 (bucket id & 7 at this+0x1c, key at +0x60); when no node exists and the create flag is set it allocat… |

**ClientActionBar** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45510` | ClientActionBar_Construct | confirmed | Constructor: runs the BaseActionBar constructor with the owner argument, installs ClientActionBar::vftable, zeroes the extra fields at +0x398..+0x3b4, stores t… |
| `00c455f0` | ClientActionBar_Destruct | confirmed | Destructor: reinstalls ClientActionBar::vftable, releases the data source pointer at +0x3b0 (virtual slot 10 result handed to FUN_0081ad80 while the engine glo… |
| `00c45680` | ClientActionBar_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the ClientActionBar destructor 00c455f0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c44600` | — | unknown | Client override of the slot setter: keeps a copy of the previous slot payload (FUN_0101cd80), applies the base set (00c445a0), then compares old and new fields… |

**(no class)** (26)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a7e40` | RunActionBarMode | confirmed | Mode dispatch on param_3: mode 2 calls FUN_00a12840 only when this+0x3e0 is non-null; mode 3 calls FUN_009aaeb0(param_2); mode 4 calls FUN_009b6590(param_2); a… |
| `008a7eb0` | DispatchActionBarSlot | confirmed | Mode selection: if FUN_008a5850 returns non-zero it dispatches mode 4; otherwise if FUN_009ad3c0 returns non-zero and either FUN_009ad0b0(0,0) is 0 or its byte… |
| `009cf430` | DispatchActionBarCastOrFailureReason | confirmed | If the indexed action is not active it clears or routes its state; otherwise checks the player's status bits and mana, then emits the matching NoCast Lua reaso… |
| `00a16d50` | Client_GetOrCreateActionBar | confirmed | Looks up the action bar object for type param_2 in the client's 5-bucket table at this+0x35434 (chain +0x398, key +0x39c); when absent it allocates 0x3b4 bytes… |
| `00c44160` | ClientActionBar_ActivateSlot | confirmed | Activation gate for an action-bar slot: looks the slot up by id (bucket id & 7 at this+0x1c) and returns when byte +8 is set; otherwise it tests the player no-… |
| `008ab6d0` | ActionBarSlotInitAndStore60 | likely | Saves the incoming ECX in ESI, then calls FUN_0101cd80 with ECX still holding this; the callee installs ActionBarSlot::vftable and initializes its fields. The … |
| `00c43a90` | FieldGetter_0x3ac | likely | 7-byte field getter (8B 81 AC 03 00 00 C3): returns the dword at this+0x3ac, the action bar's currently selected slot id (0xffffffff when nothing is selected). |
| `00c43c70` | SlotCheck_ForwardToGameVtable | likely | Finds the ActionBarSlot with key param_2 (bucket param_2 & 7 at this+0x1c, key at node+0x60, next at node+0x64); when found it forwards the slot's +0x14 dword … |
| `00c43d20` | Player_CanCastStatusCheck | likely | Finds the slot by id; when the slot's byte at +8 is zero and the player object (*(this+0x3a8)) and its state pointer (*+0xc8) exist, returns true exactly when … |
| `00c43d80` | ClientActionBar_SelectSlot | likely | Verifies the requested slot id exists (bucket id & 7 at this+0x1c); then, when a previously selected id (this+0x3ac) is also present in the list, notifies the … |
| `00c43e00` | ClientActionBar_ClearSelection | likely | Reads the selected slot id at this+0x3ac; when that id is present in the slot list it notifies the UI with code 7, and in every path it resets the selected id … |
| `00c43fa0` | ActionBar_OnOwnerStateReset | likely | Iterates the slot node list at this+0x10 (next at +0x58) calling FUN_0101ce20 on every node without arguments, then tail-calls virtual slot 4 (offset 0x10) of … |
| `00c43fd0` | ActionBarSlot_IsUsable | likely | Finds the slot by id and returns true only when all three conditions hold: the player no-cast status check 00c43d20 passes, the requirement check 00c43cd0 pass… |
| `00c440d0` | ActionBar_NotifyAllSlots | likely | Iterates the slot node list at this+0x10 (next at +0x58) and calls the UI notifier FUN_010cb9a0 with each node's key at +0x60 and the caller-supplied notificat… |
| `00c44440` | ActionBar_NotifySlotsCode3_A | likely | Iterates the slot node list at this+0x10 (next at +0x58) and calls the UI notifier FUN_010cb9a0 with each node's key and the constant code 3; no other work. |
| `00c44470` | ActionBar_NotifySlotsCode3_B | likely | Byte-identical twin of 00c44440: iterates the slot node list at this+0x10 (next at +0x58) and calls FUN_010cb9a0(node key, 3) for every node. |
| `00c44070` | ActionBar_NotifySlotAvailability | speculative | For every slot node in the list at this+0x10: when the node key equals param_2 and its dword at +0x2c is greater than 0x32 it calls FUN_0101cfc0, otherwise it … |
| `0101ce20` | ResetRecordFields | speculative | Fastcall, ECX = this: sets the byte at +8 to 1, clears the dwords at +0x0c, +0x14..+0x24, +0x2c..+0x38 and the byte at +0x28, the byte at +0x50 to 0; calls 007… |
| `0101ce90` | RemainingAboveThreshold30 | speculative | Fastcall, ECX = this: when the pair at +0x40/+0x44 is not zero, computes the 64-bit difference of the pair at +0x48/+0x4c minus the pair at +0x40/+0x44, clamps… |
| `0101cee0` | SetValue30SnapshotPair48 | speculative | Thiscall, one stack argument: stores the argument at this+0x30, then copies the pair at +0x48/+0x4c to +0x40/+0x44 (ret 4). |
| `0101cfc0` | CopyDword2cToDword30SnapshotPair | speculative | Fastcall, ECX = this: copies the dword at +0x2c to +0x30, then copies the pair at +0x48/+0x4c to +0x40/+0x44. |
| `009d08f0` | — | unknown | For every slot through the stored maximum, checks status and mana; it disables or enables slots and expires timed flags when their deadlines have passed. |
| `00a326a0` | — | unknown | Checks that the handler named 'HandlerActionBar' resolves (00977740). If so, it stores into this+0x68/+0x6c a 64-bit deadline made from param_2 plus the owner'… |
| `00c43bf0` | — | unknown | Field-by-field copy of a 0x50-byte record into the object in ECX: copies the byte at +8 and the dwords at +0xc..+0x4c plus the byte at +0x50 from the source ar… |
| `00c43cd0` | — | unknown | Finds the slot by id and returns true when the slot's requirement dword at +0x24 is zero, or when it is not greater than the player level found as *(*(this+0x3… |
| `00c45450` | — | unknown | Virtual slot 2 shared by BaseActionBar and ClientActionBar: for every slot node in the list at this+0x10 (next at +0x58) it adds the 64-bit argument to the nod… |

### `game.effect`

**ActorProjectileEmitter** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00964ff0` | ActorProjectileEmitter::Constructor | confirmed | Calls FUN_00775f70 for the base, installs ActorProjectileEmitter's vtable, initializes its IString member, copies ProjectileParameters, initializes Target-arra… |
| `009650e0` | ActorProjectileEmitter::IsSlot0cClear | confirmed | Calls the object's virtual slot +0xc and returns true exactly when that call returns zero. |
| `00965100` | ActorProjectileEmitter::ScalarDeletingDestructor | confirmed | Calls FUN_010c0f60 for emitter cleanup, frees param_1 when flag bit 0 is set, and otherwise returns the pointer. |

**ClientEffectTag** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a13690` | ClientEffectTag_ScalarDeletingDtor | confirmed | Stores ClientEffectTag::vftable, calls FUN_0103da90 with this object in ECX (the EffectTag base destructor installs EffectTag::vftable), then calls _free on th… |
| `00a0fcd0` | ClientEffectTag_Ctor | likely | Constructs a ClientEffectTag: runs the EffectTag base constructor FUN_0103daa0, stores ClientEffectTag::vftable, then zeroes the tag payload fields at +0x68, t… |
| `00a0fd30` | ClientEffectTag_Dtor | likely | Destructor body: stores ClientEffectTag::vftable back into this and calls the EffectTag base destructor FUN_0103da90 (registered as an unwind funclet target fo… |
| `00a0fd80` | ClientEffectTag_ScalarDeletingDtor | likely | Scalar deleting destructor: restores ClientEffectTag::vftable, calls the base EffectTag destructor FUN_0103da90 and, when bit 0 of param_2 is set, frees the ob… |
| `00a10f50` | ClientEffectTag_Dtor2 | likely | Second destructor body for ClientEffectTag: restores ClientEffectTag::vftable and calls the EffectTag base destructor FUN_0103da90; no fields released. |
| `00a10fa0` | ClientEffectTagDerived_Ctor | likely | Constructs a ClientEffectTag-derived object: calls the ClientEffectTag constructor FUN_00a0fcd0 with this, then copies *param_2 into the field at this+0x80 and… |

**EffectDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010649c0` | EffectDefinition::~EffectDefinition | confirmed | Fastcall with the object in ECX (cookie-framed). Stores EffectDefinition::vftable at +0, then runs the IString<char> destructor body 00722bf0 on the embedded s… |
| `01064bb0` | EffectDefinition_ScalarDeletingDtor | confirmed | Thiscall with a flag byte (ret 4). Stores EffectDefinition::vftable at +0, runs the IString<char> destructor body 00722bf0 on the embedded string at this+0x38,… |
| `01064a50` | — | unknown | Fastcall with the object in ECX (no stack arguments, cookie-framed). Stores EffectDefinition::vftable at +0, sets the words at +4, +8 and +0xc to -1, +0x10 to … |
| `01064c20` | — | unknown | Virtual (vtable slot [1] of EffectDefinition), thiscall with a text pointer and a delimiter. Parses about fifty consecutive fields into the object in a fixed o… |

**EffectTag** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103da90` | EffectTag::DtorBody | confirmed | Destructor body: stores EffectTag::vftable into *this and returns (no fields released). |
| `0103daa0` | EffectTag::Ctor | confirmed | Constructor: stores EffectTag::vftable at *this, zeroes the dwords at +8, +0xc, +0x10, +0x34, +0x38, +0x3c, +0x40, +0x48 and +0x58, zeroes the bytes at +0x30 a… |
| `008bf990` | EffectTag::ScalarDeletingDestructor | likely | Calls 0x0103da90 with this in ECX and no stack arguments; the callee installs EffectTag::vftable and returns. Frees this when deleting-flag bit 0 is set. |

**PlayerCameraEffectEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095cac0` | PlayerCameraEffectEvent::PlayerCameraEffectEvent | confirmed | Constructs the base event, installs camera-effect vtables, stores the referenced object, and increments its reference count with the activation callback. |
| `0095cb70` | PlayerCameraEffectEvent::~PlayerCameraEffectEvent | confirmed | Restores camera-effect vtables, decrements the referenced object's count when positive, and invokes FUN_00957df0. |
| `0095cbf0` | PlayerCameraEffectEvent::ScalarDeletingDtor | confirmed | Calls FUN_0095cb70 and frees param_1 when the deleting flag bit is set. |
| `0095cbe0` | PlayerCameraEffectEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095cc10` | — | unknown | Forwards to FUN_0095cbf0 and returns. |
| `0095fb20` | — | unknown | Resolves the current player and effect object, obtains a resource value through FUN_007764d0, performs cleanup, and returns the value or zero. |

**PlayerEffectEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095c230` | PlayerEffectEvent::PlayerEffectEvent | confirmed | Calls the base initializer, installs PlayerEffectEvent vtables, initializes event identifiers and flags, copies a global four-word vector, and zeros trailing s… |
| `0095c370` | PlayerEffectEvent::Process | confirmed | Checks the current effect handle, optionally stops it, creates an effect through the virtual factory, initializes transform/state, updates the handle, and eval… |

**PlayerLightEmitterEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095c960` | PlayerLightEmitterEvent::PlayerLightEmitterEvent | confirmed | Constructs the base event, installs light-emitter event vtables, stores the referenced object, and increments its reference count with the zero-to-one callback. |
| `0095ca10` | PlayerLightEmitterEvent::~PlayerLightEmitterEvent | confirmed | Restores light-emitter vtables, decrements the referenced object's count when positive, and calls FUN_00957df0. |
| `0095ca90` | PlayerLightEmitterEvent::ScalarDeletingDtor | confirmed | Calls FUN_0095ca10 and frees the object when the deleting flag bit is set. |
| `0095ca80` | PlayerLightEmitterEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095cab0` | — | unknown | Forwards to FUN_0095ca90 and returns. |
| `0095fa30` | — | unknown | Resolves the current player and effect object, obtains a resource value through FUN_007763f0, performs cleanup, and returns the value or zero. |

**PlayerLightsaberTrailEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095cf90` | PlayerLightsaberTrailEvent::~PlayerLightsaberTrailEvent | confirmed | Restores lightsaber-trail vtables, decrements the referenced object's count when positive, and invokes FUN_00957df0. |
| `0095d010` | PlayerLightsaberTrailEvent::ScalarDeletingDtor | confirmed | Calls FUN_0095cf90 and frees the object when the deleting flag bit is set. |
| `0095d030` | PlayerLightsaberTrailEvent::ScalarDeletingDtorThunk | confirmed | Forwards to FUN_0095d010 and returns. |
| `0095cee0` | PlayerLightsaberTrailEvent::PlayerLightsaberTrailEvent | likely | Constructs the shared base event, installs lightsaber-trail vtables, stores the referenced object, and increments its reference count using the activation call… |
| `0095d000` | PlayerLightsaberTrailEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095fdd0` | — | unknown | Resolves the current player and effect object, obtains a property through FUN_00776740, performs cleanup, and returns the value or zero. |

**PlayerMeshEmitterEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095ce30` | PlayerMeshEmitterEvent::~PlayerMeshEmitterEvent | confirmed | Restores mesh-emitter vtables, decrements the referenced object's count if positive, and runs FUN_00957df0. |
| `0095ceb0` | PlayerMeshEmitterEvent::ScalarDeletingDtor | confirmed | Calls FUN_0095ce30 and conditionally frees the object when deleting flag bit 0 is set. |
| `0095ced0` | PlayerMeshEmitterEvent::ScalarDeletingDtorThunk | confirmed | Forwards to FUN_0095ceb0 and returns. |
| `0095cd80` | PlayerMeshEmitterEvent::PlayerMeshEmitterEvent | likely | Calls the PlayerEffectEvent base constructor, installs PlayerMeshEmitterEvent vtables at this and this+0x50, stores the input object pointer at this+0xa0, incr… |
| `0095cea0` | PlayerMeshEmitterEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095fcf0` | — | unknown | Resolves the current player and effect object, obtains a value through FUN_00776690, performs cleanup, and returns the value or zero. |

**PlayerModelMaterialEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095c8b0` | PlayerModelMaterialEvent::~PlayerModelMaterialEvent | confirmed | Restores the model-material vtables, decrements the referenced object's count if positive, and calls FUN_00957df0. |
| `0095c930` | PlayerModelMaterialEvent::ScalarDeletingDtor | confirmed | Calls FUN_0095c8b0 and frees param_1 when the deleting flag bit is set. |
| `0095c950` | PlayerModelMaterialEvent::ScalarDeletingDtorAdjustorThunk | confirmed | Subtracts 0x50 from ECX, then jumps to 0095c930. |
| `0095c800` | PlayerModelMaterialEvent::PlayerModelMaterialEvent | likely | Calls the PlayerEffectEvent base constructor, installs PlayerModelMaterialEvent vtables at this and this+0x50, stores the input object pointer at this+0xa0, in… |
| `0095c920` | PlayerModelMaterialEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095f950` | — | unknown | Resolves the current player and effect object, reads a value through FUN_00776310, performs cleanup, and returns the value or zero. |

**PlayerParticleEmitterEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095c530` | PlayerParticleEmitterEvent::PlayerParticleEmitterEvent | confirmed | Constructs the PlayerEffectEvent base, installs particle-emitter vtables, stores two constructor values, and increments the referenced object count with its ze… |
| `0095c5f0` | PlayerParticleEmitterEvent::~PlayerParticleEmitterEvent | confirmed | Restores the particle-emitter vtables, decrements the referenced object count if positive, then runs FUN_00957df0 cleanup. |
| `0095c670` | PlayerParticleEmitterEvent::ScalarDeletingDtor | confirmed | Calls the class destructor and frees the object with _free when the deleting flag bit is set; otherwise returns the object pointer. |
| `0095c660` | PlayerParticleEmitterEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095c690` | — | unknown | Forwards to FUN_0095c670 and returns; the target performs conditional object deallocation. |
| `0095f760` | — | unknown | Resolves the current player and effect object, selects an active handle when present, reads a resource value through FUN_00776f40, performs cleanup, and return… |

**PlayerProjectileEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00964e00` | PlayerProjectileEvent::Constructor | confirmed | Calls FUN_0095c230 for the base event, installs both PlayerProjectileEvent vtable pointers, stores the supplied pointer, copies ProjectileParameters, initializ… |
| `00964f20` | PlayerProjectileEvent::Destructor | confirmed | Restores both event vtables, decrements the referenced object's count when positive, destroys the Target array elements, frees its buffer, destroys ProjectileP… |
| `00964fd0` | PlayerProjectileEvent::ScalarDeletingDestructor | confirmed | Calls FUN_00964f20 for event cleanup, frees param_1 when flag bit 0 is set, and otherwise returns the original pointer. |
| `00965120` | PlayerProjectileEvent::CreateActorProjectileEmitter | confirmed | Obtains a context object through FUN_00ae2470, validates it with a virtual call, allocates an ActorProjectileEmitter from the related object's +0x360 value, in… |

**PlayerRumbleEffectEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095ccd0` | PlayerRumbleEffectEvent::~PlayerRumbleEffectEvent | confirmed | Restores rumble-effect vtables, decrements the referenced object's count when positive, then invokes FUN_00957df0. |
| `0095cd50` | PlayerRumbleEffectEvent::ScalarDeletingDtor | confirmed | Calls FUN_0095ccd0 and conditionally frees param_1 when deleting flag bit 0 is set. |
| `0095cd70` | PlayerRumbleEffectEvent::ScalarDeletingDtorThunk | confirmed | Forwards to FUN_0095cd50 and returns. |
| `0095cc20` | PlayerRumbleEffectEvent::PlayerRumbleEffectEvent | likely | Constructs the base event, installs rumble-effect vtables, stores a referenced object, and increments its reference count with its activation callback. |
| `0095cd40` | PlayerRumbleEffectEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095fc10` | — | unknown | Resolves the current player and effect object, reads a value through FUN_007765b0, performs cleanup, and returns the value or zero. |

**PlayerSoundEmitterEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095c6a0` | PlayerSoundEmitterEvent::PlayerSoundEmitterEvent | confirmed | Constructs the PlayerEffectEvent base, installs sound-emitter vtables, stores the referenced object, and increments its reference count with the zero-to-one ca… |
| `0095c750` | PlayerSoundEmitterEvent::~PlayerSoundEmitterEvent | confirmed | Restores the sound-emitter vtables, decrements its referenced object's count when positive, then calls FUN_00957df0. |
| `0095c7d0` | PlayerSoundEmitterEvent::ScalarDeletingDtor | confirmed | Runs FUN_0095c750, then frees param_1 with _free when the deleting flag bit is set. |
| `0095c7f0` | PlayerSoundEmitterEvent::ScalarDeletingDtorAdjustorThunk | confirmed | Subtracts 0x50 from ECX, then jumps to 0095c7d0. |
| `0095c7c0` | PlayerSoundEmitterEvent::GetWordAt50 | likely | Returns the four-byte value stored at this+0x50. |
| `0095f860` | — | unknown | Resolves the current player and effect object, obtains a value through FUN_00776160, performs cleanup, and returns the value or zero. |

**SoeUtil::HashListMap<int,ClientEffectTag,20,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a162b0` | HashListMap_ClientEffectTag_20_minus1_Dtor | confirmed | Destructor body: installs the class vftable and loops while the list head at this+8 is non-null, calling the node removal 00a14750 for each ClientEffectTag unt… |
| `00a16410` | HashListMap_ClientEffectTag_20_minus1_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): installs the class vftable, drains the per-client effect-tag map by calling 00a14750 on each head node, then frees … |
| `00a166c0` | HashListMap_ClientEffectTag_InsertAtTail | likely | Appends a ClientEffectTag node to the 20-bucket map: sets node+0x7c to the current tail (this+0xc) and node+0x78 to zero, updates head at this+8 when the list … |
| `00a14750` | — | unknown | Unlinks one ClientEffectTag node from the bucket chain and ordered list, updates head/tail/count, writes ClientEffectTag::vftable into the node, calls FUN_0103… |

**SoeUtil::HashListMap<int,ClientEffectTag,20,8>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a170f0` | HashListMap_ClientEffectTag_20_8_Dtor | confirmed | Destructor of the derived effect-tag map: installs the derived vftable, drains the tag nodes with 00a14750, frees the tree at +0x64 through 00a14d10 and FUN_00… |
| `00a17b70` | HashListMap_ClientEffectTag_20_8_AllocatorSlot1 | confirmed | Eight-byte adjustor thunk (bytes 83c164 e918ffffff): adds 0x64 to ecx, retargeting the call to the pool sub-object at this+0x64, and tail-jumps to the pool blo… |
| `00a17b80` | HashListMap_ClientEffectTag_20_8_AllocatorSlot2 | confirmed | Eight-byte adjustor thunk, the twin of 00a17b70: adds 0x64 to ecx and tail-jumps to the pool block free routine 00a17b00. |
| `00a187a0` | HashListMap_ClientEffectTag_20_8_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls the derived destructor 00a170f0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00a18840` | HashListMap_ClientEffectTag_20_8_Ctor | likely | Writes the base HashListMap vftable, zeros head/tail/count and 20 four-byte buckets at +0x14, installs the derived vftable, then calls FUN_00a12f70 with ECX=th… |

**ValidAttrModFilter** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01434840` | ValidAttrModFilter::ValidAttrModFilter | confirmed | Constructor (ECX = object). Calls 01343d20 on the object first (a PlayElement-family constructor; it references the PlayElement, PersistentComponent and Persis… |
| `014348e0` | ValidAttrModFilter::TypeName | confirmed | Returns the address of the literal ValidAttrModFilter (mov eax, imm32; ret). Bytes: b8 cc 8d 8e 01 c3. |
| `014348f0` | ValidAttrModFilter::GetStaticDword | confirmed | Returns the dword at global DAT_01cbe2b4 (mov eax, [imm32]; ret). The same global is written by 01434900. |
| `01434900` | ValidAttrModFilter::SetStaticDword | confirmed | Stores its one stack argument into global DAT_01cbe2b4 and returns with ret 4 (mov eax,[esp+4]; mov [imm32],eax; ret 4). |
| `01434920` | ValidAttrModFilter::~ValidAttrModFilter | confirmed | Destructor body (ECX = object). Stores ValidAttrModFilter::vftable at +0 and +8, calls 01344cd0 on the object (ECX = this), then 01300cd0 on the sub-object at … |
| `014349b0` | ValidAttrModFilter::RequireExpressionTreeValue | confirmed | Thiscall (ECX = object) with one stack argument, ret 4. If the argument's dword at +4 is not 5, it calls assert routine 012f5a60 with the text valueData->isExp… |
| `01434d70` | ValidAttrModFilter::SecondaryDeletingThunk | confirmed | Adjustor thunk (bytes 83 e9 08 e9 a8 00 00 00): subtracts 8 from ECX and jumps to 01434e20. Ghidra shows a call to 01434e20; the real form is a jump. |
| `01434e20` | ValidAttrModFilter::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument, ret 4. Calls the destructor body 01434920 on the object, then calls _free on the object when bit 0 of the flag is set, an… |
| `01434990` | ValidAttrModFilter::SetField0x48 | likely | Thiscall (ECX = object) with one stack argument, ret 4: stores the argument into dword this+0x48 (mov eax,[esp+4]; mov [ecx+0x48],eax). |
| `014349e0` | ValidAttrModFilter::SetField0x5c | likely | Thiscall (ECX = object) with one stack argument, ret 4: stores the argument into dword this+0x5c (mov eax,[esp+4]; mov [ecx+0x5c],eax). |
| `01434a10` | ValidAttrModFilter::SetField0x58 | likely | Thiscall (ECX = object) with one stack argument, ret 4: stores the argument into dword this+0x58 (mov eax,[esp+4]; mov [ecx+0x58],eax). |
| `01434a20` | ValidAttrModFilter::GetField0x58 | likely | Thiscall (ECX = object), no arguments: returns the dword at this+0x58 (mov eax,[ecx+0x58]; ret). |
| `01434d80` | ValidAttrModFilter::Create | likely | Allocates 0x60 bytes with operator new (00d833a6). If the allocation is non-null it calls the constructor 01434840 on the new object and returns the object poi… |
| `01434e00` | ValidAttrModFilter::RegisterFactory | likely | Pushes the factory 01434d80 and classID 0xab (171), calls 013ce130, then calls 013ce230 with the returned value in ECX. The second call is the second registry … |

**WorldCompositeEffectEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c51a0` | WorldCompositeEffectEvent::Constructor | confirmed | Calls WorldEffectEvent initialization, installs WorldCompositeEffectEvent vtables at offsets 0 and +0x44, stores param_2 at +0x90, clears +0x94, and copies par… |
| `008c5240` | WorldCompositeEffectEvent::GetResult | confirmed | Calls the secondary object at this+0x44 virtual slot 1; if nonnull, calls that result object slot 4 and returns its value, otherwise returns zero. |
| `008c5270` | WorldCompositeEffectEvent::HasUnsetValue | confirmed | Calls the secondary object at this+0x44 virtual slot 1 and returns true only when the returned object field at +0x1c equals -1. |
| `008c5290` | WorldCompositeEffectEvent::AdjustorThunk | confirmed | Adjusts ECX by -0x44 and jumps to 0x008c5500; this is an adjustor thunk, not the callee body. |

**(no class)** (29)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00961420` | SetFlyByEffectId | confirmed | When the supplied ID differs from this+0x6d0, resolves its definition, requires definition kind 1, logs an invalid ID otherwise, and stores the ID plus the squ… |
| `00c57d40` | ParseEffectIdAndTransform | confirmed | If 00d50af0(param_3) is non-zero, reads the attributes EffectId, DirectionX/Y/Z and LocationX/Y/Z by name through 00d517f0 and converts the numeric ones with _… |
| `0079efb0` | ResourcesPrefixInitAndLoad | likely | With ECX=this: calls 0079eac0 on this (hash base init), stores the IString<char> vtable at this+0x14, the static empty buffer at this+0x18 and zeroes this+0x1c… |
| `00962dd0` | InitializeSensoryEffectRecordFields | likely | Zeros selected pointer and count fields, sets one field to -1, copies a static 4-word default transform, initializes default scalar values, constructs the embe… |
| `00968690` | CreatePlayerProjectileEventAfterLookup | likely | Thiscall with ECX = object (param_1, zero returns 0) and an out-pointer stack argument. Calls 0095e1d0 (FindHashBucketEntry) and stores its result through the … |
| `00a15a10` | HashListMap_ClientEffectTag_Clear | likely | Twenty-seven-byte loop (bytes 568bf1 8b4608 85c0 740f 50 8bce e82eedffff ...): while the head pointer at this+8 is non-null, calls 00a14750 with this as the ma… |
| `00a18a10` | ClientEffectMap_CreateEntry | likely | Creates one map entry: allocates a node through virtual slot 1 of the map, initialises it with FUN_00a10fa0 (which installs the ClientEffectTag vtable and stor… |
| `00a19e70` | FindOrCreateBucketRecordAndDecode | likely | Selects bucket this+0x6f0+(key%0x14)*4, walks nodes comparing key at +0x80 through next link +0x84, and on a miss creates an entry, sets the owner byte at +0x2… |
| `00a19f00` | InitBoundedReaderAndForwardKey | likely | Initializes a local reader with start and cursor at param_2, end at param_2+param_3, length param_3, and both flags zero; forwards param_1 as the key and the r… |
| `0079f030` | IStringSetIfChangedThenReload | speculative | Stdcall-style with ECX=this and one stack argument (C string): compares the characters of the string data at this+0x18 with the argument; when they differ, ass… |
| `009644f0` | InitializeSensoryEffectRecordWrapper | speculative | Enters an exception frame, invokes FUN_00962dd0 to initialize a record, restores the prior exception list, and returns the supplied value. |
| `00c55cc0` | ReleaseEffectHandle | speculative | If *param_1 is non-null: looks up 00769710()+0x16c; if that is non-null it calls 007e7f00(handle,1) and, when its +8 field is non-null, calls it a second time;… |
| `00c55d10` | NodeTableWalkApplyRefresh | speculative | Looks up key param_2 with 008c52a0 (NodeTable_FindKey), walks the node chain through 007e7e70, and for each node calls the virtual at param_1 vtable +0x24 to f… |
| `00c55df0` | NodeTableApplyRecord | speculative | Looks up key param_1 with 008c52a0, walks the node chain through 007e7e70, copies a 16-dword record from param_2 into a local and calls the node vtable +0x20 w… |
| `00c56c60` | TransformEffectUpdate | speculative | When the flag at +0x42c is set: copies 16 dwords from param_2 into this+0x550 (a transform), stores +0x5ef, calls 00c53d90 and on success releases the handle a… |
| `00c588f0` | EffectAtLocationWithStrings | speculative | If the first string argument is non-empty, copies it into an IString through 00402ce0, reads a 16-dword template (DAT_01bc2660 or a sub-object block) and calls… |
| `01434a30` | CheckFilterMatch | speculative | Thiscall with two stack args. Calls FUN_01418dc0 and compares its result with dword this+0x48; if they differ it returns 1. If equal it calls FUN_01418e00 and … |
| `0079ee90` | — | unknown | With ECX=this: initialises the IString member at this+0x14 (006f9e60), builds the name CodeEffectVariableMappings.txt in a local IString (00402770), loads the … |
| `0095e950` | — | unknown | For nonzero param_1, resolves a candidate with FUN_007c4050; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095ea60` | — | unknown | For nonzero param_1, resolves a candidate with FUN_00709a70; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095eb50` | — | unknown | For nonzero param_1, resolves a candidate with FUN_007dd650; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095ec40` | — | unknown | For nonzero param_1, resolves a candidate with FUN_007dd5f0; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095ed30` | — | unknown | For nonzero param_1, resolves a candidate with FUN_007dd6a0; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095ee20` | — | unknown | For nonzero param_1, resolves a candidate with FUN_008beba0; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095ef10` | — | unknown | For nonzero param_1, resolves a candidate with FUN_007dd700; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `0095f000` | — | unknown | For nonzero param_1, resolves a candidate with FUN_007dd760; when nonnull, calls its virtual slot +8 and continues only for a nonzero result. It then calls ope… |
| `00960af0` | — | unknown | Reads each linked record's kind at +0x10, resolves a type-specific object, checks its virtual state, creates the corresponding event, updates event fields and … |
| `00a15440` | — 🌐 | unknown | Looks up the effect-tag node for param_2 in the 20-bucket table at this+0x6f0 (chain +0x84, key +0x80); when the node owner field at +0x74 equals *(*(this+0x3b… |
| `01064f80` | — | unknown | Fastcall with a float block in ECX (no stack arguments). For each named parameter it reads the stored float and overrides it with the value returned by 0108a65… |

### `game.encounter`

**EncounterDefinitionManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01063230` | EncounterDefinitionManager::LoadEncounterTeamData | confirmed | Fastcall with the manager in ECX (cookie-framed, no stack arguments). Reads a text file through 00fbf4a0 (its first argument is the dword at this+0x84, flag 1)… |

**EncounterMatchmaking::MatchmakingQueue** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010704a0` | MatchmakingQueueCtor | confirmed | Stores the EncounterMatchmaking::MatchmakingQueue vtable, zeroes +0x8, constructs the request map sub-object through 010703e0, zeroes the byte at +0x224 and dw… |
| `01070540` | MatchmakingQueueDtorBody | confirmed | Stores the MatchmakingQueue vtable, then stores the SoeUtil::List<int,-1> vtable at +0x230, calls 007798c0 (not labelled) and then the chained request-map dest… |
| `010705b0` | — | unknown | Virtual slot [0] of the MatchmakingQueue vtable. Calls 01070540 (destruct body) on this, tests bit 0 of its byte argument and calls the free thunk 00d82463 whe… |

**EncounterMatchmaking::MatchmakingQueueDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0106e850` | MatchmakingQueueDefinition::ctor | confirmed | Stores the EncounterMatchmaking::MatchmakingQueueDefinition vtable and zeroes dwords +0x4..+0x4f (param_1[1] to param_1[0x13]) plus the bytes at +0x50 and +0x5… |

**EncounterMatchmaking::MatchmakingRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0106f3f0` | MatchmakingRequestCtor | confirmed | Stores the EncounterMatchmaking::MatchmakingRequest vtable, zeroes dwords +0x8, +0xc, +0x10 and +0x20, sets +0x18 and +0x1c to DAT_01cb7668/DAT_01cb766c, const… |

**(no class)** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a979a0` | GroupEncounterRespondToInviteMatchmaking | confirmed | Thiscall with two 32-bit ids and a bool accept flag. Same minigame check as 00a97720 (FUN_009ad3c0): when a minigame is active it posts GroupEncounterMustLeave… |
| `010637d0` | EncounterTeamHolder_DtorBody | confirmed | Fastcall with the holder in ECX (cookie-framed). Stores the List<EncounterTeamData,-1> vtable at +0x118 and clears that list through 00a245f0; stores the IStri… |
| `008b8ea0` | CopyEncounterSettingsRecord | likely | Copies the dword at +4, eleven bytes at +8 through +0x12, and eight dwords at +0x14 through +0x30 from param_2 into param_1. |
| `008c5b20` | ResetRuntimeFields | likely | Calls FUN_009ad420, zeroes this+0x704, copies two globals DAT_01be48b8 and DAT_01be48bc into this+0x6f8 and this+0x6fc, calls FUN_00a89b90, clears bytes at thi… |
| `00a97720` | GroupEncounterRespondToInvite 🌐 | likely | Takes a bool accept flag. If FUN_009ad3c0 reports a minigame active, shows the string GroupEncounterMustLeaveMinigame through a vtable call on this+8+4 and ret… |
| `00c241f0` | RequestPlayIntroEncounter | likely | Constructs CommandPacketRequestPlayIntroEncounter and sends the wrapped command when the client command service is active. |
| `01050cb0` | FormatEncounterFields | likely | Thiscall (this in ECX): writes the SoeUtil::IString<char> vftable into the result object param_2, sets its buffer to the static empty string 01cb6cc0 and lengt… |
| `01063950` | EncounterTeamHolder_ScalarDeletingDtor | likely | Thiscall with a flag byte. Calls the encounter-team holder destructor body 010637d0 on this, then when bit 0 of the flag is set calls _free on this (00d82463 t… |
| `010639a0` | — | unknown | Fastcall with the holder in ECX (no stack arguments, cookie-framed). Zeroes its scalar and pointer words across +0 to +0x130 (about 80 word stores in the pseud… |

### `game.projectile`

**PlayerSpawnProjectileEvent** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b84910` | PlayerSpawnProjectileEvent::Ctor | confirmed | Thiscall constructor taking param_2 and two further stack arguments. Calls FUN_008b8160, stores ActorHandler::vftable at +0x50, then PlayerSpawnProjectileEvent… |
| `00b84cc0` | PlayerSpawnProjectileEvent::ScalarDeletingDtor | confirmed | Scalar deleting destructor. Calls the dtor body 00b84bf0 on ECX (this), then calls _free on this when bit 0 of the flag argument is set, and returns this. |
| `00b84a20` | PlayerSpawnProjectileEvent::ReleaseOwnerIfCheckZero | speculative | Virtual slot [1] of the secondary PlayerSpawnProjectileEvent vtable (01823a54). Adjusts this by -0x50 into ESI, calls FUN_00ae2470 with ECX = that object, and … |
| `00b84a50` | PlayerSpawnProjectileEvent::ScalarDeleteThunkAdj50 | speculative | Virtual slot [2] of the secondary PlayerSpawnProjectileEvent vtable (01823a54). Adjusts this by -0x50, and when the adjusted pointer is non-null tail-jumps thr… |
| `00b84a70` | PlayerSpawnProjectileEvent::DeleteSelfViaSlot3 | speculative | Virtual slot [10] of the primary PlayerSpawnProjectileEvent vtable (01823a90). When this is non-null it tail-jumps through its own vtable slot +0x0c with argum… |
| `00b84a90` | PlayerSpawnProjectileEvent::ScalarDeletingDtorThunk | speculative | Thunk, 8 bytes. SUB ECX, 0x50 then JMP 00b84cc0. It adjusts this from the secondary subobject to the primary and passes the scalar deleting argument through un… |
| `00b84aa0` | PlayerSpawnProjectileEvent::ResolveAndReleaseHandle | speculative | Virtual slot [1] of the primary PlayerSpawnProjectileEvent vtable (01823a90). Calls FUN_00ae2470 and, when it returns non-zero and flag bit 0 of the argument i… |
| `00b84ce0` | PlayerSpawnProjectileEvent::BuildRequestAndDispatch | speculative | Virtual slot [3] of the secondary PlayerSpawnProjectileEvent vtable (01823a54). Calls FUN_00ae2470 and, when it returns non-zero, calls its vtable +0x24. It th… |
| `00b84bf0` | — | unknown | Non-deleting destructor body. Stores PlayerSpawnProjectileEvent::vftable in the primary and secondary vtable slots. When the handle at +0x130 is non-zero it ca… |

**Projectile** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf3620` | Projectile::DtorBody | confirmed | Fastcall (ECX=this) with an EH frame: stores Projectile::vftable at +0; when the dword at +0xc0 is nonzero calls vtable slot [2] (+8) of the object at +0x24; w… |
| `00bf4690` | Projectile::ScalarDeletingDtor | confirmed | Thiscall with one stack flag (ret 4): calls the Projectile destructor body 00bf3620 on this, frees the object through _free when bit 0 of the flag is set, and … |
| `00bf24b0` | Projectile::LerpClampedOverDuration | speculative | Thiscall with two float stack args (ret 8). Calls vtable slot [4] of the object at this+0x24 with dword this+0xc0; when that returns nonzero and the duration a… |
| `00bf2620` | Projectile::UpdateArrivalFlag | speculative | Fastcall, ECX=this, ret 4. Sets the byte at this+0x94 when 0 < +0x164 < +0x90 or when +0x90 exceeds the constant at 017eb904. When the object at this+0x24 (slo… |
| `00bf2880` | Projectile::UpdateTravelMetrics | speculative | Fastcall (ECX=this, plain ret). Builds the current position at +0x80 from the velocity at +0x70..+0x7c scaled by DAT_017ec67c plus the start at +0x50, unless 0… |
| `00bf29d0` | Projectile::UpdateStep | speculative | Thiscall, one stack arg dt (ret 4). When the byte at this+0x94 (finished) is clear and this+0x28 is non-null, calls vtable slot [1] of the object at +0x28 with… |
| `00bf3840` | Projectile::SetPositionRecordPath | speculative | Fastcall-style thiscall with one pointer arg (a vector). Copies the vector to +0x50..+0x5c. When the object at +0x24 (slot [4] with +0xc0) returns zero it sets… |
| `00bf39d0` | Projectile::UpdateOrientationSpin | speculative | Thiscall with a float4 pointer and one float dt (ret 8). Advances the spin angle at +0x60 by +0x130 times dt. When the object at +0x24 (slot [4] with +0xc0) re… |
| `00bf49f0` | Projectile::InitFromSource | speculative | Thiscall with two stack args (ret 8): a source object and a second object whose +0x268 dword is stored at this+0xc0 (0 when the second arg is null). Calls 006f… |
| `00bf23f0` | — | unknown | Fastcall (ECX=this, no args): loads the vtable of this and jumps to its slot [5] (+0x14) with the same registers (tail jump, 7 bytes). Binary virtual slot [4] … |
| `00bf34e0` | — | unknown | Thiscall with three stack args (ret 0xc). Installs Projectile::vftable, stores arg3 at +0x20 and the dword at arg3+4 at +0x24, stores arg2 at +0x98, zeroes the… |
| `00bf36d0` | — | unknown | Fastcall (ECX=this, plain ret). When the dword at +0x158 is nonzero it reads the held pair (0101b8d0). If the pair is the sentinel it reads a held object throu… |

**ProjectileArc** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf3ea0` | ProjectileArc::Ctor | confirmed | Thiscall (ECX=this) with three stack args: calls the Projectile base constructor 00bf34e0 with the same args, then installs ProjectileArc::vftable at +0. |
| `00bf4710` | ProjectileArc::ScalarDeletingDtor | confirmed | Thiscall with one stack flag (ret 4): calls the Projectile destructor body 00bf3620 on this, frees the object when bit 0 of the flag is set, and returns this. … |
| `00bf2bb0` | — | unknown | Thiscall, one stack arg dt (ret 4). When the byte at +0x94 is clear and this+0x28 is non-null, calls the object at +0x28 slot [1] with (this, dt*_DAT_017ebf0c)… |

**ProjectileBeam** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf3de0` | ProjectileBeam::Ctor | confirmed | Thiscall (ECX=this) with three stack args (ret 0xc): calls the Projectile base constructor 00bf34e0 with the same three args and this, then installs Projectile… |
| `00bf46b0` | ProjectileBeam::ScalarDeletingDtor | confirmed | Thiscall with one stack flag (ret 4): calls the Projectile destructor body 00bf3620 on this, frees the object through _free when bit 0 of the flag is set, and … |
| `00bf2a10` | ProjectileBeam::UpdateStep | speculative | Thiscall, one stack arg dt (ret 4). When the byte at +0x94 is clear and this+0x28 is non-null, calls the object at +0x28 slot [1] with (this, dt*_DAT_017ebf0c)… |

**ProjectileBoomerang** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf45c0` | ProjectileBoomerang::Ctor | confirmed | Thiscall (ECX=this) with three stack args: calls the Projectile base constructor 00bf34e0, then installs ProjectileBoomerang::vftable at +0. |
| `00bf4830` | ProjectileBoomerang::ScalarDeletingDtor | confirmed | Thiscall with one stack flag (ret 4): calls the Projectile destructor body 00bf3620 on this, frees the object when bit 0 of the flag is set, and returns this. … |
| `00bf4630` | ProjectileBoomerang::GetByte180 | speculative | Fastcall, no args: returns the byte at this+0x180 (mov al,[ecx+0x180]; ret). Binary virtual slot [9] of ProjectileBoomerang. |
| `00bf4b10` | ProjectileBoomerang::InitFromSource | speculative | Thiscall with two stack args. Calls the Projectile base init 00bf49f0 with them, then sets this+0x178 = DAT_017ea718 (2.0), the 4-float constant at +0x120..+0x… |
| `00bf3160` | — | unknown | Thiscall, one stack arg dt (ret 4). When the byte at +0x94 is clear and +0x28 is non-null calls its slot [1]; calls 00bf2880; when not finished it uses 0090454… |

**ProjectileManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b83d20` | ProjectileManager::Ctor | confirmed | Thiscall constructor with three stack arguments after this. Stores ProjectileManagerInterface::vftable and then ProjectileManager::vftable into this, zeroes th… |
| `00b84100` | ProjectileManager::~ProjectileManager | confirmed | Non-deleting destructor body. Stores ProjectileManager::vftable into this, calls ClearAll (00b84050) on the list, then calls the base destructor body FUN_010c1… |
| `00b84590` | ProjectileManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor. Calls the dtor body FUN_00b84100 on ECX (this), then calls _free on this when bit 0 of the flag argument is set, and returns this. |
| `00b84050` | ProjectileManager::ClearAll | likely | Fastcall with this in ECX. Walks the whole list from this+0x08 without any flag test. For each node it unlinks it from its bucket (this+0x14 + (key % 100) * 4)… |
| `00b84e90` | — | unknown | Per-update steering pass of ProjectileManager (vtable slot 1): for the owner object param_2 it resolves this+0x1ac->+200 into a world/actor pointer, then eithe… |

**ProjectileManagerInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b83bb0` | ProjectileManagerInterface::ScalarDeletingDtor | confirmed | Scalar deleting destructor. Calls the dtor body FUN_010c17e0 on ECX (this), then calls _free on this when bit 0 of the flag argument is set, and returns this. |
| `010c17e0` | ProjectileManagerInterface::DtorBody | confirmed | Fastcall destructor body (ECX = this). Stores ProjectileManagerInterface::vftable into [this] and returns. Frees nothing. |

**ProjectileSeek** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf4020` | ProjectileSeek::Ctor | confirmed | Thiscall (ECX=this) with three stack args: calls the Projectile base constructor 00bf34e0, then installs ProjectileSeek::vftable at +0. |
| `00bf47d0` | ProjectileSeek::ScalarDeletingDtor | confirmed | Thiscall with one stack flag (ret 4): calls the Projectile destructor body 00bf3620 on this, frees the object when bit 0 of the flag is set, and returns this. … |
| `00bf40f0` | ProjectileSeek::UpdateStep | speculative | Thiscall, one stack arg dt (ret 4), large body (about 1.2 KB of code). When the byte at +0x94 is clear and this+0x28 is non-null it calls the object at +0x28 s… |

**ProjectileThrowCatch** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf3f60` | ProjectileThrowCatch::Ctor | confirmed | Thiscall (ECX=this) with three stack args: calls the Projectile base constructor 00bf34e0, then installs ProjectileThrowCatch::vftable at +0. |
| `00bf4770` | ProjectileThrowCatch::ScalarDeletingDtor | confirmed | Thiscall with one stack flag (ret 4): calls the Projectile destructor body 00bf3620 on this, frees the object when bit 0 of the flag is set, and returns this. … |
| `00bf2e40` | ProjectileThrowCatch::UpdateStep | speculative | Thiscall, one stack arg dt (ret 4). Same control flow and interpolation-segment code as ProjectileArc::UpdateStep (00bf2bb0) without the extra store to +0x11c … |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf4890` | CopyStateFromSource | speculative | Thiscall with one stack arg (source object). Copies dwords +0x10..+0x2c, +0x40 (byte), +0x50..+0xcc from the source to this, calls 0101c8a0 (ReplaceHeldWithClo… |

### `game.stats`

**AttributeModifier** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01418c40` | AttributeModifier::AttributeModifier | confirmed | Fastcall with ECX = this. Runs FUN_01343d20 under an SEH frame, stores AttributeModifier::vftable at this+0 and at this+8, calls FUN_012fa910 and FUN_01385940 … |
| `01418d00` | AttributeModifier::GetTypeName | confirmed | Virtual slot 11 of AttributeModifier::vftable. Returns the address of the text AttributeModifier (mov eax, imm32; ret). |
| `01418d40` | AttributeModifier::~AttributeModifier | confirmed | Fastcall with ECX = this. Stores AttributeModifier::vftable at this+0 and at this+8, runs FUN_01344cd0 under the SEH state, then FUN_01385460 and FUN_01300cd0 … |
| `014190d0` | AttributeModifier::Serialize | confirmed | Virtual slot 10 of AttributeModifier::vftable. Calls the writer virtual at vtable+8 on this with the writer argument. Labels go through FUN_013c26e0 (a no-op) … |
| `01418d10` | AttributeModifier::GetStaticDword | likely | Virtual slot 12 of AttributeModifier::vftable. Returns the dword in the global DAT_01cbe25c (mov eax,[abs]; ret). |
| `01418d20` | AttributeModifier::SetStaticDword | likely | Virtual slot 13 of AttributeModifier::vftable. Stores its one stack argument into the global DAT_01cbe25c and returns with ret 4. |
| `014192d0` | AttributeModifier::AdjustThunkMinus8 | likely | Adjustor thunk for slot 0 of AttributeModifier::vftable: subtracts 8 from ECX and jumps (e9) to FUN_01419380. It has no body of its own. |
| `01418eb0` | AttributeModifier::Deserialize | speculative | Virtual slot 9 of AttributeModifier::vftable. Calls the reader virtual at vtable+0x10 with the value (false at AttributeModifier.cpp line 0x153 on failure), th… |
| `01419380` | AttributeModifier::ScalarDeletingDtorSecondary | speculative | Slot 0 of the second AttributeModifier vtable (018e607c) with the scalar deleting shape: with ECX = this it calls the destructor body 01418d40, tests bit 0 of … |
| `014193a0` | — | unknown | Virtual slot 16 of AttributeModifier::vftable (secondary vtable 018e607c). Thiscall with five stack arguments (param_2 to param_6). It tests FUN_01348760 and p… |
| `01419650` | — | unknown | Virtual slot 17 of the second AttributeModifier vtable (018e607c). Thiscall with the value output (param_3) and the changed flag output (param_4). It clears bo… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a12aa0` | CharacterStatMap_Find | likely | Looks up param_2 (key) in the 71-bucket hash table at this+0x35afc+(key % 0x47)*4, walking the chain at +0x18 and comparing the key field at +0x14; returns the… |
| `00a12ae0` | CharacterStatMap_GetOr | likely | Same 71-bucket character-stat lookup; returns the node's value dword at +8 when found, otherwise returns the caller-supplied default param_3. |
| `00a12b20` | CharacterStatMap_GetFloatOr | likely | Same 71-bucket character-stat lookup; returns the float at node+8 when found, otherwise the default param_3 (returned as a float10). |
| `00b404b0` | — | unknown | Logs one transaction record through the log writer FUN_00720c60: 'Total Transaction Time: %d seconds' as (+0x10 - +8), 'Time into Session: %d seconds' as (+8 -… |
| `00b40550` | — | unknown | Builds a StringFixed<2048> (constructor FUN_00402a70) from the order object's virtual slot +8 ((*(this+0x50))[8]) before calling the per-transaction logger 00b… |

### `game.target`

**MoveAndSelectTargetControllerHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008bb4d0` | MoveAndSelectTargetControllerHandler::LoadDefaultPair | likely | Writes the two global dwords at object offsets +8 and +0x0c and returns. |

**Target** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101b370` | Target::~Target | confirmed | Destructor body: stores Target::vftable at *this, calls vftable slot 0 of the object at this+4 with argument 1 when that pointer is nonnull, then clears this+4. |
| `0101c850` | Target::Slot1ReadTypedHeld | confirmed | Thiscall, one stack argument (reader): reads a dword type code from the reader cursor (bounded by reader+0xc; error byte set when exhausted and the function re… |
| `0101c930` | Target::Slot2WriteTypedHeld | confirmed | Thiscall, one stack argument (writer): when this+4 is nonnull, reads the held object's type code through its vftable slot 1 (offset 4), writes it with 00911420… |

**TargetActorBone** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101bef0` | TargetActorBone::TargetActorBone | confirmed | Constructor (thiscall, stack arguments param_2 and param_3): stores TargetBase::vftable, copies the 16-byte constant from 01cb53d0 into +0x10..+0x1c, stores Ta… |
| `0101c040` | TargetActorBone::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetActorBone): calls 0101bfc0 on this, then calls _free(this) when bit 0 of the byte argument is set; returns this. |
| `0101c230` | TargetActorBone::CtorFromSource | confirmed | Constructor (thiscall, one stack argument): stores TargetBase::vftable, copies the 16-byte constant from 01cb53d0 into +0x10..+0x1c, stores TargetActorBone::vf… |
| `0101bfc0` | TargetActorBone::DtorBody | likely | Destructor body: stores the IString<char> vftable at this+0x24, releases the buffer at this+0x28 when the dword at this+0x30 is positive (refcount decremented … |
| `0101bf90` | TargetActorBone::Slot6ReturnSelf | speculative | Thiscall, no arguments: bytes 8b c1 c3 = mov eax,ecx; ret. Returns this. |
| `0101bfa0` | TargetActorBone::Slot9WriteDwordAt20 | speculative | Thiscall, one stack argument (ret 4): loads the dword at this+0x20 and stores it through the argument pointer. |
| `0101bfb0` | TargetActorBone::Slot10GetDwordAt28 | speculative | Thiscall, no arguments: bytes 8b 41 28 c3 = mov eax,[ecx+0x28]; ret. Returns the dword at this+0x28. |

**TargetActorBoneId** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101b760` | TargetActorBoneId::TargetActorBoneId | confirmed | Constructor: stores TargetBase::vftable at *this, copies the 16-byte constant from 01cb53d0 into +0x10..+0x1c, stores TargetActorBoneId::vftable, then stores t… |
| `0101b870` | TargetActorBoneId::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetActorBoneId): stores TargetBase::vftable at *this, then calls _free(this) when bit 0 of the byte argument is se… |
| `0101bc60` | TargetActorBoneId::CtorFromSource | confirmed | Constructor (thiscall, one stack argument): stores TargetBase::vftable, copies the 16-byte constant from 01cb53d0 into +0x10..+0x1c, stores TargetActorBoneId::… |
| `0101b7f0` | TargetActorBoneId::Slot7ReturnSelf | speculative | Thiscall, no arguments: bytes 8b c1 c3 = mov eax,ecx; ret. Returns this. |
| `0101b800` | TargetActorBoneId::Slot11GetDwordAt24 | speculative | Thiscall, no arguments: bytes 8b 41 24 c3 = mov eax,[ecx+0x24]; ret. Returns the dword at this+0x24. |
| `0101b860` | TargetActorBoneId::Slot9WriteDwordAt20 | speculative | Thiscall, one stack argument (ret 4): loads the dword at this+0x20 and stores it through the argument pointer. |

**TargetBase** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101b090` | TargetBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetBase): stores TargetBase::vftable at *this, then calls _free(this) when bit 0 of the byte argument is set; retu… |
| `0101b170` | TargetBase::TargetBase | confirmed | Constructor body under an SEH frame (push -1, handler, fs:[0] link): stores TargetBase::vftable at *this and returns. |
| `0101afd0` | TargetBase::SetBaseVftable | likely | Stores the TargetBase vftable at *this (bytes c7 01 imm32 c3, a single store and return). Thiscall, no arguments. |
| `0101b1c0` | TargetBase::~TargetBase | likely | Destructor body under an SEH frame: stores TargetBase::vftable at *this and returns. Same body as the constructor 0101b170 but separate code; seven EH unwind f… |
| `0101b060` | TargetBase::Slot12PayloadAddr | speculative | Thiscall, no arguments: bytes 8d 41 10 c3 = lea eax,[ecx+0x10]; ret. Returns the address of the 16-byte payload at this+0x10. |
| `0101b070` | TargetBase::Slot13CopyPayloadIn | speculative | Thiscall, one stack argument (ret 4): copies 16 bytes from the argument (4 dwords) into this+0x10..+0x1c. |
| `0101b460` | TargetBase::Slot8WriteConstPair | speculative | Bytes 8b 44 24 04 (mov eax,[esp+4]), two loads of 01cb6148 and 01cb614c, two stores to [eax] and [eax+4], ret 4. Writes an 8-byte constant pair to its argument… |
| `0101b6b0` | TargetBase::SetBaseVftableSEH | speculative | Fastcall, ECX = this, under an SEH frame: stores TargetBase::vftable at *this and returns. Same body as 0101b810. |
| `0101b810` | TargetBase::SetBaseVftableSEH | speculative | Fastcall, ECX = this, under an SEH frame: stores TargetBase::vftable at *this and returns. Same body as 0101b6b0. |
| `0101bb90` | TargetBase::Slot9WriteZero | speculative | Thiscall, one stack argument (ret 4): stores 0 as a dword through the argument pointer (mov eax,[esp+4]; mov dword [eax],0). |
| `0101c770` | TargetBase::Slot14ReadFloat4 | speculative | Thiscall, one stack argument (reader): reads a float4 into this+0x10 by calling 008e2410(reader, this+0x10); returns nothing. |
| `0101cc60` | TargetBase::Slot15WriteFloat4 | speculative | Thiscall, one stack argument (writer): appends the four dwords at this+0x10 to the writer via 0091a640. |

**TargetCharacterBone** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101bd90` | TargetCharacterBone::TargetCharacterBone | confirmed | Fastcall constructor: calls 0101b4e0 on this (TargetCharacterGuid base), stores TargetCharacterBone::vftable at *this, zeroes +0x38 and +0x3c, stores the IStri… |
| `0101bed0` | TargetCharacterBone::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetCharacterBone): calls 0101be50 on this, then calls _free(this) when bit 0 of the byte argument is set; returns … |
| `0101c190` | TargetCharacterBone::CtorFromSource | confirmed | Constructor (thiscall, one stack argument): calls 0101bae0 on this with the argument, stores TargetCharacterBone::vftable, stores the IString<char> vftable at … |
| `0101be50` | TargetCharacterBone::DtorBody | likely | Destructor body: stores the IString<char> vftable at this+0x30, releases the buffer at this+0x34 when the dword at this+0x3c is positive (refcount decremented … |
| `0101be30` | TargetCharacterBone::Slot4ReturnSelf | speculative | Thiscall, no arguments: bytes 8b c1 c3 = mov eax,ecx; ret. Returns this. |
| `0101be40` | TargetCharacterBone::Slot10GetDwordAt34 | speculative | Thiscall, no arguments: bytes 8b 41 34 c3 = mov eax,[ecx+0x34]; ret. Returns the dword at this+0x34. |
| `0101cce0` | TargetCharacterBone::Slot15WriteFloat4Sub20Sub30 | speculative | Thiscall, one stack argument (writer): appends the float4 at this+0x10 via 0091a640, calls 008fb810 on this+0x20, then calls 00911c30 on this+0x30. |

**TargetCharacterBoneId** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101b600` | TargetCharacterBoneId::TargetCharacterBoneId | confirmed | Thiscall constructor with three stack arguments (param_2..param_4): calls 0101b4e0 on this with param_2 and param_3 (pushed as a stack pair), stores TargetChar… |
| `0101b700` | TargetCharacterBoneId::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetCharacterBoneId): stores TargetBase::vftable at *this, then calls _free(this) when bit 0 of the byte argument i… |
| `0101bbe0` | TargetCharacterBoneId::CtorFromSource | confirmed | Constructor (thiscall, one stack argument): calls 0101bae0 on this with the argument, stores TargetCharacterBoneId::vftable at *this, sets this+0x30 to -1, and… |
| `0101b690` | TargetCharacterBoneId::Slot5ReturnSelf | speculative | Thiscall, no arguments: bytes 8b c1 c3 = mov eax,ecx; ret. Returns this. |
| `0101b6a0` | TargetCharacterBoneId::Slot11GetDwordAt30 | speculative | Thiscall, no arguments: bytes 8b 41 30 c3 = mov eax,[ecx+0x30]; ret. Returns the dword at this+0x30. |

**TargetCharacterGuid** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101b4e0` | TargetCharacterGuid::TargetCharacterGuid | confirmed | Constructor: stores TargetBase::vftable, copies the 16-byte constant from 01cb53d0 into +0x10..+0x1c, stores TargetCharacterGuid::vftable, then stores the two … |
| `0101b5a0` | TargetCharacterGuid::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetCharacterGuid): stores TargetBase::vftable at *this, then calls _free(this) when bit 0 of the byte argument is … |
| `0101bae0` | TargetCharacterGuid::CtorCopyIdFrom | confirmed | Constructor (thiscall, one stack argument): stores TargetBase::vftable, copies the 16-byte constant from 01cb53d0 into +0x10..+0x1c, stores TargetCharacterGuid… |
| `0101b570` | TargetCharacterGuid::Slot2ReturnSelf | speculative | Thiscall, no arguments: bytes 8b c1 c3 = mov eax,ecx; ret. Returns this. |
| `0101b580` | TargetCharacterGuid::Slot8WriteIdPair | speculative | Thiscall, one stack argument (ret 4): copies the two dwords at this+0x20 and this+0x24 into the argument's first and second dwords (mov edx,[ecx+0x20]; mov ecx… |
| `0101c7c0` | TargetCharacterGuid::Slot14ReadFloat4PlusPair | speculative | Thiscall, one stack argument (reader): reads a float4 into this+0x10 (008e2410), then two dwords into this+0x20/+0x24 from the reader cursor (zeroed and error … |
| `0101ccb0` | TargetCharacterGuid::Slot15WriteFloat4Sub20 | speculative | Thiscall, one stack argument (writer): appends the float4 at this+0x10 via 0091a640, then calls 008fb810 on this+0x20. |

**TargetLocation** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0101b0b0` | TargetLocation::TargetLocation | confirmed | Constructor: stores TargetBase::vftable at *this, copies a 16-byte constant from 01cb53d0 into this+0x10..+0x1c, then stores TargetLocation::vftable, copies th… |
| `0101b3a0` | TargetLocation::CtorCopyPayload | confirmed | Thiscall with one stack argument: stores TargetBase::vftable, copies the 16-byte constant from 01cb53d0 to this+0x10..+0x1c, stores TargetLocation::vftable, th… |
| `0101b480` | TargetLocation::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vftable[0] of TargetLocation): stores TargetBase::vftable at *this, then calls _free(this) when bit 0 of the byte argument is set; … |
| `0101b160` | TargetLocation::Slot3ReturnSelf | speculative | Thiscall, no arguments: bytes 8b c1 c3 = mov eax,ecx; ret. Returns this. |
| `0101c790` | TargetLocation::Slot14ReadThreeFloat4 | speculative | Thiscall, one stack argument (reader): calls 008e2410 three times, filling this+0x10, this+0x20 and this+0x30 from the reader. |
| `0101cc80` | TargetLocation::Slot15WriteThreeFloat4 | speculative | Thiscall, one stack argument (writer): appends the float4 values at this+0x10, this+0x20 and this+0x30 via 0091a640 in order. |

**TargetProcessor** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acb770` | TargetProcessor::TargetProcessor | confirmed | Stores the two constructor arguments at +0x10/+0x14, installs TargetProcessor::vftable, zeros state through +0x50, and initializes float +0x54 to zero. It inst… |
| `00acb8c0` | TargetProcessor::Destructor | confirmed | Non-deleting destructor: resets *this to TargetProcessor::vftable, reinstalls the embedded list vtable at this+0x78 and removes every handler node through FUN_… |
| `00acbc60` | TargetProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor (27 bytes): calls the non-deleting destructor 00acb8c0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00acb490` | NotifyHandlers | likely | Stores the event value at this+0x20 and iterates the handler chain resolved from this+0x7c (next pointer at element+8): each handler's virtual slot +8 is calle… |
| `00acb920` | — | unknown | Scans the 0x30-byte array using base +4, count +8 and stride 0x30. In a preliminary pass enabled when param_6==0 and the context at this+0x14+200 exists, it re… |
| `00acbc80` | — | unknown | Adds the time delta to this+0x98 and delta*DAT_017ebf0c to float +0x54. If state byte +0x88 is clear and a target at +0x28 is set, it clears the selected targe… |
| `00acc160` | — | unknown | Mirrors input byte param_2+4 bits 8, 0x20 and 0x10 into state +0x24 bits 8, 0x10 and 0x20. If this+0x28 is nonzero, maps kinds 6/7/8/0xc/0xd/0xe/0xf/0x10/0x11 … |

**TargetProcessorEventHandler** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3390` | MatchOneAndZero | confirmed | Returns 1 only when param_1 equals 1 and param_2 equals 0, and 0 for every other argument pair. |
| `008b3380` | TargetProcessorEventHandler::DestructorCleanup_008b3380 | likely | Stores TargetProcessorEventHandler::vftable into *this and returns; a 7-byte fragment with no free, called from MSVC unwind funclets (0157f780, 0157f7b0, 0157f… |
| `008b3400` | TargetProcessorEventHandler::DestructorCleanup_008b3400 | likely | Stores TargetProcessorEventHandler::vftable into *this and returns; 67 bytes of SEH prologue plus the single store, called from unwind funclets 01582320, 01582… |
| `008b34e0` | TargetProcessorEventHandler::DestructorCleanup_008b34e0 | likely | Stores TargetProcessorEventHandler::vftable into *this and returns; 67 bytes with SEH prologue, called from unwind funclets 01581580, 015815b0, 015822f0, 015b1… |
| `008b3600` | TargetProcessorEventHandler::DestructorCleanup_008b3600 | likely | Stores TargetProcessorEventHandler::vftable into *this and returns; called from unwind funclets 015815e0, 01581610, 015823b0, 015a3740 and neighbours. |
| `008b33b0` | — | unknown | Stores TargetProcessorEventHandler::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**TargetProcessorPlayerEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3470` | TargetProcessorPlayerEventHandler::TargetProcessorPlayerEve… | confirmed | Installs TargetProcessorEventHandler::vftable, zeroes +4 and +8, then overwrites the first dword with TargetProcessorPlayerEventHandler::vftable, so the finish… |
| `008b3530` | — | unknown | Stores TargetProcessorEventHandler::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**TargetProcessorTileEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3590` | TargetProcessorTileEventHandler::TargetProcessorTileEventHa… | confirmed | Installs TargetProcessorEventHandler::vftable, zeroes +4 and +8, then stores TargetProcessorTileEventHandler::vftable into the first dword. |
| `008b3650` | — | unknown | Stores TargetProcessorEventHandler::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**TileEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093bcc0` | TileEventHandler::HandleTargetEvent | likely | For event kind 8, initializes a temporary Target, copies two supplied values into its state, then calls the shared input-event handler. |

**(no class)** (22)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acae80` | DispatchCollisionEvent | confirmed | Resolves the actor through FUN_00fd8230(FUN_00769710(this+0x28)); when no actor exists it switches on the collision object's virtual slot +4 (types 0,1,4,5,6 -… |
| `008c61e0` | TargetHolder_ApplyLocalTarget | likely | Sets up an exception frame, builds a local object whose first dword is Target::vftable and second dword is zero, calls FUN_0101b910 with this+0x3a0, this+0x3a4… |
| `008c6c10` | TargetPair_Apply | likely | Scans the list at (this+500)+0x358 for node field +0x10 equal to -0x17dd1f30 (empty loop). If FUN_00d83c90() is not 1, stores the default pair DAT_01be4850/DAT… |
| `00acae30` | CopyThreeInputBitsToState | likely | Mirrors source byte param_2+4 bits 0, 2 and 1 into this+0x24 bits 0, 2 and 1 respectively, clearing each destination bit when its source bit is clear. Machine-… |
| `00acb170` | NormalizeQuaternionAndProbe | likely | Resolves object +0x1a0 and the object at the +0x130 pointer chain. If both resolve, normalizes the four floats at param_3 with rsqrtps/refinement and a NaN mas… |
| `00a312c0` | ArrayGetOrEmplaceSlot8 | speculative | Returns this[4] + index*8. When index is at or beyond the count at this+8, first calls 00a2f990 to emplace the slot at that index. |
| `0101b230` | ReleasePtrAt4ScalarDelete | speculative | Fastcall, ECX = this. If the pointer at this+4 is nonnull, calls its vftable slot 0 with argument 1 (scalar delete), then stores 0 at this+4. |
| `0101b8d0` | ReadHeldPairOrDefault | speculative | Thiscall with an out pointer argument: if this+4 is null it uses the constant at 01cb6148; otherwise it calls vftable slot 8 (offset 0x20) of the held object w… |
| `0101b9b0` | ReplaceHeldWithNewObjC | speculative | Fastcall, ECX = this, with three stack arguments: releases the held object at this+4 (vslot 0 with 1 pushed), allocates 0x40 bytes (00d833a6); on success calls… |
| `0101bba0` | HeldObj_ReadDwordViaSlot9 | speculative | Thiscall with one stack argument (ret 4): if this+4 is nonnull, calls vftable slot 9 (offset 0x24) of the held object with a pointer to a local holding this, t… |
| `0101c300` | ConstructByTypeFromSource | speculative | Switch on param_1 (type 1..6): allocates 0x30 bytes for types 1 and 6 and 0x40 bytes for types 2..5 (00d833a6); on success returns the result of the matching c… |
| `0101c8a0` | ReplaceHeldWithClone | speculative | Thiscall, one stack argument (source): releases the held object at this+4 (vslot 0 with 1 when nonnull), sets this+4 = 0; when the source's +4 pointer is nonnu… |
| `0101c990` | ReplaceHeldWithCloneAlt | speculative | Thiscall, one stack argument (source): same body as ReplaceHeldWithClone (0101c8a0): releases the held object at this+4 (vslot 0 with 1 when nonnull), sets thi… |
| `0101c9d0` | ReplaceHeldWithCloneOrClear | speculative | Thiscall, one stack argument (source): releases the held object at this+4 (vslot 0 with 1 when nonnull) and sets this+4 = 0; when the source pointer is nonnull… |
| `008c80e0` | — | unknown | Sets up an exception frame, increments the count at this+8 (calling FUN_008ac460(count, 0) when the capacity at this+0xc is short, stdcall), computes the eleme… |
| `008c9f90` | — | unknown | Reserves count+param_3 through FUN_008ac460(newcount, 0) when the capacity at +0xc is short. For each of param_3 new elements at buffer+count*8: stores Target:… |
| `0101b250` | — | unknown | Thiscall with one stack argument: releases the held object at this+4 (vslot 0 with 1 when nonnull), allocates 0x40 bytes (00d833a6); on success it calls 0101b0… |
| `0101b2f0` | — | unknown | Thiscall with two stack arguments: releases the held object at this+4 (vslot 0 with 1 when nonnull), allocates 0x40 bytes (00d833a6); on success it calls 0101b… |
| `0101b910` | — | unknown | Fastcall, ECX = this, three stack arguments (ret 0xc): releases the held object at this+4 (vslot 0 with no argument when nonnull), allocates a block (00d833a6)… |
| `0101ba50` | — | unknown | Fastcall, ECX = this: releases the held object at this+4 (vslot 0 with no argument when nonnull), allocates a block (00d833a6); on success calls 0101b760 on it… |
| `0101c060` | — | unknown | Fastcall, ECX = this: releases the held object at this+4 (vslot 0 with no argument when nonnull), allocates a block (00d833a6); on success calls 0101bd90 on it… |
| `0101c100` | — | unknown | Fastcall, ECX = this: releases the held object at this+4 (vslot 0 with no argument when nonnull), allocates a block (00d833a6); on success calls 0101bef0 on it… |

### `ui.actionbar`

**ActionBarDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c48f60` | ActionBarDataSource_GetColumnName | confirmed | Column-name accessor (vtable slot 1 of the ActionBarDataSource subobject vtable): a switch over the column index returns the literal names Assigned (0), Icon I… |
| `00c49030` | ActionBarDataSource_GetValue | confirmed | ActionBarDataSource vtable slot 4. It obtains a service/object through 00769710 and follows fields +0x3e0, +0x504 and a +0x4a8 hash chain; it bounds param_2 be… |
| `00c492a0` | ActionBarDataSource::~ActionBarDataSource | confirmed | Destructor body: installs the four ActionBarDataSource base-subobject vtables (+0, +0x30, +0x34 and the +0x254 subobject), destroys the embedded list-like memb… |
| `00c49420` | ActionBarDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of the 0183212c vtable): runs the destructor body 00c492a0 and frees the object with CRT _free when bit 0 of the flag… |
| `00c48fe0` | ActionBarDataSource_NotifyOwner | likely | Vtable slot 6 of the ActionBarDataSource subobject at +0x254: loads the owning object from ecx-0x254, adjusts this to that owner and tail-jumps to the owner's … |
| `00c49350` | ActionBarDataSource_SecondaryScalarDeletingDtor | likely | Eleven-byte adjustor thunk (bytes 81 e9 54 02 00 00 e9 ...): subtracts 0x254 from ecx and jumps to the scalar deleting destructor 00c49420, i.e. the deleting d… |
| `00c49360` | — | unknown | Constructor taking two arguments: registers the data source name 'BaseClient.ActionBar' via 00922890, runs another base constructor 0081b470, stores the argume… |

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ee3d0` | BaseClient::RefreshActionBarSlots | confirmed | Virtual slot [34] of PlayClient and BaseClient. Requires this+0x3e0, +0x17c and +0x174 to be non-null; initialises the action-bar Lua handler (009cf6a0) and lo… |

**ClientActionBarDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf3b20` | ClientActionBarDataSource::GetFieldName | confirmed | Maps field ids 0 through 8 to Is Empty, Icon ID, Name, Is Enabled, Refresh Time Left, Total Refresh Time, Quantity, Is Targeting, and Icon Tint Value; returns … |
| `00cf3bb0` | ClientActionBarDataSource::GetCount | confirmed | Returns the dword at the nested object offset this+0x224+0x18. |
| `00cf3bc0` | ClientActionBarDataSource::GetFieldValue | confirmed | Finds an action-bar slot by id in an eight-bucket map and returns selected fields for ids 0 through 8, including empty, icon id, name, enabled, refresh, quanti… |
| `00cf3d60` | ClientActionBarDataSource::Dtor | confirmed | Restores the ClientActionBarDataSource vtable at object/base offsets and destroys the embedded GUI datasource base through 0091bf60. |
| `00cf3dd0` | ClientActionBarDataSource::Ctor | confirmed | Constructs the GUI datasource base, installs the ClientActionBarDataSource vtable at its base offsets, stores the owner pointer at +0x254, and returns this. |
| `00cf3e50` | ClientActionBarDataSource::ScalarDeletingDtor | confirmed | Calls the ClientActionBarDataSource destructor body, frees the object when the low bit of the flag is set, and returns the object otherwise. |

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009cf6a0` | InitializeActionBarLuaHandler | confirmed | Looks up HandlerActionBar; when found, invokes its :init callback and resets this+0xb0 to zero. |
| `009cf7d0` | SetActionBarVisible | confirmed | When the requested visibility differs from this+0xb4, looks up HandlerActionBar, invokes :show or :hide, and stores the new byte if lookup succeeds. |
| `009cf920` | SetActionBarButtonData | confirmed | Builds four Lua parameters, calls HandlerActionBar:setButtonData, raises the highest action index, stores a per-index dword, and cleans temporary values. |
| `009cfb80` | SetActionBarItemCount | confirmed | Builds two Lua parameters and invokes HandlerActionBar:setItemCount, then destroys the temporary parameter array. |
| `009cfd60` | SetActionBarTimer | confirmed | Invokes HandlerActionBar:setTimer when an action's timer is new or extends the existing deadline, sets its timed bit, and stores the computed deadline. |
| `009d0030` | EnableActionBarSlot | confirmed | For slots with active bit 0, sends HandlerActionBar:setEnabled(slot) and then clears bit 0 in the local slot flags. |
| `009d01f0` | DisableActionBarSlot | confirmed | For slots whose active bit is clear, sends HandlerActionBar:setDisabled(slot) and sets bit 0 in the local slot flags. |
| `009d0570` | SetActionBarTargetingOn | confirmed | For slots without bit 3 set, sends HandlerActionBar:setTargetingOn(slot) and sets bit 3 in the local flags. |
| `009d0730` | CancelActionBarTargeting | confirmed | For slots with bit 3 set, sends HandlerActionBar:setTargetingCanceled(slot) and clears bit 3. |
| `009d0a70` | SetTargetingFlagForAllSlots | confirmed | When the selected slot lacks bit 1, cancels targeting on all slots, sends HandlerActionBar:setTargetingOn for the selected index, then sets bit 1. |
| `00a12840` | ItemActionBar_LookupAndGate | confirmed | Finds the action-bar object whose id equals param_3 in the 5-bucket table at this+0x35434+(id%5)*4 (chain via +0x398, id at +0x39c) and then calls FUN_00c44160… |
| `009d09f0` | CancelTargetingForAllSlots | likely | Loops over slot indices zero through this+0xb0 inclusive and calls FUN_009d03b0 for each index. |
| `009d0a20` | StartTimersForOtherActionSlots | likely | For each index other than the supplied one, starts a 500-unit timer when bit 2 is clear. |
| `00a125d0` | ResolveItemStateForBar | likely | Resolves the current player object with FUN_008a6660, looks up its keyed node (bucket key & 3 at +0x74, chain +0x120, key +0x124), then walks that node's list … |
| `00a12660` | SyncAbilitySlotWithPlayer | likely | Resolves the player object with FUN_008a6660 and reads the 8-byte pair at +0x88/+0x8c; it then reads the same 8-byte pair from the object at this+0x54 (virtual… |
| `00a12890` | ItemActionBar_LookupAndDispatch | likely | Same 5-bucket action-bar lookup keyed by *param_2 (bucket table this+0x35434, chain +0x398, key +0x39c) followed by a call into FUN_00c44070. |
| `00a128e0` | ItemActionBar_LookupAndActivate | likely | Same 5-bucket action-bar lookup followed by a call into FUN_00c43d80 (slot lookup inside the bar and activation). |
| `00a12930` | ItemActionBar_ResetSelection | likely | Finds the action bar by id in the five-bucket table, then calls 00c43e00 for that bar. |
| `00a12980` | ItemActionBar_LookupAndVirtualCall | likely | Same 5-bucket action-bar lookup, then calls virtual slot +4 on the found node with (param_2[1], param_3, 0). |
| `00a129e0` | ItemActionBar_FindSlot | likely | Finds the action bar by id (*param_2, 5-bucket table at +0x35434) and then the slot node inside it whose key at +0x60 equals param_2[1], walking the slot bucke… |
| `00c49000` | SetField278AndNotify | likely | Setter: compares the argument with the dword at this+0x278 (dword index 0x9e), stores it when different and then calls this->vtable slot 4 (offset +0x10) to no… |
| `00958740` | HideActionProgress | speculative | When the linked owner points back to this, dispatches ActionProgress:Hide; then calls virtual slot 9 and conditionally invokes 00770270(param_2,0). |
| `009d03b0` | — | unknown | For slots with targeting bit 1 set, sends HandlerActionBar:setTargetingCanceled(slot) and clears that bit. |

### `world.targeting`

**FreeformActorManipulationActorClickHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c9b20` | FreeformActorManipulationActorClickHandler::DispatchTargetC… | confirmed | If the target pointer reached through +0x0c exists, calls its vtable slot +0x10 with this call's arguments 3 and 5 through 8. |
| `009c9b50` | FreeformActorManipulationActorClickHandler::AcceptInputKind | confirmed | Returns true for nonzero secondary flag or kinds 1, 0x1000, 8 and 0x40; returns false for other kinds. |
| `009c9ed0` | FreeformActorManipulationActorClickHandler::Ctor | confirmed | Constructs the TargetProcessor event-handler base, installs the actor-click vtable, and stores the target processor at +0x0c. |
| `009c9f80` | FreeformActorManipulationActorClickHandler::ScalarDeletingD… | confirmed | Restores the TargetProcessor base vtable and frees the allocation only when flag bit 0 is set. |

**FreeformActorManipulationTerrainClickHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c9ab0` | FreeformActorManipulationTerrainClickHandler::DispatchTarge… | confirmed | If the target pointer reached through +0x0c exists, calls its vtable slot +0x10 with this call's arguments 2 and 4 through 7. |
| `009c9ae0` | FreeformActorManipulationTerrainClickHandler::AcceptInputKi… | confirmed | Returns true for nonzero secondary flag or kinds 1, 0x1000, 8 and 0x40; returns false for other kinds. |
| `009ca240` | FreeformActorManipulationTerrainClickHandler::Ctor | confirmed | Constructs the TargetProcessor tile-event base, installs the terrain-click vtable, and stores the target processor at +0x0c. |
| `009ca2f0` | FreeformActorManipulationTerrainClickHandler::ScalarDeletin… | confirmed | Restores the TargetProcessor base vtable and frees the allocation only when flag bit 0 is set. |

**Target** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c017f0` | Target::ApplyGuidAndVector | likely | Parses a GUID, three coordinates, and optional values, constructs a vector target plus GUID target, calls 008c6170, and destroys both temporaries. |
| `00c01190` | — | unknown | Parses three numeric coordinates, optional radius and boolean, constructs a Target object from the vector, calls 008c61e0, then destroys the temporary target a… |
| `00c01320` | — | unknown | Parses six coordinates and optional boolean, constructs two Target objects, calls 008c6170 with both, destroys both objects, and pushes the result. |
| `00c01520` | — | unknown | Parses a GUID and optional radius and boolean, initializes a Target object with that GUID, calls 008c61e0, destroys the temporary target, and pushes the result. |
| `00c019d0` | — | unknown | Parses three coordinates and a GUID plus an optional boolean, creates temporary targets for both values, calls 008c6170, and destroys them. |

**TargetProcessorEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c9f30` | TargetProcessorEventHandler::Dtor | speculative | Restores the TargetProcessorEventHandler vtable at the base object. |
| `009ca2a0` | TargetProcessorEventHandler::Ctor | speculative | Installs the TargetProcessorEventHandler vtable at the object base. |

## Source shards

[[S0173|S0173]] (3), [[S0220 008a3ce0-008a8500|S0220]] (2), [[S0221 008a86b0-008b1d80|S0221]] (1), [[S0222 008b1e00-008b7730|S0222]] (10), [[S0223 008b7790-008bcf40|S0223]] (2), [[S0224 008bcf60-008c5290|S0224]] (5), [[S0225 008c52a0-008ca980|S0225]] (5), [[S0233 008ec2a0-008f0cc0|S0233]] (1), [[S0236|S0236]] (1), [[S0248 009392c0-0093d740|S0248]] (1), [[S0254 00954ec0-0095a710|S0254]] (1), [[S0255 0095a840-00960520|S0255]] (58), [[S0256 009606e0-00965340|S0256]] (11), [[S0257|S0257]] (1), [[S0275 009c8e90-009cdee0|S0275]] (10), [[S0276 009cdf80-009d3fe0|S0276]] (15), [[S0287 00a0c3a0-00a13140|S0287]] (16), [[S0288 00a13190-00a18a10|S0288]] (14), [[S0289 00a18a90-00a1dd30|S0289]] (2), [[S0292 00a29620-00a2fe20|S0292]] (18), [[S0293 00a2fef0-00a35000|S0293]] (12), [[S0294 00a35150-00a39a80|S0294]] (11), [[S0312 00a97450-00a9d360|S0312]] (2), [[S0321 00ac8340-00acd3f0|S0321]] (10), [[S0343 00b3f370-00b44f00|S0343]] (2), [[S0356 00b80ac0-00b84ce0|S0356]] (14), [[S0357 00b84e90-00b8a5f0|S0357]] (1), [[S0379|S0379]] (30), [[S0382 00bfcfe0-00c056a0|S0382]] (5), [[S0386 00c1f190-00c26aa0|S0386]] (1), [[S0392 00c42400-00c482e0|S0392]] (23), [[S0393 00c48300-00c4e490|S0393]] (8), [[S0395|S0395]] (6), [[S0424 00cf1430-00cf68d0|S0424]] (6), [[S0563|S0563]] (74), [[S0568|S0568]] (2), [[S0573|S0573]] (1), [[S0575|S0575]] (13), [[S0577|S0577]] (5), [[S0592|S0592]] (1), [[S0743|S0743]] (11), [[S0749|S0749]] (15)
