# Free Realms Player Studio Weapons

There are 11 Player Studio weapons in the game files. Each one's description ends with "This weapon will level up with you, increasing your power in battle!" and "Created by ___ for Player Studio!".

| Weapon | Creator | Job | Effect on the weapon | Swing trail |
|---|---|---|---|---|
| Ice Axe | Ashley | Warrior | Frost: a white ice particle loop and 4 icy sounds (effect 15449) | Gray (5485) |
| Snowflake Wand | Ashley | Wizard | The same frost effect as the Ice Axe (15449) | None |
| Lightning Blade | Shyshadow | Warrior | Electric: a blue crackle loop and 3 zap sounds (5728) | Yellow (5694) |
| Gleam Energy Fist | Shyshadow | Brawler | Blue magic sparkles (15010) | Blue (5686) |
| Gloam Energy Fist | Shyshadow | Brawler | Purple arcane trail (16095) | Purple (5691) |
| Lunar Blade | Shyshadow | Ninja | Gold stardust trail; it reuses the Herbalist "glitterblossom" trail (15859) | Gray (5485) |
| Wing Blade | Shyshadow | Medic | None | Gray (5485) |
| Bellringer | MasterMagnus | Brawler | None (a shiny gold metal texture) | Yellow (5694) |
| Precursor Energy Blade | Shyshadow | Ninja | None (a glow texture on the model instead) | None |
| Hole-In-One Golf Driver | Firehawk894 | Brawler | None | None |
| The Kingmaker | CoffeeOutlaw | Warrior | None (a golden texture) | None |

The swing trails are the game's standard colored trails (`WFX_trail_default_gray`, `_yellow`, `_blue` and `_purple`), linked through each item's weapon-trail field. The first batch of weapons (item ids 78711–78717) has them. The Snowflake Wand and the later batch (79020–79022) don't. Effect 15449 (frost) is also used by one non-Player Studio weapon, the Elitist's Sword.

## Do they have unique animations?

**No.** That's why nobody could find them.

- Each weapon's model file (`.adr`) holds only its mesh, its textures and a right-hand attach point (`R_WEAPON`).
- The swing animations come from the weapon's **job**. Each item's class sets a wield type, and the wield type picks that job's normal attack set (animation group `com_swing`):
  - Warrior, Brawler, Ninja and Medic weapons use the standard one- and two-hand slash attacks (`com_1hs_attack_01` to `04` and `com_2hs_attack_01` to `04`).
  - The Snowflake Wand uses the Wizard cast animations (`com_cast_special_03` to `05`).

Players only designed the model and textures. The game added the shine by reusing existing effects: the standard colored swing trails, plus effects such as the Herbalist trail on the Lunar Blade and the frost shared by the Ice Axe and Snowflake Wand.

## Special attacks

Every weapon has two named attacks: one that hits **opponents in front of you**, and a bigger **area attack** that hits everyone nearby (the "special").

| Weapon | Front attack | Area special |
|---|---|---|
| Ice Axe | **Freezing Strike**: "Attack all opponents in front of you with icy fury!" | **Snowstorm**: "Unleash the full power of winter to damage all nearby opponents." |
| Gleam Energy Fist | **Gleam Smash**: "Smash opponents in front of you with Gleam energy!" | **Gleam Explosion**: "Unleash the power of Gleam energy upon all opponents around you." |
| Gloam Energy Fist | **Gloam Smash**: "Smash opponents in front of you with Gloam energy!" | **Gloam Fire**: "Summon Gloam fire to damage all opponents around you." |
| Lightning Blade | **Electric Punch**: "Punch opponents in front of you with electrical power!" | **Lightning Blast**: "Blast out lightning and damage all opponents in front of you." |
| Lunar Blade | **Moon Slice**: "Slice opponents in front of you with this lunar blade!" | **Celestial Spin**: "Send a wave of celestial strikes to damage all opponents around you." |
| Wing Blade | **Winged Slice**: "Slice opponents in front of you with this winged blade!" | **Feather Spin**: "Send a wave of feathered strikes to damage all opponents around you." |
| Bellringer | **Ringing Smash**: "Deliver a ringing smash to oppponents in front of you." (sic) | **Tolling Bell**: "Delivers a bell-tolling attack to all nearby opponents." |
| Snowflake Wand | **Magic Snowball**: "Fire a magic charged snowball at all opponents in front of you." | **Ice Storm**: "Unleash a wave of icy destruction on all nearby opponents." |
| Hole-In-One Golf Driver | **Strong Layup**: "Deliver a robust smash to the opponents in front of you." | **Power Drive** (the game has no description text for it) |
| The Kingmaker | **Regal Strike**: "Strike down all opponents in front of you." | **Royal Power**: "Destroy all nearby opponents with your Royal Power." |
| Precursor Energy Blade | **Energy Slash**: "Slash at opponents in front of you with the energy of the Precursors." | **Energy Storm**: "A surge of Precursor energy damages all nearby opponents." |

