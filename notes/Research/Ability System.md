# Ability system: W17 first family

Status: isolated Rust candidate for lead review; not registered or native-accepted.

## Client facts and provenance

The client has no shipped ability/effect balance tables. Definitions arrive from the server as 36/13; the job toolbar arrives as 36/5. The client sends use requests as 36/10. The guide records the network direction and field summaries in [[Build Guides/Gameplay/Combat and Abilities#Network]].

* The native 36 dispatcher is `00a35cc0`, case 3 reads StartCasting through `00a31aa0`/reader `00a2fc70`, and case 1 reads Failed through `00a31470`. `00a2fc70` reads two 8-byte identifiers, four dwords and a byte; the guide identifies the cast-time float and client-side animation/effect uses. The Rust builder preserves that shape; ability/animation/effect dword assignment is partly (R).
* Ability definition parsing is `00a32930`; effect parsing is `00a30360`. `nmq dis` checked both accepted-unknown labels against bytes: the routines perform bounded fixed-width reads. Field roles beyond the named fields in the guide remain unknown. `00a309c0` and `00a30d20` expose the client names `CastSeconds`, `ManaCost`, `AuraDuration`, `ManaCostPerSecond`, `MaxAoeTargets`, `ChanceToOccur`, `DurationSeconds`, `MaxCharges` and `Param1..3` in inferred named-field builders. Do not treat those names as recovered server rules.
* The client reads 32/7 with `00a2a910`; handler `00a2ba40` builds a damage-number event, health-bar event and reaction/effect work. `nmq dis 00a2ba40` confirmed the handler constructs the family-32/sub-7 result and parses through `00a2a910`. The local writer follows the existing encounter writer's 32/7 field order. Critical, reaction and unclassified values are fixed to zero (R).
* The 36/1 failure body is one `i32` read by `00a31470`. Shipped locale ids are 285 (not enough energy), 287 (ability not ready) and 288 (choose a target); 286 says “You can't use abilities right now.” Choosing 286 for ownership/job/slot/range/cast-state rejection is server policy (R), not recovered server behavior.
* Family 38 sub 1 supplies current and maximum health; sub 13 supplies energy fields. Family 35 sub 5 updates a non-local actor's health, and 35/35 displays a signed hit-point delta. `00a2e540` disassembly confirms a client lockout against its millisecond clock with a 1,000 ms threshold; this does not recover a server cooldown.
* Effect tags use 38/16 and 38/17; native dispatch is `009392c0`, the 38/16 reader is `008d8870`, and the client map helpers include `00a19e70`/`00a15440`. Existing `restoration/piece_056.rs` writes the 85-byte tag row. Its four interpreted fields are duration, icon, name and composite-effect id; other row members stay zero (R).
* Knockout uses status bit 7 in 35/20, then 38/1, then encounter 41/125 with timer and revive cost. Dispatcher `0090d830` handles 41/125; `00904300` displays the timer/cost values. Their server values and revive policy are (R).
* Medic profile 11 is from the client `ItemFilterOptions.txt` table summarized in the combat guide. First Aid's rank-1 healing text is client locale 4168670255; Medic's job page also lists tier-1 Incision and Triage. The fixture preserves those names and the displayed First Aid amount. The Incision/Triage magnitudes come from wiki starter-weapon material and remain balance hypotheses (R).

These facts are from the reviewed native map at `lab/native-map/run/map.sqlite`, the current build guide, client locale and job content guide. The client predicts required response shapes; it does not reveal the original server's validation or balance policy.

## Candidate model

`src/abilities.rs` loads bounded JSON whose gameplay values each carry `{ "value": ..., "source": ... }`. It stores actor health, energy, job profile, owned ability ids, assigned slots, slot cooldown deadlines, cast deadline and active effect tags. `use_ability` checks ability ownership, slot binding, job profile, caster/target health and kind, range, energy, cooldown and active cast before mutating either actor. Rejections return a typed failure plus one 36/1 packet; each path leaves actor state unchanged.

Success packets are inner gateway messages in this order: optional 36/3 cast start, 32/7 plus the appropriate health update (or 35/35 plus health for healing), 38/13 after a nonzero energy cost, then 38/16 for applied tags. Lethal player damage inserts 35/20, 38/1 and configured 41/125 in that order. `tick_effects` removes elapsed tags with 38/17.

All server-side numbers and rules remain (R): distance metric and range, slot ownership, costs/cooldowns/cast deadlines, charging at request acceptance, response order, immediate effect resolution after a cast-start message, generic failure-code routing, stable tag ids, reapplication refresh, expiry semantics, the 35/35 delta ordinal, the second 38/13 dword, duration rounding and unknown packet fields. The fixture uses local ids 10001–10003 because original ability ids are not recovered. Its profile id and cited locale/name facts are client evidence; its toolbar placement, ranges, costs, timers and balance are explicitly (R).

## Integration boundary

1. Lead registers the module and owns gateway dispatch. Resolve request bar/slot/target mode from the last 36/5 definition; never trust the client request as ownership proof.
2. Join the module's state to session-owned actor/equipment state. Recheck weapon, level, line of sight, loading state, status mask and authoritative target disposition when those existing APIs are ready.
3. Keep the current `combat.rs` basic attack route and native acceptance intact. Reconcile shared cooldown and health ownership before connecting these messages to `gateway.rs`.
4. Add delayed cast completion/interruption, encounter versus overworld knockout behavior, status immunity/stacking, projectile/AOE resolution, multi-actor effects, save/reload and native-client acceptance before claiming complete combat.

The 36/5 builder in `startup_catalogues.rs` sends typed slot records but explicitly leaves kind-specific tails opaque; this module does not change or claim to serialize a complete original ability set.

## Open gaps

No native ability request/result capture or live-client ability acceptance was run in this worker scope. Original ability ids, toolbar records, most values, damage formulas, effect semantics, valid range/line-of-sight, server cooldown units, cast completion timing, effect stacking, and failure routing are not recovered. The 36/3 identifier/dword roles are only partly interpreted. Cooldown deadlines are enforced in actor state, but no 38/25 slot refresh or 36/4 timer packet is emitted because this model lacks the recovered icon/name/selector fields and bar identity. Player knockout in an overworld needs its distinct 41/132 path; this slice only emits a configured encounter 41/125 window. Lethal hostile removal and defeat/reward handoff remain owned by the encounter/combat integration.

## Verification

* Formatted only `src/abilities.rs` and `tests/abilities.rs` with the private toolchain's `rustfmt.exe`.
* `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W17; .\scripts\cargo.ps1 test --test abilities --manifest-path .\prototype\osfr-tools\Cargo.toml` — final run passed 8 tests, 0 failed. Cargo emitted one unrelated existing warning at `src/housing.rs:577` (`instance_index` unused).
* The first Cargo attempt exited before the test target because `src/account.rs` and `src/account_store.rs` could not resolve `osfr_tools`; repeating the same scoped command later compiled and passed. This task did not edit those shared files.
* An isolated `rustc --edition=2024 --test` build of `tests/abilities.rs` using the private toolchain and Cargo-built `serde`/`serde_json` artifacts also passed 8/8; the requested Cargo target is the final acceptance check.
