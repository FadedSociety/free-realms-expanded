# NPC Location Hints

W66 analysis report. Updated 2026-10-14. Analysis only; no regional NPC data or Rust code changed.

## Result

The locale miner found **394 distinct quote/place claims for 217 normalized NPC names** across all 85,932 locale rows. Repeated identical phrases share one JSONL row with all matching locale IDs. Claim destinations resolve to 304 zone-name rows, 80 POI rows and 10 client-area rows. Relations are 310 `in`, 73 `at` and 11 `near`.

| Measure | Result |
|---|---:|
| Unanchored regional rows newly assigned a missing zone from direct `in`/`at` text | 0 |
| Unresolved NPC refs gaining general area evidence | 2 |
| Unresolved NPC refs gaining a unique direct POI ID | 4 |
| Distinct unresolved refs gaining either type of location | 5 |
| Non-legacy regional rows with direct text POI proposals | 26 |
| Regional rows with only a nearby-POI hint | 1 |
| Legacy-data rows agreeing with a direct zone claim | 100 |
| Legacy-data rows with a conflicting direct zone claim | 1 |
| Legacy-data rows without direct zone evidence | 477 |

The five unresolved refs with new typed location evidence are Gunther (9000000100), Hilbert (9000000349), Jeni Shortfuse (9000000363), Kade Carmichael (9000000123), and Sampson Idlefist (9000000453). Jeni gains both area and POI specificity. The 118 unresolved `npc` rows include eight slash-separated name variants; these were excluded from automatic name matching. The three rows already typed `ambiguous` remain unresolved.

Of the 26 direct regional POI proposals, 24 agree with the current reconstructed POI candidate and two conflict with it. Four proposals have high confidence from multiple locale IDs; 22 have medium confidence from one locale ID. Morninglory has one low-confidence nearby-POI hint, which is not an exact anchor.

## Method and provenance

- **Workflow:** Ponytail kept the miner to one standard-library script in the lab; Caveman kept progress communication concise. Game-recon, web research, reverse-engineering and asset-pipeline were not used because this task joins already extracted text/content and neither inspects binaries nor creates engine assets.
- Read `lab/reports/locale-id-to-text.json` as the source of quote text and locale IDs. The output retains only short clauses with an NPC-name match and an explicit location predicate.
- Inspected all 17 quest-content files (651 rows). Their fields include `giver_npc_id` and `title_string_id`, but all objective arrays are empty; they provide no locale-to-speaker binding for “here at Y” or “welcome to Y” lines.
- NPC display names and IDs come from `prototype/osfr-tools/data/content/npcs/*.json`; matching aliases also use resolvable locale name IDs. Unresolved refs come from `npcs_unresolved.json`; names from `npc_spawns_extra.json` help resolve additional Legacy spawn identities. Slash-separated unresolved names are not assigned to one person. Leading titles such as `Captain` are tolerated by matching the unique base name.
- POI names are resolved from `startup.json` `world.points_of_interest.name_string_id`; positions come from the same captured startup rows. 149 POI rows were present and 135 had a resolvable name string. A unique POI ID carries its captured POI position in `hints.jsonl`.
- Client sub-area names and IDs come from the extracted `FabledRealmsAreas.xml`, at `lab/extracted/client-assets/pack_12/000026.xml` (1,279 `AreaDefinition` entries; SHA-256 `fef2fe4f4a1189504c5ba6de9253eefb9071d6e6142bbe405d8d3caf03dd0a8c`). The asset name and extraction mapping are recorded in `lab/reports/starting-area-audit.json`. Only full/camel-split labels and multiword suffixes are matched; generic one-word suffixes such as “town” and “water” are excluded.
- Zone names/IDs come from `data/content/zones.json`. Those numeric IDs are reconstructed content keys, not original native zone IDs. Regional NPC IDs and unresolved ref IDs are also content/reference keys, not recovered native actor IDs. `notes/Research/NPC Placement.md` records that 578 regional positions were copied from Legacy built data and are not capture-derived.
- Matching is case-insensitive and Unicode-normalized. It accepts direct forms such as `Find X in Y`, `X can be found in Y`, `X is waiting at Y`, `X lives in Y`, and movement directions that explicitly send the player to a named place before meeting an NPC. Sentence boundaries and another NPC mention block a pair from spanning unrelated clauses. A unique “Junction” alias resolves to Highroad Junction; a same-name zone takes precedence over its teleport POI.
- `near` stays a `near` claim. It does not contradict a different recorded zone because proximity alone does not prove zone membership. Repeated POI labels remain ambiguous and have no position or proposed anchor.

