---
type: content-dungeon
zone: Wilds (northeast / northern Wilds)
combat: true
---

# Return to Sender

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Return to Sender is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* and *Combat Mini-game*, difficulty 3, "started by clicking on an Evil Postman in the northeast Wilds". [wiki: Return to Sender!] No video frame of it was found.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Return to Sender!" (UI `loc 1438097234`, game data `loc 169385862`); undecorated forms "Return to Sender" (`loc 4248947857`, `1191819207`); dev name "Return to Sender Encounter" (`loc 1989502559`). The undecorated pair may be a quest title (as with Nettleseed Nibblers); unconfirmed | [client] |
| Panel description | "The Postal Chaos Alliance is wreaking havoc on the postal service! Stop them before they destroy all the mail!" (`loc 2740930807`) | [client] [wiki] |
| Type | Forest battle, difficulty 3 | [wiki] |
| Entrance | Click an **Evil Postman** in the Wilds. The wiki text says "northeast Wilds" and its infobox says "Northern Wilds". The Postal Chaos Alliance camp is a Northern Wilds location on the wiki. | [wiki: Return to Sender!, PCA Camp] |
| Story hook | The Postal Chaos Alliance (PCA) postmen steal mail and throw it on bonfires. Client lines: "There is mail scattered everywhere! The evil postmen are collecting it to throw into the bon fires! Don't let them destroy more than 15 letters or packages." (`loc 347866273`). That line is not tied to this battle by name; it matches the dev strings below. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 or higher** (infobox "Levels 5 and higher"; difficulty 3). Generic tier strings: "Requires a level 5 combat job" (`loc 2145075890`), "You must have a combat job of at least level 5 to play this battle" (`loc 3173661191`), invitee form (`loc 66524078`). They are not tied to this battle by name. | [wiki] [client] |
| Members only | No. Members get "Members Only Bonus: 37 Coins". A client string "Bonus: 37 Coins!" exists (`loc 2664870581`), one of many "Bonus: N Coins!" strings, so the match is weak evidence. | [wiki] [client] |
| Unlocking quest | None found. | gap |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Knock out all of the evil postmen!"** | [client `loc 3300949067`] [wiki] |
| Bonus | None listed. No "Return to Sender Bonus Objective" or "Win Combat Encounter Return to Sender" string exists in the client. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Enemy | **Evil Postman** (`loc 4062420947`, game data `4214333656`) | The wiki lists the group as "Evil Postmen". Counts unknown. Client calls PCA Operatives "evil postmen that can be found wandering the wilderness" (`loc 11589577`); "PCA Operative" is a separate client name (`loc 1323460666`), identity with Evil Postman not confirmed. | [wiki] [client] |
| Boss | **Mailbit** (UI `loc 3225363343`, game data `3167520901`) | The wiki lists Mailbit under bosses. Its abilities are unknown. | [wiki] [client] |
| Mechanic (dev strings) | "Return to Sender 219: Throw package", "Drop Letters on death", "Mock throw into fire", "Drop Stack of Mail on death proc effect", "On Death Drop Package", "Drop Package triggered by on death ability 2825" | Suggest postmen carry mail, throw it into fires, and drop it when knocked out. What "219" and "2825" mean is unknown. | [client `loc 519825352`, `569482365`, `948241664`, `2207756256`, `2228151720`, `4028634546`] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "59 Stars" | [wiki] |
| Members Only Bonus | "37 Coins" | [wiki] |
| Prize wheel | One wheel per combat job, each with an **8 Coins** slot (as the wiki writes it). | [wiki] |
| Quest reward | Not found. | gap |

### Prize wheel by job

Every item name exists in the client locale as written. [client, checked by exact name] [wiki]

| Job | Items (plus 8 Coins) |
|---|---|
| Archer | Archer's Composite Bow of Volleys, Archer's Power Shard of Vitality III, Archer's Ring of Energy III, Battle Item Mystery Pack, Drifting Archer Cap |
| Brawler | Battle Item Mystery Pack, Brawler's Anvil Hammer of Sweeps, Brawler's Power Shard of Vitality III, Brawler's Ring of Energy III, Catchweight Brawler Helmet |
| Medic | Battle Item Mystery Pack, Emergency Medic Cap, Medic's Bonesaw of Triage, Medic's Power Shard of Vitality III, Medic's Ring of Energy III |
| Ninja | Battle Item Mystery Pack, Ninja's Power Shard of Vitality III, Ninja's Ring of Energy III, Ninja's Scythe of Dragonstrike, Nusumi Ninja Cowl |
| Warrior | Battle Item Mystery Pack, Combat Bonus Warrior Helm, Warrior's Battle Hammer of Spinning, Warrior's Power Shard of Vitality III, Warrior's Ring of Energy III |
| Wizard | Battle Item Mystery Pack, Scholar Wizard Fez, Wizard's Bone Wand of Shock, Wizard's Power Shard of Vitality III, Wizard's Ring of Energy III |

Example loc ids: Drifting Archer Cap `1285231415`, Scholar Wizard Fez `39413729`, Nusumi Ninja Cowl `508185249`. [client]

## Related quests and NPCs

No quest could be proven to send the player to this battle. The wiki has no quest page for it, and no video frame shows it. Client text about the Postal Chaos Alliance that may be related, **unconfirmed**:

| Client text | Id | Notes |
|---|---|---|
| "Stop the PCA postmen from destroying the mail!" / "Stop the PCA from destroying the mail." | `loc 1607767424`, `2465017856` | Objective-style lines. Owner quest unknown. [client] |
| "Stop them from destroying as much mail as possible!" / "...mail belonging to other people!" | `loc 981406864`, `1015253946` | Could be goal or bonus text for this or another PCA activity. [client] |
| "You sure showed those PCA postmen a thing or two!" | `loc 1923872681` | Completion-style line. [client] |

The Postman job's quest line culminates at the PCA Camp in the Northern Wilds. [wiki: PCA Camp] NPCs: [[Mailbit]]. Zone page: [[NPCs - Wilds]]. Job: [[Postman]].

## Conflicts

1. **Location.** The wiki text says "northeast Wilds"; its infobox says "Northern Wilds". Unresolved.
2. **Enemy name.** The wiki writes "Evil Postmen" as the enemy; the client enemy name is the singular "Evil Postman" (`loc 4062420947`). Plural "evil postmen" appears only inside goal text.
3. No wiki item name lacked a client match, so no item corrections apply.

## Gaps

- Everything about live play: no video. Wave layout, postman count, Mailbit's health and abilities, how the 15-letter limit works (if it applies).
- Exact coin and star payouts. The meaning of "8 Coins" is not interpreted.
- Which quest(s), if any, start or require the battle.
- How it spawns and respawns in the Wilds.
- Group size.
