# Harvesting job binding

October 9, 2026, Pacific. Research decision, not native acceptance.

The missing Farmer rotation profile is not the right blocker for the cooking-related harvesting minigame. The cached wiki's Chef page explicitly associates Chef experience with harvesting and cooking. Its Harvesting page distinguishes minigame produce used in cooking from housing-farm produce, which cannot be used in cooking. The Farmer page describes a separate housing-farm progression introduced in May 2011, outside the normal job rotation. The preserved rotation profiles include Chef45.

Chef45 is therefore the supported job candidate for a later local Harvesting111 launch. This is evidence for the job relationship, not a recovered native launch packet. Do not use game111 as a profile id, invent a Farmer profile, or replace the current quest gather path until native entry and ingredient-conservation checks pass.

## Provenance

Read-only SQLite query of `lab/references/freerealms-wiki/wiki.sqlite`; original cached rows were not modified. The four complete evidence rows are saved in `lab/reports/harvesting-job-binding-lead-20261009.json` with source URLs, revisions and hashes.

- Chef: revision46136, hash `870b47c6896dd4432808ef10278cb5b6d42f99fb974ff360858c270b9bf925a6`.
- Harvesting, Farmer and Jobs: complete revision/hash/text records in the same private receipt.
- `src/harvesting.rs::content` currently uses `profile_id: game_id` and explicitly labels it as a placeholder. That is not a usable production job gate.

The earlier [[Reviews/Type8 Gateway Integration Handoff]] remains useful for dispatch/save hooks. Its missing-Farmer assumption is superseded by this distinction. Mining18 integration proceeds separately using the already supported Miner14.
