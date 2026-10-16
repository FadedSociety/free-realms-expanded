# Original cave exit to Farnum Farmstead

2026-10-08 UTC. The supplied2014 client proves the opening cave is followed by **Darkthorne's Magic Door/portal to Farnum Farmstead**. The prior generic activity return to the old overworld spawn requires crossing an impassable cliff and does not reproduce this original story transition. A story-specific travel route is supported; the exact original arrival point remains unrecovered.

## Primary evidence

The unmodified pinned [FreeRealmsLocaleTools](https://github.com/Udaya-X2/FreeRealmsLocaleTools), revision `ba2ede225ff33edab8e8e0d20c45d4c5156309e5`, exported the preserved original `locale/en_us_data.dir/.dat` files. The focused proof records their hashes, export hash and14 exact private locale records.

| Locale ID | Original meaning |
| --- | --- |
| 4419563/4419564 | The Last Best Hope; defend Darkthorne trapped in the Heartseed Chamber |
| 4419610/4419612 | Defend Darkthorne, then talk to Pappy Farnum |
| 4420033 | Darkthorne instructs the player to use the Magic Door to Farnum Farmstead; she lacks enough mana to send them directly to the Royal Palace |
| 4419565/4420036 | Pappy recognizes the hero sent by Darkthorne and agrees the Farnums will help |
| 4420041 | Gloam creatures came through the portal with the player and started burning the farm |
| 4420042/4420043 | Saving Farnum's Farm describes Darkthorne's teleport to the farm |
| 5100071 | Story recap: player helped seal the Gloam Overlord, Darkthorne sent them to the farm, and wrote a message for Queen Valerian |
| 5100532 | Encounter411 Darkthorne Teleport |
| 5101947 | Gloamlings slipped through Darkthorne's portal to the farm |

Original Lua `Tutorial:MagicDoor` (27749) opens its tutorial dialog; the MagicDoor dialog branch (27914) plays original sound definition15643. Other original battle-exit strings115991/431160 likewise describe interacting with a Magic Door to leave. This supports the visible interaction concept, not a complete cinematic/door implementation.

[FreeRealms-Legacy](https://github.com/raisingkaines/FreeRealms-Legacy), pinned `9e082025391177d5a33ee18c714ddd06b457a695`, supplies Pappy NPC3211/model430 and his original community spawn `[-2184.513,-31.53728,328.1112]`. Mae2538 and Cleetus3232 are nearby in the farm. This identifies the destination NPC; it is not proof of the player's exact original arrival coordinate.

## Community-first coverage check

One ordinary public MediaWiki API request through actual Scrapling returned HTTP200 and confirmed **Quest:The Last Best Hope**, **Quest:Saving Farnum's Farm** and **Briarwood Caverns** currently have missing-page responses. Their links in other articles do not establish recoverable article contents. The existing [Farnum's Farm revision50145](https://freerealms.fandom.com/wiki/Farnum%27s_Farm?oldid=50145) places the farm south of the Cobblestone warpstone; [Gloamling revision50126](https://freerealms.fandom.com/wiki/Gloamling?oldid=50126) corroborates appearances at the farm and cave. Neither supplies the transition coordinate. Original supplied locale is therefore the decisive story evidence. No authentication, challenge bypass or new format parser was used; the verified searchable database was left unchanged.

## Implementation boundary

Use the existing verified normal zone-transition delivery for a legitimate **story-gated guide/door travel action**. Gate it on accepted/unrewarded Last Best Hope with cave goal complete and Pappy talk pending, plus the completed encounter411 state. An arrival near Pappy must be independently terrain-checked and labeled reconstructed placement. Keep generic activity return/restart positions separate so other minigames and user walking are preserved. Travel must not complete the Pappy conversation or grant a second cave reward. This review changes no gameplay, save or native input.

Private proof: `lab/reports/opening-farnum-travel-proof.json`; reproducible read-only helper `lab/reports/prove-opening-farnum-travel.py`. The report preserves raw public API response SHA256/path, original source hashes, NPC lines and explicit missing-data boundaries.
