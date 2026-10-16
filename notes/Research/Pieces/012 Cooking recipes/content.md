# 012 Cooking recipes and content

Work item: inventory the recovered Cooking recipe (materials, result, locale)
and check that repeated recipe transactions stay atomic and once-only.

Status: code plus tests returned for lead review. The recipe facts below are
labelled as recovered, inferred or reconstructed. Native acceptance is not
claimed here; this piece adds no game data and touches no shared file.

## Recovered recipe facts

One Cooking recipe is recovered with usable provenance: the tutorial recipe
**Strong Arm Stew**. The preserved wiki revision lists 25 Chef recipes in
total; none of the other 24 has a recovered item definition or recipe binding
in the preserved corpus, so this inventory covers exactly one recipe
(`other_recipes_unbound`).

| Fact | Value | Evidence |
|---|---|---|
| Recipe name | Strong Arm Stew | locale id 1232 (item name) and wiki heading |
| Where to find | Tutorial | wiki `List of Recipes (Chef)` revision 45318, line `Where to find:` |
| Effect | Increases health and size | same wiki revision; item description id 3172 says eating the food increases the player's health and size |
| Materials | 4x Bat Meat (2862), 4x Bundle of Gummyfruit (2875), 4x Bundle of Rainleaf (2884), 4x Spice (2900) | wiki ingredient line matched name-for-name against the preserved item definitions |
| Product | 1x definition 35 Strong Arm Stew, `class_id` 9, `max_stack` -1 | preserved item definition blob; wiki recipe is the only recovered Cooking recipe row |
| Minigame rows | MiniGameData 46 and 560, both type 9, name_id 386023, description_id 20520, icon_id 916 | `Resources/MiniGameData.txt` sha256 `8ce3cc61...`, exported inventory `../lab/reports/minigame-content-inventory.json` |
| Minigame locale | "Cooking: Strong Arm Stew"; "Stir up some flavor in this cooking game! Slice, chop and stir your way to delicious results." | `../lab/reports/locale-id-to-text.json` |
| Material locale | Bat Meat; Bundle of Gummyfruit; Bundle of Rainleaf; Spice | plural markup `{n}Bundle of Gummyfruit{pl="Bundles of Gummyfruit"}` normalised by the checker |

Source hashes used by the checker:

| Source | sha256 |
|---|---|
| `../lab/references/rust-cooking-isolated/cooking.json` | `60d208f566699124272b9a00e7e2578f1512a747ca160700151de7bdaeecdd93` |
| `../lab/references/rust-cooking-isolated/cooking-item-definitions.json` | `a6b6270a1baa8c384208cc9cf718c2bcc90612a43b9f9c7b704f5f0688b98959` |
| `Resources/MiniGameData.txt` | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| wiki `wikitext/2131-45318-393e86050956.wiki` | `393e860509568de98e263fc5429a984dc283f7524b6443c15b52f88c53f2d1e1` |
| wiki `wikitext/1974-46136-870b47c6896d.wiki` (Chef page) | `870b47c6896dd4432808ef10278cb5b6d42f99fb974ff360858c270b9bf925a6` |

