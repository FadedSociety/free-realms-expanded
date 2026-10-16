# Native Cooking Controls

Read-only control notes from the preserved original `game_cooking.gfx` AS2 export, for the reconstructed six-round configuration. Screen coordinates must come from the actual native movie; the notes do not invent click targets or establish successful play.

| Round | Actual movie control |
| --- | --- |
| Tenderize Bat Meat | Aim the hammer over the moving meat and click while the hammer is idle. Eight successful tenderizing hits are configured. Clicks during the smash animation do not count. |
| Chop Rainleaf | Click the visible knife, not the ingredient. The knife's hit test starts each chop; fifteen one-click slices are configured. |
| Smash Gummyfruit | Aim over a fruit and click while the hammer is idle. The meter controls hit power; stronger hits count more. Two required hits per fruit are configured. |
| Measure Brown Broth | Hold the mouse button to control the bucket. Moving the pointer upward increases the bucket angle and pouring pressure; releasing stops pouring and resets the bucket. Follow the visible fill/target line and avoid overshooting. |
| Add ingredients | Drag the correct displayed ingredient into the bowl, in the order shown by the movie's ingredient list. A release outside the bowl or with the wrong ingredient does not advance the correct ingredient sequence. |
| Stir MeatStew | Hold on the bowl, then move the mouse around the bowl inside its hit zone. Follow the displayed direction arrows; ten complete turns are configured. Initial required direction is counterclockwise, but use the actual visual arrows. |

Each round shows original feedback before the next round. The movie stops its stopwatch and sends four stage statistics on `roundCompleted`; final completion sends summed bad/good/perfect action counts, cumulative game time, then `FRServer_GameWon(score,0)`. The server ignores the reported quantity for reward size and requires all six report groups before the fixed recipe transaction. The native movie’s clock is a stopwatch rather than the reconstructed nominal90-second row countdown.

Primary files under `../lab/decompiled/minigames/cooking/scripts/__Packages`: `Cooking/RoundTenderize.as`, `TenderizeElem.as`, `RoundChop.as`, `ChopKnife.as`, `RoundSmash.as`, `Hammer.as`, `RoundMeasure.as`, `RoundAddIngredient.as`, `RoundStir.as`, `StirBowl.as`, `CookingGameClient.as`, and `Library/Timer.as`. The export tool/source provenance is in [[Research/Original Cooking]].
