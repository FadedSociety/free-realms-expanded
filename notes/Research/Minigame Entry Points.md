# Minigame entry points: kart tracks and derby arenas

T221 research, 2026-10-20. Scope: where each kart track and demolition-derby arena is entered, which
drawable driver stands there, and how the existing NPC menu starts the race. No packets or gameplay
rules changed; the entry path remains the 26/9 menu -> 26/10 choice handled by `NpcInteraction` and
`KartGateway`/`DerbyGateway`.

Tags: **[client]** extracted client asset or native client code/locale observation, **[wiki]** fan
wiki archive, **[video]** reviewed video frame from the build guides, **(R)** reconstructed server
policy. Client evidence beats wiki.

## Method and source assets

- **Client zone objects.** `lab/reports/world-objects-20261015.json` is the catalogue produced by
  `scripts/world_objects.py` from the client's GCNK terrain chunks (the v6 parser follows the
  MIT-licensed ForgeLightToolkit layout; object records carry the model file, position, rotation and
  object id). Source chunks are the extracted `lab/extracted/osfr-v1.0-assets/OSFR/<pack>/*.gcnk`
  and `lab/extracted/client-assets/pack_*/*.gcnk` files. These are the positions used below.
- **Client model table.** `lab/runs/client-baseline-20261014T203930Z/client/Resources/Models.txt`
  maps model ids to files; `lab/extracted/client-assets/catalog.tsv` confirms the files ship.
- **Client locale.** `lab/reports/locale-id-to-text.json` supplies every NPC name id quoted here.
- **Wiki.** `lab/references/freerealms-wiki/wiki.sqlite`, page `Speedways` (page id 2597): five
  speedways — Tangletrack (Briarwood), Frost Ridge (Snowhill), Wildwood (Sanctuary), Thunder Falls
  (Lakeshore), Sandstorm (Seaside); "All speedways, excluding the Sandstorm Speedway, offer both Kart
  Racing and Demo Derby"; "Wildwood Speedway and Thunder Falls Speedway are both the same demolition
  area." [wiki]
