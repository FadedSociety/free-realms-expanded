# Cooking Later Phase Geometry

Read-only extraction from the existing official JPEXS26.3.0 XML export establishes authored Measure, Add and Stir positions. This is geometry evidence only. The Cooking agent's separate original AS2 control review is required before claiming the later rounds are playable.

At the supplied1280×960 client transform, origin(210,50), scale215/192:

| Target | Client registration | Bounds |
| --- | --- | --- |
| Measure bucket | (642.69,327.67) | Authored rotated registration only |
| Measure mold | (743.08,702.36) | Authored registration only |
| Add MeatStew drop | (643.02,520.84) | AABB(486.42,427.31)–(799.63,614.36) |
| Stir bowl | (626.28,534.27) | Hit AABB(387.40,395.65)–(861.12,687.85) |

Add sprite560 recipe frame51 updates inherited mcBottom274 to(386.7,420.55), Xscale0.87597656 and Yscale0.6345062, before parent Yscale0.99980164. Using its frame1 pan pose would target the wrong size/position. All15 mcIngredient registrations are inherited unchanged from frame1 and are recorded individually; their indices are placement slots, not ingredient identities.

Stir sprite1758 MeatStew frame1 has mcBowl1732 at(371.75,432.55), mcHitZone1744 at(370.55,439.5) with scale0.8999939. Its shape1743 is an irregular closed four-quadratic path, not an ellipse. AABB corners are not hit targets. A central geometric candidate path is(693.47,534.27)→(626.28,601.44)→(559.10,534.27)→(626.28,467.10); this does not resolve control-entry behavior. Authored spoon offset(-44.85,-85.05) and static hand offset(182.5,-175.5) are artwork poses. They must not be added to runtime mouse registration, whose semantics are reviewed separately.

Important unresolved risks: sprites560/1758/2205/2206 contain no pt-prefixed named placements, no clipActions placements and no DoAction tags in any frame. Default measure mold2165 frame1 has no named mcMeasureLine. The control code refers to these names, and this XML does not prove runtime construction elsewhere. Geometric center coordinates do not bypass this evidence gap. No movie repair or control/source/save/input changes were attempted.

Final proof `lab/reports/cooking-later-phase-geometry-v2.json` SHA256 `62d221325652488621745dbd5c807e6fb5220ad92ca7370b396db621b81a50fd`, reproducible with sibling `extract-cooking-later-phase-geometry.py`. V2 composes the unused inner-mold registration with its outer mold before labelling client coordinates; the original report remains preserved, and all outer target coordinates/bounds/risks above are unchanged. It uses standard XML access to the pinned community export, not a new game-format reader. Screen mapping must be rechecked if the client is resized.
