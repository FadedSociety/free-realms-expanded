# Stage7 Native Return Routes

Read-only route follow-up for wolves → Cleetus/new anvil → Pappy. The return combines proposed short connectors with actual historical native walking. **There is no new native traversal acceptance in this review.** The native operator controls all input and campaign state. Y below is observed player altitude or sampled render floor, never an instruction to set altitude.

## Short return from the later fight position

Root subsequently reports all five wolves defeated with ordinary input and player `[-2061.9998,-45.539227,437.4896]`. This position is east of the old tree, so the earlier west detour is unnecessary from here. The root-proposed short campsite connector passes the existing static geometry checks:

`(-2061.9998,437.4896) → (-2072,420) → (-2080,400) → (-2088,380) → (-2088.2224,368.29822)`.

Sampled render floors are−45.581955,−48.319850,−53.934550,−57.284668 and−56.863082. Maximum grade0.441820 and minimum corridor normalY0.900566 occur on the middle descent. Selected footprint clearance is at least3.054817; no selected prop is within4m on the first two legs, and there are no missing floor samples. A medium tree at `(-2080.822,378.58197)` stays at least3.05m from the later centerline. Use bounded actual walking, then the reverse observed farm backbone below. Original Bullet collision and native traversal remain unproved by this geometry review.

Proof: [postcombat return candidate](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-postcombat-return-v1/guide.json), SHA256 `2c64c742edcafc4fd38c4d4e5767b78e1db51aa28d0c42c75cb8c132fa783e6d`. Earlier wolf-side routes remain preserved for their different starting positions.

## Wolves back to the farm

Actual181715 stopped at `[-2098.401,-52.621204,423.78705]` beside the tree. This invalidates the old straight campsite-to-wolves connector in [[Farnum Stage7 Route Geometry]]. Starting at the current southwest approach `(-2108,432)`, the following west-side valley detour avoids that blocked point by at least6.509 horizontal units:

| X | Render floor Y | Z |
|---:|---:|---:|
|-2108|-51.698166|432|
|-2104|-53.290638|420|
|-2102|-55.749756|400|
|-2100|-57.291140|380|
|-2088.2224|-56.863082|368.29822|

This proposal has maximum sampled grade0.305801, minimum ±0.75-unit corridor normalY0.952210, selected ground-band footprint clearance2.594348, and no missing sampled floor. Routes farther west at X−2112…−2120 were rejected because they climb steep slopes. These checks reuse original community terrain/mesh exports; the original reader does not decode Bullet collision, so bounded walking and fresh position/screenshots remain necessary. The approach is close to pursuing enemies and is not an idle safety zone.

From the campsite, reverse the actual arrived181715 farm backbone:

`[-2088.2224,-56.820347,368.29822] → [-2083.688,-55.694443,340.82516] → [-2080.2969,-56.293385,270.49878] → [-2083.4875,-55.164326,201.49933]`.

Those are observed forward arrivals, not exact reverse-input replay. Follow the known visible corridor around tents, crates and the farm stump. The new anvil is at `[-2088,-54.79,200]`; its native render/approach acceptance is separate from this route review and [[Anvil Placement Repair Proposal]].

Proof: [wolf/farm geometry and native endpoints](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-return-routes-v2/wolf-farm-guide.json), SHA256 `b5d2d410e9c572f52648d11d82444808a90eeb4404c98af825458efd33133306`.

## Farm staging to the historical Pappy approach

A proposed connector from the farm staging point to the old fire1/well area is:

`(-2083.4875,201.49933) → (-2086,213) → (-2095,220) → (-2109,222) → (-2121.7434,216.48042)`.

Its maximum grade is0.192087, minimum corridor normalY0.967754 and minimum selected footprint clearance2.105837, beside the fallen log on the first leg. This connector is not a historical native walk. Inspect the local log before departure. A shorter initial proposal with only1.191 units of log clearance was rejected.

The following151602 path is recovered from actual native navigation arrivals:

`[-2121.7434,-56.44453,216.48042] → [-2131.79,-56.68549,224.51976] → [-2125.3047,-55.953716,235.63026] → [-2129.9429,-55.266953,271.21738]`.

The next old target `(-2160,306)` did **not** arrive. Its useful moving samples approach the tree through:

