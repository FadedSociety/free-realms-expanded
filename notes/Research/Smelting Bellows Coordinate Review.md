# Smelting Bellows Coordinate Review

For the next original Smelting77 HEAT phase at a nominal1280×960 client, use **open(843,515) → compress(740,534)**. No mouse button is required. Pause about0.3seconds at each endpoint so the native update loop can observe both positions. This dwell is a conservative test choice, not an original minimum. Verify that opening widens the bellows and compression narrows them while the thermometer rises. Four nearly full cycles should reach the current100-heat goal; wait for the thermometer burst/completion feedback, then click once to advance. Fresh native acceptance of this margin pair remains pending.

This supersedes the earlier unresolved numerical/art-pivot alternatives in [[Smelting Native Control Geometry]]. The movie, source, content, checkpoints and native client remain unchanged.

## Why the numerical pivot applies

Original `SmeltingGameClient.initRound` gives the heat-state clip and its identity-transformed `mcCatcher` to `Smelting.RoundHeatFurnace`. That class inherits the original SpecialRounds heat control without overriding mouse movement or `doAction`. `Core.Round.onMoveMouse` reads `mcCatcher._xmouse/_ymouse`; `RoundClick.mouseMoving` calls `mcRef.globalToLocal(oPos)`. The heat angle compares that `oPos` directly with `mcFollow._x/_y`, numerically(488.75,251.95).

The authored `mcFlameControl` has an X translation−144, but the mouse is never transformed into that nested clip's coordinates. Its rendered artwork pivot is therefore unsuitable as the control pivot. The former alternative(682,515)→(596,534) calculates about112°→129° under the nominal mapping. Both points stay on the compression side and never rearm at≤80°. An isolated projection permits at most the initial heat gain from that repeated alternative.

The root `mcGame` hasY0.05 and the heat-state scale is(1.0002747,1.0001373). Accounting for both mouse conversions gives nominal radius180 targets:

| Angle | Nominal client coordinates |
| --- | --- |
|65° open |(842.830,515.049) |
|90° threshold |(757.599,533.939) |
|92° compress |(750.561,533.816) |
|95° compress |(740.022,533.172) |

The recommended integer pair gives rounded65°→95° at the outer Lua mapping. Original heat gains once at≥90° and rearms at≤80°. The100-frame bellows clip reaches frame1 when fully opened; its original gain equation produces29.7heat per such stroke, before decay and goal capping. Four source-equation cycles produce118.8gross heat. The current reconstructed phase row has goal100 and loss1heat/second.

## Native calibration and limits

Original Lua calculates an **outer**860×860 window at(210,50) from the768×768 movie response. The unchanged original `wndMiniGameFlash.xml` uses a Dialog window and an expanding Flash child. This does not by itself prove the child's exact runtime raster rectangle. Both saved173814 HEAT screenshots are1279×960 rasters, although the intended control canvas is1280×960.

The self-contained original mouse-icon sprite415 was exported with pinned official JPEXS26.3.0 and compared against both saved screenshots using an opaque-alpha RGB mask. Sampled best matches were scale1.07/56×86px at(862,529) and scale1.09/57×87px at(846,528), versus the outer Lua scale1.11979. This identifies the original icon and suggests roughly3–5% smaller inner rendering. Raster rounding, interpolation and the animated mouse frame prevent an exact layout/padding attribution. A native inset has **not** been decoded or proved.

To avoid relying on an exact threshold, the isolated review samples a disclosed symmetric inner-inset envelope0,4,8,12,16,20,24pixels. This is an uncertainty envelope, not discovered layout code. The recommended pair remains on the correct sides throughout: open62.21°–64.95°, compress93.26°–94.99°. The old x757 compression point becomes88.23°–89.22° at the larger samples and may fail to compress. The x750 point also passes, but x740 supplies more margin.

The173814 input log row53 records eight(843,515)→(757,534) pairs at17:58:16.455UTC. The wide and compressed native images preserve a visible bellows-pose change; that record does not establish that all input pairs were sampled on separate rendered frames. Rapid events may coalesce. The authoritative ended-run audit already accepts original HEAT statistics157seconds and full Smelting completion, without identifying who operated the effective controls. This coordinate review does not reattribute that completion.

Composite JPEXS bellows/scene PNG exports contain red placeholders for external DDS artwork. They are retained as an exporter limitation and are not used as native pixel oracles. Only the self-contained mouse415 export is used for approximate matching. No new asset parser or broad asset recovery was introduced.

## Reproducible evidence

Final private proof: `lab/reports/smelting-bellows-geometry-v3/guide-v4.json`, SHA256 **e808220f10f308202bb29bc8122aa3e300f06e0fd3d09b2dd01ef42999de973f**. It binds34 source/output hashes, the unchanged original movie/XML/helper, original UI/Lua/AS2, both native images, recorded input and the prior authoritative completion proof. `review-smelting-bellows-geometry-v4.py` refuses to overwrite an existing proof and checks the angle/rearm discriminant, four-cycle projection, seven-sample margin envelope and matching source UI bytes. Earlier v3 proof/script are preserved.

Original movie SHA256: `e755dfe1b42be08e39ddeda43962f06aece2b0f3abe822378aac678f6ad10f80`. Original XML SHA256: `92eb7a4ceebde876dfb2d7fabf2eb023d1d4269a828d82e680593f0d1f11e32f`. Cooking reviewer independently confirmed the original inheritance, two mouse conversions, numerical pivot and rejection of the art-pivot alternative. Native pumping of the recommended margin pair is the remaining acceptance gate.
