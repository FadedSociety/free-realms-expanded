# Native SelfData 011719

Run `lab/runs/client-baseline-20261014T011719Z`: release build after W45, where startup message 07 (opcode-12 self data) is built from session state by `startup_selfdata::build_self`. This was the first native run with all 46 startup messages built rather than replayed. The test character is a copy of `kart-weapons-probe-v20-20261010`.

| Check | Result |
| --- | --- |
| Login, character select, Go | Passed |
| World entry (welcome screen, HUD, minimap, name plate, Kart Driver job) | Passed |
| Health and power 1000/1000 and 100/100; action bar counts 59/28/33 | Passed |
| Inventory (VK 73): 359 coins, job items, detail pane | Passed. Coins match the save |
| Dev teleport `realm goto Mae` | Passed. Mae Farnum present with the interaction prompt |
| Menu → Exit Game → Yes | Passed. `session_ended` with `client_disconnect`; client exit 0 |

## Outfit

In the world the character wears base clothing; character select shows shoes, bootcut pants and so on. This matches the save, so it isn't a fault. The active profile is 48 (Kart Driver), which has no equipped rows. Character select shows the creation attachments, which are profile 1's equipment: definitions 28178, 14964 and 17412. The lighter outfit noted in [[Reviews/Native Startup Builders 215401]] has the same cause.

## Notes

- `world_ready` reports `zone_id 0` at spawn (-1901, -46, 454) in every run since 2026-10-10. It is pre-existing behaviour, not a regression. The zone id model is still owned by the world/instances integration round.
- The server saved three times. The middle save repeated the spawn position between two saves at the teleport destination. The final save was at the destination. This is worth checking when movement validation is built.