Two exceptions to the pattern: the Lightning Blade's special (Lightning Blast) also says "in front of you", and the Golf Driver's Power Drive has no description in the game text.

**How they were matched:** the client's text file stores each pair directly after the weapon names, in the same order as the weapons (text ids 5103184 to 5103215 for the first batch, 5102642 to 5102652 for the second), and every pair's theme matches its weapon. The item data itself has no item-to-ability link, because the server assigned weapon abilities. So the pairing rests on order and theme (strong, but not a direct data link).

**Their damage, animations and effects aren't in the client files.** Ability definitions were server-side, so the numbers and visuals for these attacks are unknown.

## Item ids

| Item id | Weapon | Model | Item class |
|---|---|---|---|
| 78711 | Ice Axe | `axe_ar_ag_weapon_iceaxe.adr` | 17 (Warrior) |
| 78712 | Gleam Energy Fist | `fist_ar_ag_weapon_gleamenergy.adr` | 71 (Brawler fist) |
| 78713 | Gloam Energy Fist | `fist_ar_ag_weapon_gloamenergy.adr` | 71 (Brawler fist) |
| 78714 | Lightning Blade | `fist_ar_ag_weapon_lightning.adr` | 17 (Warrior) |
| 78715 | Lunar Blade | `fist_ar_ag_weapon_lunar.adr` | 6 (Ninja) |
| 78716 | Wing Blade | `fist_ar_ag_weapon_wing.adr` | 28 (Medic) |
| 78717 | Bellringer | `hammer_ar_ag_weapon_bellringer.adr` | 5 (Brawler) |
| 78718 | Snowflake Wand | `wand_ar_ag_weapon_snowflake.adr` | 10 (Wizard) |
| 79020 | Hole-In-One Golf Driver | `club_ar_ag_weapon_golfdriver.adr` | 5 (Brawler) |
| 79021 | The Kingmaker | `sword_ar_ag_weapon_kingmaker.adr` | 17 (Warrior) |
| 79022 | Precursor Energy Blade | `sword_ar_ag_weapon_precursorblade.adr` | 6 (Ninja) |

## Sources

- **Special attacks:** the client locale text (`en_us_data.dat`), text ids 5103184 to 5103215 and 5102642 to 5102652.
- **Swing trails:** each item's weapon-trail field, found with the asset graph (`python -I project/scripts/build_asset_graph.py --who item:78711`).
- **Names, creator credits, item classes and effect links:** the client item definition table (`ClientItemDefinitions`, as dumped by the community Legacy project) and the client locale text.
- **Effects:** `ActorCompositeEffectDefinitions.xml` in the client.
- **Wield types:** `ItemClasses.txt` and `WieldTypeAnimationMappings.txt`.
- **Animations:** `AnimationGroups.xml` in the client.
- **Models:** the weapon `.adr` files in the client asset packs.
- **More items:** the game files hold 41 more Player Studio items, mostly housing items (gazebo, laptop, pots and so on) and some clothing.
