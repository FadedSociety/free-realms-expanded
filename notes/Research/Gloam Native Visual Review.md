# Gloam Native Visual Review

Ended native run 183551 now has retained visible approach, differing close-range melee poses, damage, goal completion and disappearance. A complete terminal death/corpse animation remains unproved. [Manual visual review](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/gloam-visual-183551-v1/visual-review.json), SHA256 `4f6b898a2bfd560ea4eca9aa942f7eabd82c8754907e169808fa029baf23d851`, binds the original ended recording, extraction manifest, contact sheet and reviewed full frames.

Original `gloam-valley-retry.mkv` SHA256 is `47261abedfd20cfc9f6c16eb17a419fd5a4e21984c3c720e673f78df812f70c3`. Existing installed FFmpeg extracted 80 frames at 2 fps from nominal 170–209.5 s into a fresh private report directory. These times are nominal filter times, not exact packet/UTC synchronization; the recording file creation time served only as a search anchor.

|Nominal time|Retained native observation|
|---|---|
|185s/frame0031|Purple stone Gloam with blue/purple flame effects is visible beyond the fallen log. PlayerHP1000/1000.|
|188s/frame0037|Labeled Gloam approaches and substantially enlarges on screen; torso and limbs differ from distant pose.|
|190s/frame0041|Creature is at melee range with raised/canted limbs, visible−35 hit text and playerHP965/1000.|
|192s/frame0045|Torso is canted and an arm extends horizontally left, distinctly different from190s. Yellow−100 and GoalUpdate1/1 appear; playerHP930/1000.|
|193s/frame0047|Creature is absent, completion chat is visible and playerHP895/1000. This is defeat/disappearance evidence, not a complete death animation.|

![Gloam approach and melee evidence](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/gloam-visual-183551-v1/contact-sheet.jpg)

Pixels establish observed movement/pose variation and completion. Animation IDs, exact damage timing, whole-state preservation, rewards and restart belong to the separate finalized authoritative packet/checkpoint proofs owned by combat_results. Neither the fallen-log appearance nor these frames prove general collision/pathfinding quality. No source, active content/save, release, native input, movie or shared decoder output was changed.