- **Build guides.** `notes/Build Guides/Content/Jobs/Kart Driver.md` and
  `Demolition Derby Driver.md` (trainer/track lists), `notes/Build Guides/Gameplay/Kart Racing.md`
  ("five speedways, each with a kart gate ... an interaction prompt named after the gate ... above
  the race arch"). [client][wiki][video]

The open-world portal is the **turnstile pair** (`sg_turnstyle_01/02`, client assets
`sg_turnstyle_01.adr`, `sg_starting_turnstyle.adr`) beside the speedway arch/start flags. The client
UI that consumes these is `UI/Resource/ActivityPortal.gfx` (`ActivityPortal.lst` ships beside it).
[client]

## Kart entries

Portal positions are the client turnstile/arch objects nearest the speedway; the driver position is
the existing regional NPC record (`data/content/npcs/*.json`). Distances are from that record to the
object. Each record is `placement: legacy_data`, so the client is sent its AddNpc actor; the model id
and file are verified in `Models.txt`.

| Track (zone) | Entrance objects [client] | Driver NPC [client] | Model |
|---|---|---|---|
| Wildwood (Sanctuary / Sacred Grove) | `sg_raceway_01` (-2262.71, -16.44, 585.04); `sg_turnstyle_01` (-2268.80, -16.47, 587.89), `sg_turnstyle_02` (-2268.81, -16.37, 586.81); start flag `sg_raceway01_flag01` (-2269.7, -12.31, 585.91) | **Johnny Thunder** 9500000911, (-2255.642, -16.614, 579.877), 8.8 m from the arch; locale 20061 | 360 `human_m_racer_johnny_thunder.agr` |
| Frost Ridge (Snowhill) | `sg_raceway_01` (-237.60, 1.05, -51.89); `sg_turnstyle_01` (-231.10, 1.15, -52.51), `sg_turnstyle_02` (-231.48, 1.15, -51.50) | **Mustang Alli** 9500000759, (-237.116, 1.1, -46.706), 5.2 m from the arch; locale 100843 | 820 `fairy_f_mustang_alli.agr` |
| Thunder Falls (Lakeshore) | `sg_turnstyle_01` (-915.95, -39.16, 1205.96), `sg_turnstyle_02` (-915.64, -39.17, 1205.46); derby sign `sg_destruction_derby_01` (-911.67, -39.82, 1213.06); start flags `sg_raceway01_flag03` near (-905.6, -27.0, 1192.4) | **Chase Wheeler** 9000000273, (-912.977, -37.689, 1192.06), 13.7 m from the nearest turnstile; locale 395167 | 1317 `human_m_chase_wheeler.agr` |
| Tangletrack (Briarwood) | `sg_turnstyle_01` (-271.49, -58.86, 1952.21), `sg_turnstyle_02` (-271.11, -58.83, 1953.23); derby sign `bw_destruction_derby_01` (-264.65, -58.96, 1947.87); start flags `sg_raceway01_flag03` near (-269.7, -40.8, 1963.7) | **Cale Yardbird** 9000000039, (-276.710, -59.532, 1964.762), 12.8 m from the nearest turnstile; locale 141805 | 360 `human_m_racer_johnny_thunder.agr` |
| Sandstorm (Seaside) | `ss_raceway_01` (31.6, -14.6, -1192.3); `sg_turnstyle_01` (37.06, -14.89, -1189.44), `sg_turnstyle_02` (37.21, -14.72, -1190.02); garage props around (10-20, -14, -1180..-1200) | **Spud Edgins** 9500000630, (28.754, -15.086, -1207.71), 15.7 m from the arch; locale 407192; guide calls him "Sandstorm race-area NPC" | 360 `human_m_racer_johnny_thunder.agr` |

Client track scenes: `sg_kt_track.gzne` (the scene named by the shipped `kart_entry.json` context),
plus `sg_kt_track2.gzne`, `bw_kt_track.gzne`, `sh_kt_track.gzne`, `ss_kt_track.gzne` and their
`*_kt_trackAreas.xml` area tables. The Sacred Grove scene also carries an in-scene
`sg_kt2_entranceroad_bbec.adr` at (340.0, 10.2, 500.0) and `sg_kt2_gates_bbec` — those are track
geometry, not open-world portals. [client]

T221 keeps the three reviewed gates (Chase Wheeler, Cale Yardbird, Mustang Alli) and adds Johnny
Thunder (Wildwood) and Spud Edgins (Sandstorm) to `data/content/kart_gates.json`: one driver at every
kart portal. The server still races the one reconstructed `sg_kt_track` scene from every gate (the
existing behaviour); per-track scene selection is not implemented.

## Derby arena entries

All speedways except Sandstorm host a derby. The client arena geometry is scene-owned:
`sg_dm_track_bbec_101` (Sacred Grove; serves Wildwood and Thunder Falls, the same demolition area per
the wiki), `sh_dm_track_bbec_311/312` (Snowhill/Frost Ridge), `bw_dm_track_*` (Briarwood/Tangletrack).
No derby arena exists at Seaside in the shipped client art. [client][wiki]

| Arena (zone) | Entrance objects [client] | Driver NPC [client] | Model |
|---|---|---|---|
| Wildwood (Sanctuary / Sacred Grove) | `sg_raceway_01` (-2262.71, -16.44, 585.04); turnstiles at (-2268.8, -16.4, ~587) | **Mad Jack** 9000000391, (-2253.199, -17.164, 590.133); locale 434622 | 1323 `human_m_mad_jack_01.agr` |
| Thunder Falls (Lakeshore) | `sg_destruction_derby_01` (-911.67, -39.82, 1213.06); turnstiles at (-915.8, -39.2, ~1205.7); shares the Wildwood demolition area | **Crash Davis** 9000000289, (-913.768, -38.165, 1195.946), 17.3 m from the sign; locale 419307 | 830 `human_m_crash_davis.agr` |
| Frost Ridge (Snowhill) | `sg_raceway_01` (-237.60, 1.05, -51.89); turnstiles at (-231.3, 1.15, ~-52.0) | **Burt 'Wrecking' Crewe** 9500000731, (-237.309, 1.1, -61.321), in front of the gates per client string 2360694093; locale 100244 | 829 `human_m_burt_crewe.agr` |
| Tangletrack (Briarwood) | `bw_destruction_derby_01` (-264.65, -58.96, 1947.87); turnstiles at (-271.3, -58.9, ~1952.7) | **Rip Ringa** 9000000183, (-286.567, -59.556, 1949.635), 22.0 m from the sign; locale 100248 | 835 `human_m_rip_ringa.agr` |

`data/content/derby_gates.json` is new with these four ids and the existing Demo Derby Lobby label
(3053). The shipped package has **no** `derby_entry.json`, so `DerbyGateway::load` returns `None` and
no driver offers the derby today (the existing dormant-content rule). When an operator adds a derby
package, the same gate list applies. No derby package was invented here: the client arena assets and
the recovered type-11 lifecycle exist, but the vehicle/rule values are unrecovered. T219 (integrated
after this research was written) owns the payout; the T221 entry test package sets its completion
rewards to 0 so it exercises entry only.

## Drawable driver look (no new art)

Every gate NPC is an existing, already-spawned speedway driver, so no new actor, model or outfit is
needed and no second named character is duplicated at a portal:

- Kart: `human_m_racer_johnny_thunder.agr` (Johnny Thunder, Cale Yardbird, Spud Edgins — the same
  racer body the client uses for the kart trainers), `fairy_f_mustang_alli.agr` (Mustang Alli),
  `human_m_chase_wheeler.agr` (Chase Wheeler).
- Derby: `human_m_mad_jack_01.agr`, `human_m_crash_davis.agr`, `human_m_burt_crewe.agr`,
  `human_m_rip_ringa.agr`.

All seven files are in `catalog.tsv` and the matching model ids are in `Models.txt` (ids above). The
content test emits the production `NpcVisibility` actors and asserts the model id/position and the
`npc_id -> npc_id` click identity, so a gate id that is not a drawn driver fails the test.

**(R)** If a future speedway loses its driver, add a spawn row to `data/content/npc_spawns_extra.json`
with `content_key` = the NPC id, a positive model id from `Models.txt`, a `Global.Text` name id and
five provenance entries (the T201 Warrior-trainer shape); the gate list needs no code change.

## Minimap launch (skipped)

- The race HUD includes a minimap. Client **33/34** (parser `0x00ba0680`) carries a dword at
  `+0x34` that the dispatcher hands to `RaceMiniMap:SetTrackId` (the binding string is at
  `0x01824acc`; `notes/Build Guides/Gameplay/Kart Racing.md`). The per-speedway track-id values are
  **not recovered**, so a per-track minimap launch is not implemented. `RaceMiniMap` also has
  `SetRacerLocation` (0x01824914); the race scene already drives the HUD from the 33/x stream.
- The open-world **ActivityPortal** UI (`ActivityPortal.gfx`) is not what starts kart/derby today:
  `src/activity_portal.rs` publishes only battle, chess and soccer rows, and its own comment records
  that races are deliberately not listed, so a `167/1/2` Play cannot start a race. The NPC menu
  remains the only entry (T221 adds no second path).

## (R) reconstructed rules

- The **portal area** is the turnstile pair plus the raceway arch/sign; the 25 m test reach is a
  reconstruction for the content test only. The server's interaction reach is the existing 12 m from
  the NPC record (tested through the real path).
- The gate menu label is the existing client lobby label: 3052 "Kart Race Lobby" for kart, 3053
  "Demo Derby Lobby" for the derby.
- No level gate exists on the kart or derby NPC entry path today; `begin_kart_race`/`begin_derby`
  unlock and switch the job idempotently, and the shared busy table, race-content check, already-
  racing check and the gateway account action budget are the checks that apply. T221 keeps them
  untouched.
- The NPCs are stateless per player: each session's `Gameplay` has its own `KartGateway`/
  `DerbyGateway`; the test starts two players' entries from the same driver and asserts the first
  session is unchanged by the second.

## Open gaps

- No native client run in this task (the lead owns the client window). The models are the client
  model-table bodies for the existing trainers, not a live draw check.
- Per-track race scenes are not selected by the server; every gate loads `sg_kt_track` (existing
  behaviour, out of scope for the entry task).
- No derby content package ships, so the four derby drivers correctly offer nothing until an operator
  adds `derby_entry.json`.
- The minimap track id per speedway and the exact client prompt text per gate remain unrecovered.
