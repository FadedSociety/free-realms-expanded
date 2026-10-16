# Opening journal reconstructed labels

2026-10-08 UTC. Native run154503 accepted Cooking101 and visibly updated its first three objectives to green. The same screenshot shows two `<STRING 0 NOT FOUND>` rows for locally reconstructed Chef selection/equipment goals. The journal used literal locale ID0; this is a presentation defect, not missing quest progress. Both distinct ingredient objectives also displayed one original combined Rainleaf/Gummyfruit label. Source screenshot: `lab/runs/client-baseline-20261008T154503Z/cooking-live-journal-goals_small.png`.

The optional repair is prepared and independently reviewed. **Native rendering remains pending.** No release build, client input, active run change, campaign migration or save write was performed for this repair.

## Community tool and native path evidence

Used unchanged MIT [FreeRealmsLocaleTools](https://github.com/Udaya-X2/FreeRealmsLocaleTools), pinned `ba2ede225ff33edab8e8e0d20c45d4c5156309e5`. Its existing strict reader, `AddEntry` and paired `WriteEntries` handle the format; no custom locale parser/writer was added. `scripts/prepare-journal-text.py` builds a small private helper against that source and verifies the entire output through the strict reader again.

Independent original native function00e8e360 formats `%s%s_data.dir` and `%s%s_data.dat`; root00e861e0 defaults to `./locale/`. Locale8 is en_US. These main-game loose files are outside Assets_manifest and main-game packs. Two same-name TCG catalog entries are unrelated and excluded. The correct target is the disposable run's **client/locale**, using both files before any process starts. Proof: `lab/reports/native-locale-path-review.json` SHA256 `d7bfe9c269f31cbf0aa602781d404948b30e2293a14254ee55fd46702f68a0a2`.

## Exact private package

- Final package: `lab/references/opening-journal-text-v2`.
- NEW matching content directory: `lab/references/opening-journal-text-v2/content`.
- Original source directory remains `lab/references/rust-opening-cooking-original-assets`. All29 original content files are byte-identical in the clone; only `journal-text.json` is added.
- Opening SHA256 remains `3e6f4228efc76460b8b4ff5c9ecefb36264e2caf308da7d0cc7fc3ea8e34229e`. **No save migration is required.**
- Mapping SHA256: `a643bbb0b127bfa98fe75fa23e828791e9b22be0754f742cb0f74b5f4c6af573`.
- Package manifest SHA256: `7de0bb30130c314ae179b6726bcee52a7874347edadf4aec1c079874978b0af5`.
- Output dat SHA256: `72a9d6ee2dc0363d0fa6f086641a160897a07bdb111e754c79ab94985b82b289`, 6930220 bytes.
- Output dir SHA256: `c5a0db940c5d3edd48eb8b45cac9c38cc0ef6486d8161a8b67d0b68782a6541a`, 2799262 bytes.

The first v1 preparation package is preserved as history. It lacks the final stock-export binding required by the preflight; it must not be launched.

The writer preserves all113286 stock entries and all85932 original numeric IDs, adding28 new entries/IDs without collisions within the community tool's proven1..5103267 domain. All new IDs are between619 and1236. The exact original pair remains unchanged:

- Original dat SHA256 `9c764d76b05138e607f23276a3a03f364b22c44d2787e31bb96be1b4fe4fc097`.
- Original dir SHA256 `a3835d4b69aab9308fabe188c1d72f211038b11f9efdbbddbab688212cd8c458`.

All51 goals have readable nonzero text after presentation mapping.24 objectives receive explicit reconstructed labels: all14 zero-ID class/equipment goals, four separate ingredient goals, three numbered farm fires and three numbered Gloam artifacts. Identical labels reuse the same new ID; different strings never share an ID. Existing original objective labels elsewhere remain unchanged. These additions are local restoration text, not claimed original quest text. Required counts continue to use the native current/required formatter.

## Narrow runtime boundary

`OpeningJournal` optionally reads `journal-text.json` from the content directory, binds it to the exact opening bytes, validates each quest/objective/source-ID tuple and requires coverage of every zero-ID objective. It substitutes only the two text IDs in full-self/Add quest records. OpeningEngine definitions, state, rewards, counters, trigger bindings, discovery progress and incremental runtime keys remain unchanged. Live49/9+10 still use objective definition IDs; full keyed-list ordinals remain ordinal. Missing presentation files preserve the original serializer behavior byte-for-byte.

`scripts/run-local-baseline.py` accepts opt-in `--journal-text <package>`. Its helper validates schema, Locale8, exact content/mapping/stock-export/proof hashes, noncolliding added IDs and both fixed locale targets before creating a run. Mapped content without the matching opt-in, and an overlay without mapped content, fail before launch. Both copied original targets and output hashes are checked before either write; changing a started run is rejected. The run result records the package/mapping/opening and paired-file provenance. The asset manifest is unchanged and no altered locale bytes are served under an original asset CRC.

For the existing native command, replace its `--content` argument and append these options on a **NEW** run, after stopping the existing run normally:

```powershell
--content '../lab/references/opening-journal-text-v2/content' `
--journal-text '../lab/references/opening-journal-text-v2' `
--character-select-title '../lab/references/character-select-title-v3'
```

The last option is separate and optional: the independently reviewed original character-selection movie title repair. Its original renderer used `_id` for title; JPEXS changes the single operand to `_name`. All56 script exports and full XML were compared:55 scripts identical, one renderer operand changed, and exactly one original action byte differs. The private movie's different hash is explicitly recorded; original assets/cache are preserved. This feature also applies only before launch in a new disposable run. Native acceptance is pending. See the immutable v3 package proof and `lab/reports/character-select-title-single-byte-proof.json`.

## Verification and independent review

- Ordinary journal checks cover full/Add label substitution, exact unchanged surrounding bytes/counters/state, unchanged live updates, wrong opening/locale/source binding, duplicate mappings, conflicting strings, unbounded IDs and incomplete zero-ID coverage.
- Existing absent-feature compiled original Gateway.dll serializer oracle still matches the entire full-self payload.
- Private all51-goal check passes: all17 records map exactly, all counters/IDs/ordinals/flags stay unchanged, and a synthetic stage4 checkpoint with completed prior stages and counts[1,1,1,0,0,0] reloads identically from the source and clone.
- The complete17-stage packet/restart flow passes against the mapped content. This is not a native playthrough.
- Offline copy preflight passes both paired outputs and rejects23 unsafe transformations, including missing counterpart/opt-in, wrong mapping/locale/source/output/proof, stock collision, active/outside run and unrelated GFX overrides. Independent reviewer separately rejected25 malformed cases.
- Independent unchanged community reader confirms all original entries/IDs preserved, exact28 additions, metadata MD5/count and all mapped text values.
- Private proofs: `journal-text-overlay-preflight-v2.json`, `journal-locale-independent-review-v2.json`, `journal-text-content-independent-review-v2.json`, and `native-locale-path-review.json` under `lab/reports`.

Next native acceptance: restart from the unchanged Cooking checkpoint with the paired opt-in, inspect distinct ingredient/class/equipment rows, advance a counter, reopen the journal and reconnect. Confirm character selection shows the saved character name when its separate title option is enabled. Do not claim either UI repair accepted until those actual-client checks pass.
