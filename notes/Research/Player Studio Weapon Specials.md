# Free Realms Player Studio Weapon Specials

Every Player Studio weapon has two named attacks: one that hits **opponents in front of you**, and a bigger **area attack** that hits everyone nearby (the "special").

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

## How they were matched

- **Names and descriptions:** certain. They come straight from the game's text file (`en_us_data.dat`, text ids 5103184 to 5103215 and 5102642 to 5102652).
- **Which weapon each pair belongs to:** strong, but not a direct data link.
  - The item data has no item-to-ability link, because the server assigned weapon abilities.
  - The text file stores each pair directly after the weapon names, in the same order as the weapons, and every pair's theme matches its weapon.
- **Damage, animations and effects:** not in the client files. Ability definitions lived on the old server, so these are unknown unless someone has video of the attacks being used.

See also: [[Player Studio Weapons]] for the weapons' creators, effects, swing trails and models.
