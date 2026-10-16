# Pappy to Bartle Route Candidates

Read-only bounded research after stage7. **No complete safe native Pappy→Bartle walk is accepted. The relaxed candidate below fails the finer slope checks and is not recommended for blind navigation.** No source, content, saves or native input changed. Actual151602 southern climb/input history remains stronger local evidence than newly inferred descent chords; see [[Stage7 Native Return Routes]].

Follow-up [[Pappy to Bartle Main Road Detour]] finds a longer503.63m checked valley/main-road/eastern approach after the known climb foot. It uses individual original fence triangles rather than filled corral hulls. Native reverse rocky descent and onward walking still require inspection; this supersedes the rejected cliff route as the next candidate.

## Original geometry coverage

Unmodified ForgeLightToolkit9c06714c5d79d9152c322814bc904abad944e06e exports95 original DME meshes for528 original scene objects in X−2220…−1830/Z210…450. Unmodified fr-adr-toolkit5acfdf8d6006fa6f43d1d3605f076a6be7393e72 resolves direct ADR meshes. Every selected raw ADR/DME matches the preserved catalog CRC; original hashes, scene transforms, reader wrapper and outputs are bound in the proofs. Existing four original GCNK terrain meshes and original2,019-node MAP export are reused without a new format reader.

The GCNK community reader does not decode Bullet collision. Ground-band convex hulls conservatively include fences, till rows and plants; a failed proxy search does not prove an actual game passage is impassable. Two appearance-wrapper names cover three unresolved objects (roses and Wildwood billboard). They are at least90.67m by registration origin from the relaxed path, but their full bounds were not recovered or silently treated as known.

## Strict search and original road failures

Pappy is `[-2184.513,-31.53728,328.1112]`; Bartle is `[-1860.185,-51.30442,341.1369]`. A2-unit search with grade≤0.45, grid normalY≥0.9 and ground-footprint clearance≥1.5 finds no complete route to the tested Bartle approaches within this bounded rectangle. This is a conservative search result, not a collision or engine verdict.

The original shortest road graph candidate is734→733→732→731→730→1220→1221→1222→1223→1219→721→1215→718→717→716. Its problematic sections include:

- 734 `(-2164.974,337.39896)`→733 `(-2195.8774,291.04532)` intersects original corral piece02 object1254767884; nearby damaged corral clearance is1.073m.
- 733→732 `(-2168.1775,245.22307)` intersects damaged corral object1360108822 and reaches sampled grade0.534486.
- 732→731 intersects a farming-till footprint, illustrating the proxy's conservative treatment of walk-over-looking geometry.
- Further eastern road sections also intersect corral/plant footprints; MAP adjacency alone cannot justify automatic input through them.

Proof: [strict search and road checks](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/pappy-bartle-route-v1/guide.json), SHA256 `5f59d0d29920572e0b60b8639e9c2e5f9e38dba79f6d793d2a4de168106e8150`,303 source bindings. Preparers `lab/reports/prepare-pappy-bartle-props.py` and `analyze-pappy-bartle-route.py` preserve their existing outputs.

## Relaxed exploration candidate and finer rejection

Relaxing the grid to normalY≥0.8/grade≤0.55 finds a450.1909m candidate ending `[-1868,-51.13072,344]`,8.32477m in3D from Bartle. It descends southwest from Pappy via `(-2192,294)`, `(-2184,286)`, `(-2174,278)` and `(-2156,278)`, crosses the valley through `(-2134,300)→(-2070,300)`, then proceeds east through `(-2060,292)`, `(-1994,286)`, `(-1940,286)`, `(-1900,314)` and a northern loop near Bartle. The proof preserves every2-unit node and exact collinear turnpoint; it does not smooth across arbitrary nodes.

Its grid-only maxima looked less severe than a subsequent0.5-unit centerline/±0.75-unit corridor check of all45 legs. The finer result has maximum grade0.597576, minimum corridor normalY0.669507 and21 flagged legs, despite selected footprint clearance≥1.8179 and no missing floor. In particular:

- `(-2184,282)→(-2182,280)` reaches grade0.597576; `(-2178,282)→(-2174,278)→(-2172,280)` has side-corridor normalY≈0.770 near the cliff.
- `(-2070,300)→(-2068,298)→(-2066,298)` crosses another steep rise, grade up to0.521555.
- `(-1880,356)→(-1874,362)` near Bartle reaches side-corridor normalY0.669507; the next leg reaches grade0.547067.

This candidate is retained as a map of hazards, not a safe walking route. It needs actual screenshots/terrain inspection and a different descent or normal-jump testing before use. The actual151602 route climbed to Pappy farther east through the disclosed rock/jump gap; reversing that history is also unaccepted, but is a better-informed local starting hypothesis than claiming this new west cliff path is clear.

Grid proof: [relaxed exploration](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/pappy-bartle-route-v1/guide-relaxed-v2.json), SHA256 `3be28f5bc4da0d0af8d48c16ab37b7ebfd5ea74448ecd677e047c31981a6edd3`. Fine review: [all45 candidate legs](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/pappy-bartle-route-v1/corridor-review-v3.json), SHA256 `610dab8cbedff06552604150980f74455d052a1d5af40c89a9379339f962185a`,307 rechecked source bindings. Fine checking reuses the existing geometry mathematics and unchanged community exports. An initially expensive relaxed smoothing attempt was stopped before writing a result; the completed v2 retains the grid and v3 checks its fixed legs once.

## Remaining native boundary

Use the known farm/Pappy historical coordinates and fresh native observations to choose the first descent. Bartle can be approached from a different side within his12m interaction radius; the endpoint above is only one tested western approach. No Pappy→Bartle walking history was recovered from151602;090058's Pappy arrival was a Magic Door zone transition, not a walk. The full onward traversal remains pending rather than accepting either road adjacency or a coarse grid as collision proof.
