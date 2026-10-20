# NPC Greetings - Lakeshore and Wilds (T298)

Adds 19 recovered `greeting_string_id` values to the drawable `legacy_data` NPC rows in
`data/content/npcs/lakeshore.json` (3 of 4 rows) and `data/content/npcs/wilds.json`
(16 of 74 rows). No Rust production change: `npc_interaction::dialogue_context` already
copies `greeting_string_id` into the 26/3 body, and no wire shape changes. T246's
starting-road files, T280's Sanctuary/Cobblestone files and the Snowhill/Greenwood
greetings slice are untouched.

## Method (reproduces T246)

1. **Ids versus hashes.** A region/job guide number such as `4122489045` is a
   `JenkinsLookup2("Global.Text.<id>")` locale key, not a numeric id. Every number was
   resolved through `lab/reports/locale-id-map.json` (`hash_to_ids`), checked against the
   `{id,text}` row in `lab/reports/locale-id-to-text.json`, and then found by recomputing
   `jenkins_lookup2("Global.Text.<id>")` against the hash column of the read-only stock
   client locale `lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat`. The
   `line` column below is the physical line in that file. All 19 ids resolve twice; no
   locale overlay is needed. (Ty's E1 row lists two candidate pitches: `4122489045` is
   `Global.Text.101577`, while the chosen id `24829` is `Global.Text` hash `1824578395`,
   the line the content quest record binds as its intro.)
2. **Speaker proof.** A line counted only when (a) the line names the NPC in the speaker
   role (or is self-identifying first person for that speaker), (b) the region/job guide's
   quest row names the NPC the giver and the line is that quest's offer speech, or (c) a
   guide row directly attributes the line to the NPC. Near-lines and third-person
   mentions were rejected; for example the Wizard guide's third-person Nurble mentions
   ("Nurble does the cleaning", "Tell Nurble to clean my spell book") were not used.
3. **Quest offer speech (R).** For 13 rows the greeting is the quest record's
   `intro_text_string_id`: the record binds `giver_npc_id` to the NPC and the intro id to
   the offer speech. Those intro bindings are reconstructed: `scripts/quest_text_backfill.py`
   filled the `T-1/T+1/T+2` block ids where unambiguous, and the Wizard record's intro
   24947 is the guide-cited line (`(R) T242`). Each edited row therefore carries a second
   `reconstructed` source naming the quest file, quest id and intro id.
4. **Guide-attributed lines.** Desi, Voggun, Troggers, Bluz and Hans have no quest record
   in the bundled content; the Wilds guide row directly quotes/attributes their first-person
   line, so their source is the client line plus the fixed-default policy only.
5. **Fixed default.** Picking one verified speaker line as the fixed Talk greeting, without
   the original server dialogue/phase table, is reconstructed policy `(R)`; the locale text
   and ids are recovered client facts and no text is invented.

## Added greetings

