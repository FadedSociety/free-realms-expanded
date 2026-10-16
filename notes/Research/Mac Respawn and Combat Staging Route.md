# Mac Respawn and Combat Staging Route

Read-only approach from the starting respawn to a staging point outside Mac's reported 40 m pursuit radius. Latest context from root: Mac at `[-1842.8344,-52.71554,350.66458]`, HP 500; player safely respawned around `[-1904.883,412.6024]`, HP 1000; all three artifacts removed. Recheck actual NPC/player positions before using the helper because Mac may move.

[Compact handoff](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/mac-staging-handoff-v4.json), SHA256 `95553b0cf2c78334894f5c583fa0fb6bb85aedaf235d130262f8024bee98fe2d`, binds 426 sources and 14 walking waypoints. The 181.70 m approach remains at least 51.152 m in 3D from the reported fixed Mac position. Maximum sampled grade is 0.34847, corridor normal Y ≥ 0.92848 and no sampled floor is missing. The first northern-boulder detour is actual prior native history; the new road join and helper chord remain candidates pending this run's native walk.

## Respawn, northern boulder detour and road staging

Use the actual 183551 northern boulder backout rather than the old blocked direct west chord near `(-1915.6758,412.31323)`:

|Purpose|X|Z|
|---|---:|---:|
|Known respawn|-1904.883|412.6024|
|Observed northern detour|-1911.2793|417.65683|
|Observed northern detour|-1917.1147|423.47748|
|Observed northern detour|-1922.9386|429.2909|
|Observed northern detour|-1928.8301|435.16827|
|Observed northern detour|-1934.6497|440.85202|
|Observed northern detour|-1940.3143|445.18912|
|New fence/road join|-1940.3143|456|
|New cone clearance bend|-1918|455|
|Road704|-1907.6592|456.4899|
|Road703|-1882.1732|453.847|
|Road713|-1870.3472|443.28207|
|Road714|-1869.4022|403.30176|
|Helper staging|-1868|395|

The native-known first 48.21 m corridor has a narrow original-map triangle clearance of 1.17452 m and stays at least 88.62 m from the reported Mac. That historical traversal is not a universal collider/step guarantee. The new 43.60 m road join passes all original map triangles by at least 2.97816 m, with grade ≤0.34847 and normal Y ≥0.94431. The original direct diagonal join passed only 0.81450 m from a corral fence and was replaced. The northern alternative at Z458 passed close to decorative cones; the final Z456/(-1918,455) bend improves clearance.

Road704→703→713→714→staging is 89.89 m, with grade ≤0.14372, normal Y ≥0.98780 and map-triangle clearance ≥2.27863 m. It ends at render floor `[-1868,-48.51963,395]`, 51.15215 m in 3D from reported Mac. The earlier current-run approach to713 needed a normal jump after the client drifted to the wrong side of a low fence; remain on the original road and inspect any foreground/navigation failure. [[Pappy to Bartle Main Road Detour]] links the retained actual jump input and screenshot.

## Final helper segment

The straight downhill staging→reported Mac chord is 50.97977 m, with maximum sampled grade 0.17978, corridor normal Y ≥0.94757, no original map triangle within 4 m and no missing sampled floor. It avoids the previous artifact blockers, assumed removed here per root. A via-road715 comparison also passes but adds a bend; the direct chord is the smaller static candidate.

The endpoint render floor lies 0.66318 m below the reported Mac Y. This difference is retained without inferring burial or changing NPC coordinates. Actual helper facing, dynamic pursuit, native grounding, collision and time budget are separate runtime checks. The route is not a claim that a 30-second helper will finish or that the previous timeout was a geometry bug. Check fresh range/health before starting, and stop on native input or foreground errors.

The proof uses the unchanged pinned community DME/GCNK exports and existing individual-triangle clipping, not a new asset parser. It checks render floor+0.2…2.2 m, at ≤0.5 m samples with ±0.75 m terrain offsets, and does not fill corral interiors with whole-object hulls. Original Bullet/capsule and low decorative flags are not decoded. No source, content, active save, input, release or shared decoder output changed.
