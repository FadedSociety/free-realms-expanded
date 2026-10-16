---
updated: 2026-10-16
---

# Combat Power-ups

Research for W153 (Sonnet, research only; no code or content changed). Part 1 is evidence with citations. Part 2 is a design proposal; everything in it that is not recovered is marked **(R)**. Raw extracts are in `lab/reports/combat-powerups-w153/` (file list at the end).

Related: [[Combat and Abilities]], [[Hewey's Escape]], [[Bixie Hive]], [[Owner Field Notes]].

## Summary

- A combat power-up is **not an item and not a buff the client knows about**. It is a pickup **actor** (an NPC with a `powerup_*.adr` model) that the server spawns, plus a **server-side ability** that goes into **job-toolbar slot index 2** (the key `3` button) when the player picks it up. Using it is an ordinary `36/10` ability request for bar 1, slot 2.
- The shipped client has **no ability, effect or drop table**, so durations, effect sizes and drop chances are **not in client data**. The locale still holds the designers' ability-row names and notes, which show how the original server was organised (per-tier abilities on the enemy, two reactive procs, "standard weights"). The weights themselves were not found anywhere.
- **No community source has drop rates** (wiki, Allakhazam mirror, FreeRealms-Legacy, Sanctuary, the video archive). Chances must be reconstructed (R) and kept live-tunable.
- Two things need a decision from the owner before building (section "Open questions"): the key-3 slot collides with existing job-ability content, and the Flame Wave description differs between eras.

## Part 1. Evidence

### 1.1 What a power-up is in the client

| Fact | Evidence | Strength |
|---|---|---|
| The player uses a power-up with key `3`. Key `3` is `ActionBarSlot3` (action 21, `0x16` in the binding table), which dispatches **slot index 2** through `008a7eb0` (`DispatchActionBarSlot`, confirmed) then `008a7e40` mode 1, which goes to `009cf430` (`DispatchActionBarCastOrFailureReason`, confirmed) and `00a36a70`/`00a35600`/`00a35150` (likely) to send `36/10`. | [[Native Combat Controls]]; `nmq show 008a7eb0 008a7e40`; [[Combat and Abilities]] "Using an ability"; `Native Map/Default Key Bindings.md` row 21 | client, confirmed |
| `36/10` carries `i32` bar type (1 job, 2 item, 3 minigame), `i32` slot, `i32` target mode, then a target id or four floats. A self cast (selectors 1 with 2 or 4) sends at once with the player's own id. | `00a342e0` pseudocode, guide "Ability request (36/10)" | client, pseudocode |
| The client's own tutorial says "Use the others by clicking on the **third slot** where the arrow is pointing." The Lua first-time-event arrow for power-ups is drawn on the game dock (`FirstTimeEvent.PowerUpFTE`, `ScriptsBase.lua:41390`, arrow at `gdX + 325`). | locale hashes 960659507, 2856968219, numeric 388124; `lab/decompiled/client-ui/ScriptsBase.lua` | client |
| In the Hewey's Escape video the combat dock shows three round ability orbs, then a faded fourth circle, then the potion slots. Orbs 1 and 2 have number badges. The **third orb has no badge, lights up with an icon while a power-up is held**, and goes dark after use (frames 00438, 00452, 00456, 00460 of `fktAs_GRrjM`; crop saved as `hewey-bar-crops.png`). Together with the wiki text this puts the held power-up in job-toolbar slot index 2. | `lab/video-archive/out/fktAs_GRrjM/frames/` | video, read by eye |
| Held-power-up state is **server data**. The job toolbar is replaced by `36/5 SetDefinition` (handler `00a35cc0` case 5, rebuilds `BaseClient.ActionBar`), definitions come by `36/13`. `38/25 UpdateActionBarSlot` only edits the **item** and **minigame** bars (`009392c0` case `0x19`), so it cannot put an ability on the job bar. | guide "Bar refresh"; `nmq brief 009392c0 00a35cc0` | client, confirmed |
| Locale 1011626935 / numeric 421005: **"power ups - add super shield to bar"**. A power-up pickup is therefore an ability whose effect is to add an ability to the bar. | locale | client locale (dev note) |
| "You can have one power up active at a time." is shipped as a tutorial page (hashes 4186384284, 3152868716, 3222513765). | locale | client locale |
| "Power-Ups go away when you use them, but they always help you and they don't even use energy!" (hash 3939331069). So **no energy cost, consumed on use**. | locale | client locale |
| Health and Energy power-ups are "used automatically" (they never occupy the slot). Flame Wave, Earth Shard, Super Shield (and the quest-only Speed Boost) are stored. | locale 960659507 / 388124 | client locale |

**The pickup actor.** `Resources/Models.txt` has the models (id, file):

| Kind | Model id | File | Particle | Notes |
|---|---:|---|---|---|
| Health | 736 | `powerup_health_buff.adr` | `powerup_red.xml` | one model; big vs small heart is not a separate model (see 1.2) |
| Energy ("mana") | 737 | `powerup_mana_buff.adr` | `powerup_blue.xml` | |
| Flame Wave | 1949 | `powerup_flame_wave.adr` | `powerup_orange.xml` | texture `icon_flame_wave.dds` |
| Earth Shard ("quake") | 1950 | `powerup_quake.adr` | `powerup_purple.xml` | texture `icon_quake.dds` |
| Super Shield | 1951 | `powerup_super_shield.adr` | `powerup_red.xml` | texture `icon_super_shield.dds` |
| Speed Boost | 738 (also 578, race 99) | `powerup_speed_buff.adr` | `powerup_orange.xml` | quest power-up |
| Event and other | 745, 746, 747, 842, 843, 2058, 3299 to 3322 | damage-aoe, damage, defense, dogbone, time, valentines, vampire, werewolf | | holiday and Rumbledome sets; out of scope |

`build_asset_graph.py --who model:<id>` shows every one of these actors uses two animation files, `powerups_icons_spawn.gr2` (slot `com_spawn`) and `powerups_icons_bob_spin.gr2` (slot `loc_stand`), skeleton `powerup.dsk`. So a pickup is an ordinary actor that plays a spawn animation then bobs and spins, which matches the wiki: "A symbol will float in the air above their unconscious body." **Asset caveat:** the graph reports `powerup_super_shield.dme` and `icon_super_shield.dds` as *missing* from the extracted pack inventory, although both are listed in `Assets_manifest.txt` (hashes 158128089, 2618219093). Check asset delivery before a native test.

**What the client shows for "holding one".** The slot-3 orb with the power-up's icon and name (the slot record has icon id, tint id, name locale id, energy cost). Client icon ids from the Lua first-time-event: 17348 health, 17349 energy, 17350 flame wave, 17351 earth shard, 17352 super shield, 17353 speed burst (64 px, `TestDialog6Icons`), and ability-style icons 27632 `icon_abil_ninja_flame_wave_128`, 27633 `icon_quake_128`, 27634 `icon_super_shield_128` (`PowerUpIcons`). The tutorial pages list "Health, Energy, Flame Wave, Earth Shard, Super Shield" and a sixth "Speed Boost - Increases your speed." (numeric locale 388125 to 388130, `FtesPowerUp1..6`).

**What is not recovered: the pickup packet.** No client function, command class or locale line names a dedicated pickup request (`nmq command` for pick/loot/powerup/collect found only unrelated classes; `nmq strings 'pickup|loot'` finds only soccer, kart, ship and housing). The wiki says "To hotkey a power-up, click it or walk into it." So click goes through the normal actor interaction path (`26/10 InteractionSelect`, menu from `26/9`; interaction key `X`, id 45) and "walk into it" must be a server proximity check on movement. Treat both as (R) until a native run shows what the client sends when the player stands on one.

**Other effects the server can already use to show it.** Heals and DoT ticks show as `35/35 HitPointModification` (positive delta prints `+N`); health is `38/1`, energy `38/13`; buffs appear through effect tags `38/16` (icon id, duration, name; zero icon ignored); composite effects (particles/sound) by id through `35/16`. The pickup sound is `SFX_PowerUp_Pickup` (composite 15509, sounds 8008, 8025, 8071); `PFX_magic-starburst_*_powerup` are composites 40 and 58 and `..._powerup_instant` is 15507 (`ActorCompositeEffectDefinitions.xml`). Super Shield's invulnerability is status bit 8 "Nonattackable" via `35/20`; the client prints floating text "Invulnerable" (locale 7930) when it turns on.

### 1.2 The five kinds, as the client and wiki describe them

| Kind | Player-facing text (client 2014 tutorial) | Wiki (Dec 2009) | Locale ability notes |
|---|---|---|---|
| Health | "Health - Gives you health." Used automatically. | Big Heart heals everyone in your group; Small Heart heals a low amount of your own health. | "new combat spawn health power up. **Heals 50% max hp, works across all tiers**" (ability "Spawn Health Power Up 2", numeric 419605/419606); "**50% heal power up**" (419620); "**combat - 15% health power up**" (103703); "reactive proc from ability 4550. **Spawns tiny health power ups**" (420826, "Power Up Small Health" 420825). Reading: big heart = 50% max HP, small = about 15% (R reading of the notes). |
| Energy | "Energy - Replenishes energy." Used automatically. | Replenishes energy. | "**Mana Power up. Heals 100 mana.**" (hash 1788006282; "Mana Power Up" 103694). Video shows 100/100 energy, so 100 is a full bar. |
| Flame Wave | "Damages nearby opponents." | Instantly damages all surrounding enemies. | Numeric 419611 "Flame Wave Power Up", 419612 "Fire Wave Buff", 419613 "fire wave power up buff - tier 3", 419614 "Fire Wave", **419615 "Lights opponents near you on fire, slowly damaging them and lowering their run speed."** So the late design is an area burn plus slow, not a pure instant hit (conflict, see Open questions). Frame 00456 shows a fire ring and a `-279` hit on the boss. |
| Earth Shard | "Damages and stuns opponents." | Damages all enemies around you and briefly stuns them. | 419621 "Earth Shards Buff", 419622 buff tier 3, 419623 "Earth Shards", **419624 "Launches shards of rock at nearby opponents. Damaging opponents near you and stunning opponents around you."**, 419625 "Earth Shards Power Up". Agrees with the wiki. |
| Super Shield | "Makes you invulnerable." | Increases your speed and makes you invulnerable for a few moments. | 103799 "Super Shield Power Up" + **103800 "Super shield prevents you from being hurt and increases your run speed for a few seconds."**; 103700 "Super Shield Buff"; 421004 "Super Shield Power Up Buff" + 421005 "power ups - add super shield to bar". |
| Speed Boost | "Increases your speed." | "Some quests will make power-ups appear, such as Speed Boost." Quest only. | 103695 "Speed Power Up"; 70215 "Slow Power-Up" (the opposite, hazard). |

### 1.3 Definitions, tiers and effect sizes: what the client does and does not have

- **Not in the client:** any ability, effect, damage or drop table. The guide already established this ([[Combat and Abilities]], "What the client ships and what it does not"). The executable's `AbilityDefinition` text-row constructors have no caller; definitions arrive by `36/13`. There is **no combat power-up item** in `lab/reports/item-catalogue-20261015/catalogue.json` (72 MB, searched for `power up`, `flame wave`, `earth shard`, `super shield`: no hits) and none in the community item definitions (`ClientItemDefinitions.json`: only weapon names such as "Ninja's Blade of Flame Wave", which is the Ninja weapon skill, a different thing). So a power-up is not an inventory item.
- **Effect scaling the client can express** (`AbilityEffectTypeSettings.txt`): type 1 direct damage, type 2 stat modifier with `Param1` = stat id (stat 2 is Movement Speed, row 35 "run speed N times"), type 3 instant restore (rows 3 heal, 4 energy). `ChanceToOccur`, `DurationSeconds`, `MaxCharges` and `Param1..3 TunedLow/TunedHigh` exist per effect; the numbers come from the server.
- **Tiers.** "Tier 4" in the locale means **battle tier 1 to 5** (the same five difficulty tiers the combat wiki lists: levels 1, 3, 5, 9, 13). The locale holds one set of designer rows per tier. Numeric ids (name then dev note):

| Rows | Ids | Meaning |
|---|---|---|
| "Power Ups N" / "combat power ups - tier N. Put this ability ID on the npc 'ON_SPAWN_ABILITY'" | 420322 to 420331 (tier 3 at 419603/419604) | The per-tier ability a designer attaches to an enemy. |
| "Other Power Ups Tier N" / "combat power non-hp power ups - tier N" | 420332 to 420339 (tier 3 at 419607/419608) | Picks a non-health power-up for that tier. |
| "fire wave power up ability - tier N" and "... buff - tier N" | 420369 to 420375 (ability), tier 3 buff 419613 | Flame Wave per tier. |
| "earth shards power up ability - tier N" and "... buff - tier N" | 420353 to 420367, tier 3 buff 419622 | Earth Shard per tier. |
| "combat - power up on spawn ability. Put this in the ON_SPAWN_ABILITY of any combat npc that should drop power ups" | 420830 | The base spawn-time ability. |
| "combat - power up health / other - reactive proc from ability 4550" | 420826, 420828 | **Two separate reactive procs**: one spawns tiny health power-ups, one spawns other power-ups. |
| "combat generic - spawns a random power up using standard weights" ("Spawn A Power Up") | 426422/426423 | A random pick **with a weights table that the client does not hold**. |
| "combat generic - spawn a power up" / "- spawn health power up" / "- spawn a health power up on death" / "- guranteed power up" / "- guranteed mana power up on death" / "garunteed to drop a power up" / "Spawn Non ability power ups" | hashes 873650096, 3329609709, 4236512891, 4178689781, 1281729524, 1774309798, 1236747222 | Variants for scripted or boss enemies. |
| "Racial - Frogglekin Scavenger on death with power up" (35208895); "Birthday Combat ... Cakenstein Power Up (Tier 5 version of the Cake power up for cakelings)" (2544037890); "Valentines Day Combat Drop Power Ups" (2148261419) | | Per-enemy and per-event drops. |
| "vday combat - npcs in vday combat encounters do not drop **activatable** power ups, only hp and mana." / "vday combat - spawns only health and mana power ups" | 687016359, 2411507660 | "Activatable" = the stored kinds; confirms health and energy are the auto kinds. |

  Reading (R): each combat enemy gets a tier-N power-up ability at spawn; when it is defeated, two reactive procs run, a health proc and a non-health proc, and the non-health pick uses a weight table. The **tier selects the strength of Flame Wave and Earth Shard**; Health works the same on every tier. This matches the owner's field-note wording "earth shards power up buff - tier 4".
- **Durations and magnitudes.** None recovered except the percentages above (50%, 15%, 100 energy) and the player text "for a few seconds" (Super Shield) and "briefly" (Earth Shard stun). The wiki says "a few moments".
- **Other eras.** Older and platform variants exist in the same locale and conflict with the 2014 PC text: "Power up abilities sometimes drop from defeated monsters. These can be used by hitting the **number 4 hotkey**." (hash 2772042684, early), and the PS3 page "The other items are used to **fully charge your Advanced or Special Attacks**." (numeric 430267). Our target is the 2014 PC behaviour: third slot, key 3.

### 1.4 Drop chances

- **Client:** none. `nmq strings` and the locale search found no chance, weight or drop-rate number for combat power-ups. The only weight reference is the unreadable "standard weights". The server held them.
- **Wiki and community:** the fan wiki (Dec 2009), the Allakhazam mirror (`camelot.allakhazam.com/wiki/Combat_(FR)`) and web searches give only: "Power-ups are items that drop off of enemies during combat. A symbol will float in the air above their unconscious body." No chance, count or tier. FreeRealms-Legacy and Sanctuary (and the combat PR research) contain no combat power-up code or data; Legacy `Npcs.json` only lists dog-bone and werewolf power-up models as spawnable NPC rows. The video archive's reviewed frame records (batches B0001 to B0048 and `vlm.jsonl`) contain only the Hewey tutorial, the Bixie Queen lines and unrelated minigames.
- **Scripted spawns that are evidenced:**
  - **Hewey's Escape:** Harold: "Pick this up for a special attack!" (frame 00426 shows a **flame icon on the ground with a green arrow**) then "Pick up the power up and press [3] on your keyboard to use it." (locale 1441159516). So Hewey's first power-up is a scripted Flame Wave, not a roll. A tutorial panel lists the five kinds (locale 960659507 and 388124 to 388130).
  - **Bixie Hive:** the Queen "I'll create some power ups for you to use!" and "Stay close, dearie, and use my power ups!" (locale 3813749571, 3859460458; video `5_U-uAzDuR4/00160..00192`). A friendly NPC spawns them on the wave script. Effects unknown.
  - **King of the Ring:** "Use the Flame Wave power-up on Mongo." (716920982) in the Brawler quest, again a scripted Flame Wave. A quest "Get the Flame Wave power-up." exists (2026646197) and "Click the game dock button that shows the Flame Wave power-up to use the power-up attack!" (1188590723).
  - **Cakenstein:** weak to the Fire Power-Up (wiki); cakelings drop a tier-5 cake power-up.
  - **Rumbledome duel:** "spectators will occasionally throw powerups into the arena" (locale 2826525430); eight spectator kinds (Health, HP, MP, Damage, Defense, Offense, Power, Uber). A separate system.
  - Kart/derby/soccer/match-three "power-ups" are unrelated minigame systems (key `UsePowerup` = Ctrl is the kart one, `33/19`).

### 1.5 Known limits of this evidence

Nothing was observed on the wire (no capture of a power-up spawn or pickup exists in `lab/captures`), the server tables are gone, and the only proof of what the slot looks like is one video. Every number below the line is a design choice, not a recovery.

## Part 2. Design proposal (R unless stated)

### 2.1 Shape

Reuse the existing encounter, enemy and ability code; add one small data file and one small module.

1. **Data-driven drops.** New `data/content/power_ups.json` (loaded and validated like `job_abilities.json`, reload with `realm reload power_ups`; bounds checked in `content_check.py`). Enemies and field enemies reference a table by id in their existing `reward` block, so the 55 definitions in `enemies.json` and the field enemies need only one optional field.
2. **Two independent rolls per kill, mirroring the two original procs** (R): a **health roll** (pool: small heart mostly, big heart rarely) and an **other roll** (pool: energy, flame wave, earth shard, super shield, weighted). Each roll has its own chance. This keeps health available to every fight and lets designers tune the stored kinds separately.
3. **Spawn** a pickup actor at the kill position using the existing `spawn_actor`/`remove` helpers in `encounter.rs` with the model ids in 1.1 and a server-side despawn timer (R, 30 s default, so actors cannot pile up).
4. **Pickup** by proximity (server, 1.5 m, checked on the existing movement/tick path) and by the interaction route (once a native run shows what the client sends). Health and energy apply at once and the actor goes. The stored kinds go into slot index 2 only if empty (R: otherwise the pickup stays and the player gets the locale line "You can have one power up active at a time." as a failure message; native check needed).
5. **Hold** = a field on the player's combat state (not a bag item), so it is not saved across logout (R) and cannot be traded, dropped or duplicated.
6. **Use** = a `36/10` for bar 1, slot 2 on a self-cast definition. The server resolves the held kind, runs its effect through the same executor that runs job abilities, clears the slot (36/13 + 36/5 with an empty slot 2) and sets no cooldown, no energy cost.
7. **Tier** = the battle tier of the fight (dungeon `difficulty` 1 to 5, field battle `difficulty`), scaling Flame Wave and Earth Shard damage with the existing weapon-tier curve (x1.75 per tier, final-era balance). Health and energy do not scale.

### 2.2 Effect mapping on code that exists

| Kind | Effect | Existing building block |
|---|---|---|
| Health, small | heal self 15% of max health (R from the "15%" note), applied on pickup | `ability.heal` path in `encounter.rs` (scales with the HP curve); `35/35`-style delta + `38/1` |
| Health, big | heal self and group members 50% max health (note: "Heals 50% max hp") | same; group delivery needs the cross-player route in `osfr-gateway.rs` (lead file); solo falls back to self |
| Energy | +100 energy (clamped to max) on pickup | `self.energy` field and `38/13` builder in `encounter.rs` |
| Flame Wave | caster-centred area hit on every enemy in radius (R 10 m, up to 16), then a burn | `JobEffect::AreaDamage { centered_on_caster: true }` + `DamageOverTime`; the "slow" half has no building block (needs a new `JobEffect::Slow`, stat 2 via effect tag 38/16); ship without slow first |
| Earth Shard | same area hit plus stun (R 2.5 s) | `AreaDamage` + `Stun` (`skill_stuns`, animation 1401 `com_stunned`) |
| Super Shield | caster invulnerable plus run speed up (R 5 s, +30%) | `JobEffect::Buff` cannot do it: `damage_taken_multiplier` is bounded to 0.1 to 3.0 in `JobEffect::valid`, so invulnerability needs either a new `invulnerable` flag or a `0.0` allowance; the client side is status bit 8 via `35/20` and a speed effect tag |
| Speed Boost | quest only; run speed up (R 8 s) | same speed effect as Super Shield; scripted spawn only, no roll |

### 2.3 Proposed `power_ups.json` (shape only, values are a starting point to tune)

```jsonc
{
  "schema_version": 1,
  "kinds": {
    "health_small": { "model_id": 736, "auto": true,  "heal_pct_max": 15,  "group": false, "source": [ ... ] },
    "health_big":   { "model_id": 736, "auto": true,  "heal_pct_max": 50,  "group": true,  "scale": 1.5 },
    "energy":       { "model_id": 737, "auto": true,  "energy": 100 },
    "flame_wave":   { "model_id": 1949, "auto": false, "name_id": 419611, "icon_id": 27632,
                      "area": { "radius": 10.0, "max_targets": 16 }, "damage_by_tier": [1,1.75,3.06,5.36,9.38],
                      "burn": { "multiplier": 0.25, "duration_ms": 4000, "interval_ms": 1000 } },
    "earth_shard":  { "model_id": 1950, "auto": false, "name_id": 419625, "icon_id": 27633,
                      "area": { "radius": 10.0, "max_targets": 16 }, "stun_ms": 2500 },
    "super_shield": { "model_id": 1951, "auto": false, "name_id": 103799, "icon_id": 27634,
                      "duration_ms": 5000, "invulnerable": true, "speed_pct": 30 }
  },
  "tables": {
    "standard": {
      "health_roll": { "chance": 0.12, "pool": { "health_small": 85, "health_big": 15 } },
      "other_roll":  { "chance": 0.10, "pool": { "energy": 30, "flame_wave": 25, "earth_shard": 25, "super_shield": 20 } }
    },
    "elite": { "health_roll": { "chance": 0.25, ... }, "other_roll": { "chance": 0.25, ... } },
    "boss":  { "guaranteed": ["health_big"] }
  },
  "pickup": { "ttl_ms": 30000, "radius": 1.5, "max_alive_per_instance": 16 }
}
```

Name and icon ids above are the evidenced locale and image ids (numeric Global.Text ids and `Images.txt` ids from 1.1 and 1.2). Radius, durations, chances, weights, speed percent and the burn tuning are all (R). The health and energy numbers (15%, 50%, 100) are read from the locale dev notes (Part 1.2).

**Sanity bounds** (enforced at load, in the style of the existing `JobEffect::valid`; no value is pinned by a test): each roll chance 0 to 0.5; weights 0 to 1000 with at least one non-zero; pool kinds must exist; heal percent 1 to 100; energy 1 to 100; radius 0.5 to 30; max targets 1 to 16; stun 500 to 10000 ms; burn duration 500 to 30000 ms; shield duration 500 to 30000 ms; speed percent 0 to 100; pickup ttl 5000 to 120000 ms; radius 0.5 to 5; alive-per-instance 1 to 32. A bad file keeps the previous table and logs, so a live edit cannot crash a fight.

**Per-enemy hook.** Add optional `power_up_table: "standard"` to an enemy's `reward` (default none for training dummies, summoned minions and the Troll Summoner's scoutlings so they cannot be farmed; bosses use `"boss"`). Scripted spawns are a separate list on a dungeon or quest: `{ "at": "wave_start", "kind": "flame_wave", "near": "player" }` for Hewey, a repeat per wave for the Bixie Queen, and the King of the Ring Flame Wave hint (all (R) positions and timings).

**Tuning live.** `realm reload power_ups` (same as `reload abilities`), plus `realm powerup <kind>` to spawn one at your feet and `realm powerup roll <table> <n>` to print the distribution of n rolls. Both are developer tools on the existing role gate; neither ships a new network subtype.

### 2.4 Build-slice plan with gates

| # | Slice | Files (owner needed) | Gate |
|---|---|---|---|
| 1 | Schema, loader, validation, pure `roll(table, rng)` with a seeded RNG | new `power_ups.rs`, `data/content/power_ups.json`, `content_check.py` rule | unit tests: bounds reject bad files; deterministic roll; large-sample distribution within tolerance of the weights; `content_check.py --strict` 0 errors; `rustfmt --check` |
| 2 | Spawn on kill, despawn, pickup by proximity, auto kinds (health, energy), held slot state | `encounter.rs` kill branch (`if hp == 0`, about line 3243) and tick; `enemies.json` `reward.power_up_table` | encounter tests: kill spawns an actor with the model id; only one stored kind held; refusal keeps the pickup; ttl despawn; `osfr-bot` script on a scratch gateway (`--studio-port 0`) that kills wolves and expects the spawn packet and `38/1`/`38/13` changes |
| 3 | Slot display and use (`36/13` + `36/5`, `36/10` bar 1 slot 2), Flame Wave, Earth Shard, Super Shield | `opening_abilities.rs` toolbar builder (reuse `activation`), `abilities.rs` `JobEffect` (+ `Invulnerable`, optionally `Slow`), `gameplay.rs` slot routing | tests for each effect on a fixture fight (damage by tier, stun applied, damage taken 0 for the shield duration); full cargo suite; `tests/app_fuzz.rs` coverage (no new subtype, so no new seed); native acceptance run on the cave wolves (first test: does the client keep the slot lit and accept a mid-fight `36/5`) |
| 4 | Scripted spawns and group heal | dungeon JSON, `osfr-gateway.rs` cross-player routing (lead) | Hewey flame wave appears at the scripted beat; big heart heals a second bot in the group |
| 5 | Studio and `realm` tuning | `studio.rs` / realm commands | edit chances live and watch the roll output change without restart |

**Shared-file patches (not applied; the owner of those files should take them).** (a) `encounter.rs`, kill branch: after `self.state.world_enemies`/field state update, call `power_ups::roll` with the enemy's table and push the pickup spawn packet into `out.replies`; failing test: killing a field enemy with `power_up_table: "boss"` and a guaranteed pool yields exactly one spawn reply with model id 736. (b) `abilities.rs`: extend `JobEffect::valid` for an invulnerable buff; failing test: a Super Shield buff with `damage_taken_multiplier: 0.0` currently fails validation. (c) `gameplay.rs`: route `36/10` bar 1 slot 2 to the held power-up before the job-ability lookup; failing test: with a held Earth Shard, slot 2 stuns enemies in range and empties the slot.

**Security decisions (for the lead's review).** The client never names a power-up or an effect: the pickup request is validated by server distance to a server-owned actor; the `36/10` slot and target id are re-resolved against the held-state, and a player with nothing held fails closed with the existing refusal. Chances use a server RNG only. Pickup actors per instance are capped and time-limited. Table values are bounded at load so a bad live edit cannot disable the server. No power-up state is persisted or tradable.

## Open questions

1. **Slot collision (needs the owner).** `data/content/job_abilities.json` already puts job abilities on toolbar indices 1 to 5 for every combat profile (for example profile 11 has index 2 at level 1 and profile 12 has index 2 at level 5), and the guide notes Warrior, Wizard and Medic have four level-gated abilities. Key 3 is index 2, so all six jobs collide with the power-up slot at some level. Those slot numbers are (R) in our content, not recovered. The Hewey video shows only three orbs plus a faded fourth circle for a low-level Wizard. Options: (a) reserve index 2 for the power-up and move gated abilities to 3 onward, (b) shift only when a power-up is held, (c) use the faded fourth circle. The client also has `SelectAlternateActionBar` (action 18, Shift) which may be where the gated abilities lived. Recommendation: (a) after one native check of the dock at job level 5 or higher.
2. **Flame Wave: instant or burn?** The 2014 client tutorial says "Damages nearby opponents", the 2009 wiki says instantly, but the late ability text says a burn with slow. Proposal: hit then burn, slow later.
3. **Mid-fight `36/5`.** Replacing the whole toolbar while the client is fighting might reset the `AbilityProcessor` pending flags. Unverified; slice 3 starts with that native check.
4. **Click versus walk-over.** The client packet for a click on a pickup is not recovered.
5. **Shared or personal pickups** in a group (R: shared, first come).
6. **Super Shield mesh and icon** missing from the extracted pack set (1.1 caveat).

## Evidence index

`lab/reports/combat-powerups-w153/`: `locale-powerup-lines.tsv` (all 249 distinct locale rows matching power-up, by hash), `locale-dev-notes-combat.tsv` (744 combat-related dev-note rows), `powerup-name-ids.tsv` (hash to numeric Global.Text id for the player-facing names), `numeric-id-neighbours.tsv` and `ability-row-neighbourhoods.tsv` (adjacent locale rows that show each ability's name, note and description), `hewey-bar-crops.png` (five crops of the dock from the Hewey video). Sources: client `Resources/Models.txt`, `Images/Images.txt`, `FirstTimeEvents.txt`, `CodeStringMappings.txt`, `ActorCompositeEffectDefinitions.xml`; `lab/decompiled/client-ui/ScriptsBase.lua`; the wiki database `lab/references/freerealms-wiki/wiki.sqlite` (Combat, Cakenstein, Festival of Hearts pages); web: camelot.allakhazam.com/wiki/Combat_(FR).