| NPC id | Name | Greeting id | Hash | Locale line (short) | Evidence |
|---|---|---|---|---|---|
| 9000000223 | Ty | 24829 | 1824578395 | Part of me wants to help... Trolls are kidnapping pixies from Lakeshore. | en_us_data.dat line 48173. Guide: `Jobs/Ninja.md` line 69 (E1 giver Ty, first-person pitch). Quest `ninja.json` 3110001 binds intro 24829. NPC guide: `NPCs - Lakeshore.md` line 39. |
| 9000000233 | Waterwings | 409412 | 1256773114 | Finally the fish is done!... while I clean up? | line 33294. Guide: `Jobs/Fisherman.md` line 124 (An Offering, giver Waterwings). Quest `fisherman.json` 3070033 binds intro 409412. NPC guide line 29. |
| 9500000363 | Raeya Alderspark | 41207 | 3015081219 | It takes a lot to keep our campfire burning... I need globs of bixie wax... | line 79651. Guide: `NPCs - Lakeshore.md` line 45 (Flickering Flames giver Raeya). Locale id 41207 immediately precedes title 41208. |
| 9000000243 | Aster | 387488 | 3163514493 | You want to help me? As if!... Come back when you are at least a level 10 warrior. | line 83481. Guide: `Jobs/Warrior.md` line 101. Quest `warrior.json` 3150010 binds intro 387488. NPC guide `NPCs - Wilds.md` line 197. |
| 9000000273 | Chase Wheeler | 394709 | 1007521181 | Unlike Linx, I don't have any sour grapes after a loss... | line 26689. Guide: `Jobs/Kart Driver.md` line 92. Quest `kart_driver.json` 3080029 binds intro 394709. NPC guide line 180. |
| 9000000317 | Emahl | 130251 | 4010850362 | You've stumbled into the wrong place, postman... We plan on destroying all the mail. | line 106064. Guide: `Jobs/Postman.md` line 129. Quest `postman.json` 3130033 binds intro 130251. NPC guide line 106. |
| 9000000324 | Fizzlesticks | 24947 | 698508245 | That no-good Robgoblin Adept stole my spell book!... | line 18388. Guide: `Jobs/Wizard.md` line 76. Quest `wizard.json` 3160003 binds giver Fizzlesticks and intro 24947 (record marks it `(R)` from the guide). NPC guide line 89. |
| 9000000332 | Glerga | 73077 | 3466351639 | I heard Lucca's review of your cooking... I sto--er, I perfected a couple recipes... | line 91803. Guide: `Jobs/Chef.md` line 136. Quest `chef.json` 3050030 binds intro 73077. NPC guide line 90. |
| 9000000347 | Herme Fickleleaf | 104327 | 2193026908 | I can't promise Maple is still where she said she would be... | line 57742. Guide: `Jobs/Card Duelist.md` line 112. Quest `card_duelist.json` 3040027 binds intro 104327. NPC guide line 40. |
| 9000000415 | Nurble | 141009 | 2705176635 | ...a lot of my precious junk... I am unable to pay Andrew... I'll repay the rest of my debt. | line 71216. Guide: `Jobs/Brawler.md` line 136 (One Man's Junk..., giver Nurble). Quest `brawler.json` 3030028 binds intro 141009. NPC guide line 91. |
| 9000000477 | Starshine | 391043 | 3886409354 | Someone as brave as you should really share your talents!... | line 102718. Guide: `Jobs/Adventurer.md` line 115. Quest `adventurer.json` 3000027 binds intro 391043. NPC guide line 74. |
| 9000000488 | Terrence | 73492 | 1322558947 | Those medical supplies need to get back to Sanctuary!... | line 35003. Guide: `Jobs/Medic.md` line 87. Quest `medic.json` 3090010 binds intro 73492. NPC guide line 36. |
| 9500000869 | Ari the Fish | 115443 | 1892561356 | I can't keep up this charade anymore... | line 49906. Guide: `Jobs/Card Duelist.md` line 110. Quest `card_duelist.json` 3040025 binds intro 115443. NPC guide line 201. |
| 9000000335 | Grosh | 78924 | 2060862346 | Troll Summoner kidnap for Queen... You go find him... You defeat him. | line 54301. Guide: `Jobs/Ninja.md` line 70 (Stolen Away, giver Grosh). Quest `ninja.json` 3110002 binds intro 78924. NPC guide line 115. |
| 9500000893 | Desi | 25001 | 1008851474 | Please, you've got to help me!... kidnapped my dog, Lucy. | line 26732. Guide: `NPCs - Wilds.md` line 42 quotes this first-person plea in Desi's dog-rescue row. |
| 9500000958 | Voggun | 21014 | 932035218 | Voggun happy you help village... Me help you now. I teach you runecasting recipe. | line 24650. Guide: `NPCs - Wilds.md` line 116 attributes this teaching line to Voggun. |
| 9500000954 | Troggers | 20984 | 217873432 | Can you please help Troggers? Queen take too many of my pigs... | line 5721. Guide: `NPCs - Wilds.md` line 124 lists this first-person plea as Troggers' Rescue Porky text. |
| 9500000876 | Bluz | 73378 | 2001002164 | You must help Bluz get his status with Queen back! Defeat the Trolls in the Forest Troll Fort for me! | line 52697. Guide: `NPCs - Wilds.md` line 126 cites this line in Bluz's The Queen's Favor row; "for me" makes Bluz the speaker. |
| 9500000907 | Hans | 9599 | 2276171514 | You've seen my sister, Greta?... She's by the road? Ok, thanks! | line 59936. Guide: `NPCs - Wilds.md` line 198 attributes the line to Hans ("Hans says"); his own row is line 199. |

## Exclusions (still blank, with why)

- **Sam Durango** (9500000364), the fourth drawable Lakeshore row: trainer, but the
  client strings that name him are a bear-keeper's third-person complaint (hash
  2726196253) and the hand-in objective "Return to Sam Durango" (hash 408958409); neither
  is his own speech. No speaker proof, so no greeting.
- The remaining 58 drawable Wilds rows: vendors, racers, bosses and quest targets whose
  only recovered text is a third-person objective, a guide description, a video-only
  line, or a line spoken by another NPC. Examples rejected: Snarf's third-person
  self-reference ("Snarf take picnic area to have a party"), Nog's stew offer (the guide
  attributes the stew text to Nog-or-Hogosh by wording), the Peacepetal nectar line (the
  speaker is Ginger), and every kart/derby trainer objective that names the trainer in
  the third person.
- `poi_anchor`/`unanchored` rows in both files (for example Derp Axemaker, Shimmer
  Shinyaxe, Kazuo, Sakka) are not spawned by `NpcVisibility::load`, so a greeting there
  would be unreachable; they stay blank.

## Client evidence

- 26/3 handler `0x00aa12e0` (IDA decompile: calls `sub_A9EF10` then `sub_9F5AE0`) calls
  payload reader `0x00a9ef10`, which stores the first payload dword at object `+16`
  (`*(this + 16) = *v3`). Display routine `0x009f5ae0` passes `*(_DWORD *)(a2 + 16)` to
  the Lua lookup `sub_97C1E0` immediately before `":SetDialogText"`. That is the first
  dword after the four-byte family/subtype header, matching `encode_dialogue_26_3`'s
  `initial_dwords[0]`. Re-verified for T298, 2026-10-20.
- `gameplay/npc.rs::dialogue_packet_with` writes `context.body_string_id` into that dword;
  `patch_npc_actor` only overwrites bytes 12..20 with the runtime actor GUID.
- The 26/3 byte layout is pinned by
  `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`;
  no new wire shape is introduced.
- 26/9 menu layout (used by the spawn/menu assertions): header, npc id, option count,
  seven dwords per option; Talk is always choice 1 (`npc_interaction.rs` build_options).

## Open gaps

- The original server greeting/phase tables are not recovered; every row is a fixed
  default `(R)`, so an NPC always repeats the same line regardless of quest phase.
- 58 drawable Wilds rows still have no greeting; most have no recovered first-person
  line. Several have third-person quest text only.
- No native client run in this task (lead-controlled); visual rendering of these bodies
  is unverified.