Wiki text is community CC-BY-SA; the item definition blob is pinned from
`https://github.com/raisingkaines/FreeRealms-Legacy` revision
`9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0). Raw assets and
decompiled material stay in `../lab`.

## Boundary: original, inferred, reconstructed

Original (recovered) facts: the wiki recipe row itself (name, tutorial source,
four materials with quantities, effect), the four ingredient definitions and
the food definition, both MiniGameData candidate rows and their locale text,
the six original stage types the movie can run (`[1, 2, 3, 6, 17, 7]`), the
movie's ingredient art ids `[10, 24, 40, 49, 46, 46]` and the original client's
zero recipe quantity in `FRServer_GameWon`. Which stage type runs in which
position, and the phase tuning, are reconstructed.

Inferred: item definition `icon_id` values index an icon atlas, not the locale
table (locale id 993 is unrelated text while definition 2862 stores it as its
icon); phase `data[0]` values are movie ingredient art ids, not item
definitions, because only the four materials and the product have preserved
definition rows.

Reconstructed: the six-phase order and all numeric tuning (90-second nominal
columns, tenderize/chop/smash/measure/stir values, scores, meter speeds), the
fixed one-item output, and the local starter binding to row 46. The supplied
`cooking.json` labels these with `reconstructed_phase_values: true` and a phase
notice; the checker fails if that label disappears.

Explicit gaps, reported as findings rather than invented facts:

- `original_recipe_rows_missing`: the original `MimicMiniGameData` /
  `MimicMiniGamePhaseData` recipe rows for game 46 were not recovered.
- `recipe_id_unverified`: prior project text claims recipe `55824` for Strong
  Arm Stew; that identifier appears nowhere in the preserved wiki corpus,
  the decompiled client, the resources or the exported content, so this piece
  does not assert it as recovered.
- `other_recipes_unbound`: the other 24 wiki Chef recipes have no recovered
  definition or recipe binding.
- `phase_ingredient_art_ids`: phase ingredient art ids have no definition rows.
- `icon_ids_not_locale_keys`: icon ids are not locale lookups.

## Atomic repeated recipe transactions

`prototype/osfr-tools/src/restoration/piece_012.rs` (not yet registered in
`lib.rs`; the owned test uses a `#[path]` include) drives the unmodified
production `osfr_tools::cooking` service through repeated complete sessions:
native start, phase data, six ordered round report groups of four statistics,
four final statistics, `FRServer_GameWon`, a replay of that victory, a reload
replay, and the movie's `FRServer_GameClose`.

Checked invariants per cycle:

- exactly one transaction; consumes `4x 2862/2875/2884/2900`, grants `1x 35`;
- reliable retransmission of the same victory changes nothing and settles nothing;
- a restart from the persisted state replayed with the same victory settles nothing;
- the audit ledger settles all-or-nothing: short, duplicated-id or repeated
  transactions fail without partial consumption or a second grant;
- completion accounting: `completed` advances once per cycle and each cycle
  takes a fresh instance id.

The real-content run uses `--ignored` with
`OSFR_COOKING_REFERENCE=../lab/references/rust-cooking-isolated` and repeats the
cycles three times against the preserved recipe.

## Exact commands and results

Run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`.

```powershell
# RED before the module existed (exit 101, missing module)
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_012')

# GREEN: 2 passed, 1 ignored
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_012')

# Real preserved recipe: 1 passed
$env:OSFR_COOKING_REFERENCE='..\lab\references\rust-cooking-isolated'
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_012','--','--ignored')

# Sibling evidence that the integrated gameplay layer agrees: 13 passed, 1 ignored;
# private gameplay gate: 2 passed
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','cooking','--test','cooking_gameplay','--test','piece_012')
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','cooking_gameplay','--','--ignored')

# Inventory checker: RED before the script existed, then 8 tests OK
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_012.py" -v

# Real evidence build (no mismatches; exit 0)
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/piece_012.py --out '..\lab\reports\piece-012-20261009T025212Z' --stamp 20261009T025212Z
```

Evidence for this run is in `../lab/reports/piece-012-20261009T025212Z/`:
`red-rust.txt`, `red-python.txt`, `green-rust.txt`, `green-rust-private.txt`,
`green-python.txt`, `sibling-cooking-tests.txt`,
`sibling-cooking-gameplay-private.txt`, and the checker output
`inventory.json`, `findings.json`, `provenance.json`, `run.log`.

Checker summary: wiki revision 45318 agrees with the supplied content on all
four materials and quantities; five labelled gaps, no mismatches.

## Remaining gates

- Lead integration: register the module (or adopt the constants) and decide
  whether the audit runs in the ordinary suite.
- Native/gameplay: this piece does not start the client. The gateway-level
  once-only recipe settlement is already covered by the reviewed private
  gameplay tests; a fresh native whole-recipe run remains the item's native
  gate and stays with the lead.
- Content: the original recipe rows, per-recipe phase tuning and the
  historical output quantity/buff table remain unrecovered. Do not promote the
  reconstructed values to historical facts.
