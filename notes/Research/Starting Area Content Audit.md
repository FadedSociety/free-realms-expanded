# Starting Area Content Audit

2026-10-08. Read-only audit of the supplied client, recovered locale and pinned community resources. This records surviving content, not completed gameplay.

The supplied 2014 build contains the NPX 4.0 opening: **Briarwood Caverns → Farnum Farmstead → Cobblestone Village → Queen Valerian**. This is the strongest candidate for the requested opening playthrough. The present research character starts at Sanctuary's warpstone and already owns items and quests; it is not a fresh-character baseline.

The client itself defines `Newbiezone` in `FabledRealmsAreas.xml`, area6093, as a sphere centered at (-2057,-45,430), radius415, with Hero's Journal region override1. This surrounds the farm and village portion. Legacy's new-character position is (-1904.883,-39.7098,412.6024). The client retains older introductions too: Sacred Glade has explicit retired markers, while the Merigold/Highroad dialogue represents another introduction. Combining all of them would invent a single historical experience.

The private [content manifest](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/starting-area-audit.json>) records source hashes, line references, 536 locale records, 21 selected actor definitions/spawns, and 15 relevant map metadata assets. Original text and raw resources stay outside Git. The locale came from unmodified, pinned [FreeRealmsLocaleTools](https://github.com/Udaya-X2/FreeRealmsLocaleTools), not a new format parser.

| Recovered sequence | Recovered completion actions |
|---|---|
| Hero's Journal | Camera and movement instruction; guards/minions; healing potion and ultimate; find, interact with and close the journal |
| The Last Best Hope | Talk to Darkthorne, defend her, then reach Pappy Farnum |
| Saving Farnum's Farm | Fill a bucket and extinguish farm fires |
| Cooking101 | Mae, rainleaf/gummyfruit harvesting, Strong Arm Stew |
| Gear Up! | Cleetus, copper mining and **five** smelted copper bars |
| Your First Weapon | Forge the Funky/Spunky Scrapper Hammer; recovered wiki reward16 coins |
| Fighting off the Pack | Equip the hammer, switch to Brawler, defeat **five** Hooligan Wolves, return to Cleetus; recovered wiki reward18 coins |
| One More Gloam | Pappy and the nearby Gloam Creature |
| Where's the Help? | Reach Cobblestone and speak to Bartle |
| Pull the Plug | Destroy the Gloam Artifacts shielding the blockade |
| Cobblestone Showdown | Defeat Mac and free the village |
| Off to the Queen | Enter the Royal Palace and deliver the message to Queen Valerian |

This table joins recovered locale titles and actions by their adjacent dialogue/story sequence. **It does not establish each numeric quest binding.** Legacy's journal lists13 main quest IDs across these three hubs:2513,2514;2521–2526;2527–2530,2566. The locale also retains an explicitly retired link quest. No `Quests.json` ships in the pinned resources, so ordering is insufficient proof of a title-to-ID mapping. Local implementation must use reserved IDs and disclose reconstructed bindings until stronger evidence appears.

Original optional opening stickers survive for quests2563,2564 and2565: another stew helping, additional copper, and another hammer. Other region1 stickers lead outside this opening to derby and later activities. They are not evidence that every later activity belongs in the first-area gate.

Useful recovered actors include Mae2538, Pappy3211, Cleetus3232, Bartle2087, Hooligan Wolves3156–3158, cooking table2537 and harvestable plants2540/2546. Their community spawn coordinates are in the manifest. Profile IDs are Adventurer1, Brawler43, Chef45, Miner14, Blacksmith16, Medic11 and Archer35. Exact Mac/Gloam Creature actor bindings, quest rewards, original appearance grants and the complete discovery checklist are missing. Legacy supplies one mushroom collection, which cannot establish complete opening collections.

The supplied `Resources/ObjectTerrainData.xml` line33 maps geometry86, `NPXCavern`, to `sg_npx_cavern_01`, sky `sky_blackspore24.xml`. Its area XML explicitly identifies first, tutorial second and tutorial final rooms. This supports the opening cave map more strongly than the older `bw_briarheart_caverns` filename. Locale5100532 names encounter411, but current community minigame row411 belongs to another era. The local dungeon must serialize the cave details explicitly; original waves/spawn scripting remain reconstructed.

Community evidence comes from [FreeRealms-Legacy](https://github.com/raisingkaines/FreeRealms-Legacy), pinned at9e082025391177d5a33ee18c714ddd06b457a695, and Sanctuary at4f028dcc274f628b65ef665fd59b184f5f3d3e3e. The private manifest records exact paths. A [community Gloam Overlord page](https://freerealms.fandom.com/wiki/Gloam_Overlord) corroborates the later story; its indexed text is secondary evidence. A [2009 firsthand cooking guide](https://www.tentonhammer.com/guides/free-realms-chef-guide) describes the earlier Sacred Glade branch and must not determine this build's opening. The archived ZAM index was found but direct retrieval returned401. Targeted first-party searches did not recover a complete opening database.

Implementation boundary: restore the enumerated opening story and observable rewards/discoveries/classes, retaining the difference between recovered names/models/locations and reconstructed scripts/counts/rewards. Original crafting minigames, full tutorial cinematics and native ability balance remain separate work; interaction substitutes must be labeled when used. Native-client completion and restart proof are still required before marking the playthrough complete.

The separate `opening.rs` module and private `rust-opening/opening.json` now define17 local stages,51 ordered objectives,27 actors and4 discovery markers. This includes the12 named story steps,3 farm sticker contracts and2 beginner job/appearance epilogue steps. Acceptance/turn-in require nearby actors; gather/craft events require the matching location, zone and job; rewards and discoveries persist once. Native class/equipment, combat and dungeon events are typed integration inputs. The original hammer definition is unavailable in supplied item definitions, so Brawler's Mallet of Sweeps75030 is explicitly a local substitute.

Validation:7 ordinary focused tests plus2 explicit private-input checks pass. The private traversal completes all17 steps through typed events with reload after every step; it is **synthetic progression proof**, not native playthrough acceptance. Completion requires all17 steps and all4 local discovery markers. The original camera globes/barriers, potion/ultimate and cinematic/journal-close micro-actions are not claimed restored by this module.

2026-10-08 Farnum correction: the saved [Your First Weapon](https://freerealms.fandom.com/wiki/Quest:Your_First_Weapon) revision27815 records16 coins and a material grant of10 Copper Bars,5 Turquoise,5 Wooden Planks and5 Thorny Fasteners. Its goal calls the hammer Funky, while its dialogue and the supplied2014 locale call it Spunky. [Fighting off the Pack](https://freerealms.fandom.com/wiki/Quest:Fighting_off_the_Pack) revision28955 records Brawler, **five** wolves, Cleetus turn-in and18 coins. These2013 pages improve the local objective/economy evidence; they do not establish numeric quest IDs or every2014 difference. The supplied locale title remains Fighting the Pack; local visible title now uses the wiki wording.

The generator and private content now use five distinct wolf instances and rewards16/18. The original recipe is recorded, but the local anvil action still substitutes for crafting and does not grant or consume those materials. Supplied catalogs prove Turquoise2752 and Thorny Fasteners56; Copper Bar, Wooden Plank and the original hammer are absent from the supplied server/creator catalogs. Community catalogs contain candidates, but no missing or unloaded item ID was invented or silently granted. The labeled75030 hammer substitute remains.

Previous opening/encounter files and the unchanged character save are preserved in `lab/backups/opening-farnum-20261008T064435Z`. The save had no opening or encounter progress hashes to migrate. [Correction report](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/opening-farnum-correction.json>) records old/new hashes, catalog evidence and all9 passing focused checks. The native full playthrough remains pending.
