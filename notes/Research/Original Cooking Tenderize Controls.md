# Original Cooking Tenderize controls

The first reconstructed recipe phase uses the original Tenderize Bat Meat movie. Play it by following the visible moving meat, allowing the cursor-attached hammer to settle for a rendered frame, then pressing/releasing the left mouse button once. Holding or dragging does not tenderize repeatedly. The power meter is hidden for this round. Eight successful hits complete it.

Original RoundTenderize accepts a click only when the hammer is Idle. TenderizeElem rejects a new hit while its state is Hit or Tenderize. Its collision check is MovieClip bounding-box overlap between the meat clip and hammer.mcHit; it is not a pixel-mask test. Both clip bounds animate, so aim at the visible meat rather than treating the table below as a stationary click macro.

The unchanged movie is768×768 at35fps. The hammer Smash animation is9frames and the Bat Meat Hit animation10frames: nominal0.257s and0.286s respectively. Allow about0.30s plus rendering/update delay between strikes and watch the animations. A successful meat hit starts a new random different destination immediately after its Hit animation. It moves among eight targets, starting at speed15, adding2 per frame up to50. On arrival it normally waits22frames initially, shortened by one per successful hit. A miss still plays the hammer's animation and increments bad actions. These timings come from original clips and reconstructed phase tuning, not a played timing measurement.

Original Lua MiniGameFlash.Resize uses `min(canvasWidth/1024,(canvasHeight-100)/768)` and centers the requested movie dimensions. At1280×960, a768×768 response therefore gives scale1.1197917,860×860 viewport, origin(210,50) in client coordinates. This is an expected mapping: confirm the loaded game screenshot and canvas dimensions before use. Do not apply these as desktop coordinates without adding the game's client origin. If a screenshot is resized, scale its pixels accordingly. No XY offset is assumed.

| Target | Original movie X,Y | Expected1280×960 client X,Y |
| --- | --- | --- |
|mcMove1|278.30,420.30|522,521|
|mcMove2|461.90,427.90|727,529|
|mcMove3|210.20,479.40|445,587|
|mcMove4|169.05,542.90|399,658|
|mcMove5|381.25,488.40|637,597|
|mcMove6|382.80,552.40|639,668|
|mcMove7|582.25,551.15|862,667|
|mcMove8|538.15,489.15|813,598|

These are target registration points. Original tenderize parent scales Y by0.99980164, included in the expected coordinates. Hammer.mcHit's shape bounds±47.65 are transformed by hammer0.69999695 and hitY0.5590515, giving root half extents approximately33.35×18.65 pixels (expected37.35×20.88 client pixels), before the moving meat's additional bounding-box overlap. Phase1 hammer-radius increment is0.

Proof derives from the original JPEXS26.3.0 XML and AS2, without movie edits: sprites2206→1603, BatMeat1585/Hit1581, hammer1642/Smash1641/Hit1605→shape1604. Original movie SHA-2566b105f899451ff131ea0b4dcdb00b13e99dcce64c60c6e8ebe1630d2cc9cbc9c. Preserve `lab/reports/cooking-controls.xml`, `cooking-tenderize-controls-proof.json` and `prove-cooking-tenderize-controls.py`; their ledger hashes original inputs, tooling, Lua, AS2 and current recipe. This research sent no input and changed no asset or save. Native phase play remains pending.