`(-2132.5557,274.4471) → (-2135.134,277.63483) → (-2137.7112,280.8209) → (-2140.292,284.01306) → (-2142.889,287.22397) → (-2145.4568,290.39966)`.

It then stalled near `(-2148.0813,295.6197)`. A recorded normal backward S400ms command at15:27:06 is bracketed by that final position and the next observed `[-2145.8533,-56.617928,293.62283]`. From there, ordinary walking moved through `[-2144.3918,-56.197502,297.44202]` and `[-2142.8962,-55.948322,301.33954]`, then stalled at `[-2140.045,-56.173893,304.37207]` in front of the fallen log. A second normal backward S550ms at15:31:22 preceded the next observed `[-2141.65,-56.11437,300.16614]`, the start of the successful climb below.

These backouts are original native input evidence, not per-frame reconstructed paths. Do not aim blindly through either stopped point. Their headings depend on the original facing direction; the useful coordinates identify the southern side of the log where the accepted climb began. Three attempted new shortcuts from fire3 to that start overlap the coarse convex hull of the hollow corral. That proxy neither proves a solid wall across its interior nor justifies calling the shortcuts safe; the observed path and stops take precedence.

## Recovered southern climb and Pappy interaction

The successful151602 climb preserves the following observed coordinates in order:

| X | Observed player Y | Z | Evidence |
|---:|---:|---:|---|
|-2141.65|-56.11437|300.16614|Start after normal backward movement from log|
|-2155.5894|-51.746815|299.57816|Arrived by ordinary walking|
|-2157.8132|-51.324203|303.33743|Arrived by ordinary walking|
|-2162.7646|-49.64771|307.3536|Arrived by ordinary walking|
|-2164.8489|-48.630516|306.63095|Arrived by ordinary walking|
|-2163.988|-50.006493|301.88248|Walking stalled on the rocky slope|
|-2172.5498|-46.890453|293.8558|First observed position after recorded normal jumps|
|-2173.7085|-43.985966|296.73724|Arrived by ordinary walking|
|-2173.284|-34.78789|317.2152|Arrived by ordinary walking|
|-2183.0903|-31.72443|327.39545|Pappy interaction accepted|

The jump gap has three recorded Space presses: forward+Space950ms at15:38:11, then forward+Space1000ms, forward100ms, forward+Space1000ms at15:38:24. These are ordinary player controls. Exact intermediate landings and the minimum necessary number of jumps are not proved. After the gap, the last two walking segments climb along X≈−2174 toward Pappy. The final position is1.603558m from actual Pappy `[-2184.513,-31.53728,328.1112]`, inside his12m interaction radius; `farm-pappy-menu_small.png` confirms the interaction.

Peer recovery: [151602 route/input proof](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/farm-pappy-native-route-151602.json), SHA256 `48f19a51692192a5db41ba2387ec7fb00ca4405e64a46cdb8c92a0fedacbbebb`. Its source_opening_sha2564e6ac15c identifies the current coordinate-comparison content; historical151602 manifest instead names `rust-opening-forging-assets`. Do not describe4e6ac15c as that historical run's content hash. The earlier090058 arrival used a Magic Door zone transition and is not walk-route evidence.

## Verification and remaining gaps

[Compact independent history review](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-return-routes-v2/history-review.json), SHA256 `69f42a954fc009aa068c375d08940d202273ca70b118ea50bca875c20b6118c8`, rechecks31 source hashes including geometry inputs, native navigation logs, external input log and accepted Pappy screenshot. It verifies the stop/arrival distinctions, the two normal backouts, three recorded Space presses and actual Pappy distance. Reproducer: `lab/reports/finalize-stage7-return-route-review.py`. No source, content, save, native input, active analyzer or shared progress file changed.

New wolf/farm and cross-farm connectors still need actual walking; historical waypoint chords and reverse directions are not new collision acceptance. No successful native Pappy→Bartle route was recovered. Bartle's configured/community position is `[-1860.185,-51.30442,341.1369]`; [[Pappy to Bartle Route Candidates]] records the bounded original-reader search, road fence obstructions and finer rejection of a relaxed cliff candidate. Complete native descent/road/approach remains pending rather than assuming a straight line down the plateau.