Locale rows provide no speaker-to-NPC binding for `welcome to Y`, `here at Y`, or speaker-relative directions. Those speaker claims, “heard of Y” negatives, and unbound “right over there” directions are not attributed to an NPC. The outputs contain no player names or chat. Text claims show what the client text says; they do not prove a static actor spawn or its exact coordinates.

## Examples

- **Merigold — general area:** locale IDs 115463 and 407520 say “Find Merigold in Highroad Junction” and “Merigold can be found in Highroad Junction.” The name matches NPC content ID 9000000402. `Highroad Junction` resolves to reconstructed zone key 7, so this is an area vote, not a native zone ID.
- **Amaranthus — POI:** locale 419105 says “You'll find Amaranthus in Bristlewood Glade.” NPC ID 9500000084 resolves to POI 76 at `[-385.92633, -68.03617, 1871.8076]`; the text agrees with the existing reconstructed POI candidate.
- **Bulb — client sub-area:** locale 116723 places Bulb at Pixie Nursery. This resolves to `Pixie_Nursery`, area ID 90, in `FabledRealmsAreas.xml` line 232.
- **Jeni Shortfuse — unresolved ref and POI:** locale 22933 says she “can be found near Sanctuary at the Queensfields.” It supports a nearby Sanctuary area vote and a medium-confidence POI 24 anchor at `[-1490.254, -35.049, 409.188]` for ref 9000000363. The Sanctuary relation is `near`; Queensfields relation is `at`.
- **Sampson Idlefist — unresolved area:** locales 39828 and 72821 direct the player to Sampson in Sanctuary. They add area evidence to ref 9000000453; they do not supply a POI anchor.
- **Legacy-data flag — Simone:** seven locale IDs (20141, 140475, 140539, 22454, 442266, 442236, 71472) place Simone in Cobblestone Village. Regional ID 9000000467 currently says Sanctuary. Its Legacy-data position is `[-1830.891, -42.04188, 428.7502]`; the captured Cobblestone Village POI (ID 7) is about 75 m away. That distance is only a review clue, not proof of zone membership or a native capture. Keep the row unchanged pending the owner’s review.

Two reconstructed POI proposals need the owner’s review because they differ from the current candidate: Assistant Bogse (locale 440106, Precursor Ruins POI 147) and Lucinda (locale 415388, Blackspore POI 16). They are direct `in` claims, but the single locale row does not establish exact NPC coordinates.

## Applying proposals later

Treat `text_poi_proposal` as a coarse candidate anchored to the POI position, never as the NPC’s recovered spawn coordinate. Keep `relation` and confidence. A `nearby_poi_hint` is not an exact anchor. The POI positions describe named startup points, while area claims describe client area labels.

A later content builder should review every new proposal with the owner before changing a reconstructed/unanchored row. Review the two current-candidate conflicts first, then confirm NPC identity when names have multiple regional IDs. Keep all `legacy_data` and capture positions unchanged. Do not convert reconstructed zone keys into native zone IDs or treat a text match as proof of historical placement.

## Artifacts and check

- `lab/content/npc-location-hints/mine.py`
- `lab/content/npc-location-hints/hints.jsonl`
- `lab/content/npc-location-hints/summary.json`
- `project/notes/Research/NPC Location Hints.md`

Run from `project` with `python -I ..\lab\content\npc-location-hints\mine.py`. The run completed with 394 claims. A separate `python -I` integrity check parsed every output row, confirmed all locale IDs resolve, checked row/placement totals and POI positions, and passed.
