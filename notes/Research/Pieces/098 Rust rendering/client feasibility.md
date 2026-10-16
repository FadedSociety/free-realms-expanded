# 098 Rust rendering/client feasibility

Work item 098. A small, real loading slice for the three asset families a replacement client
needs first: terrain (`GCNK`), skinned models (`DMOD` + embedded `DMAT`, with `DSKE` skeletons
and `ADR` actors) and animation bindings (`.gr2` clips). The slice is CPU-side only. It does
not render, does not touch the native client, and establishes nothing about native
compatibility or gameplay.

Deliverable: `prototype/osfr-tools/src/restoration/piece_098.rs` (loaders plus limits) with
`prototype/osfr-tools/tests/piece_098.rs` (10 ordinary tests, 1 private evidence gate),
`scripts/pieces/piece_098.py` (community-reader oracle) and
`tests/pieces/test_piece_098.py` (8 tests).

## What the slice loads

| Family | Container | Loaded now | Deliberately not loaded |
| --- | --- | --- | --- |
| Terrain | `GCNK` v2-6 | tiles, coords, runtime objects, render batches, height-map metadata, shared 32-byte vertices, u16 indices, collision blob length | collision semantics, height-map pixels, v2-v4 runtime objects |
| Model | `DMOD` v2-3 + `DMAT` v1 | texture names, material rows, bounds, chunks, 52-byte `ClrNrmUVSkin` and 36-byte `ClrNrmUV` vertices, indices | material parameters, GPU upload, skinning |
| Skeleton | `DSKE` v2 | bone names, 48-byte tail bind-pose rows, hierarchy blob size | hierarchy meaning |
| Actor | `ADR` sections | skeleton/mesh/material references, model scale, animation slot table (slot, clip, playback scalar) | sockets, foley, other section semantics |
| Animation | Granny `GR2` | container header: magic-derived pointer size, version, section table | transform curves, compressed sections, reflection |

Every loader refuses truncated input, unknown magic, out-of-range versions, non-finite
floats, out-of-range indices and counts beyond a bounded envelope. Terrain indices are
validated per render batch, because the recovered batches carry a base vertex offset.

## Recovered facts and where they come from

* `lab/extracted/client-assets/catalog.tsv` holds 108,547 entries: 2,344 `GCNK`, 5,969 `DMOD`,
  865 `DSKE`, 8,631 `ADR` and 7,783 `GR2`. A read-only scan
  (`../lab/reports/piece-098-20261009T051947Z/scan-corpus-versions.py`) shows `GCNK` 2-6,
  `DMOD` 2-3 and `DSKE` 2 only, which is what bounds the slice.
* All 7,783 `.gr2` payloads start with one of two 16-byte Granny magics
  (`29de6cc0baa4532b25f5b7a5f666e2ee` → 4-byte pointers, 6,559 files;
  `e59b495e6f631f141e13eba990beedc4` → 8-byte pointers, 1,224 files). Guessing a single
  4-byte magic would have rejected the 1,224 pointer-64 clips.
* The `ADR` animation bank is **not** a nested record stream. A structural walk over
  section 9 finds zero slots; the pinned reader matches the repeating
  `01 <len> <slot> 00 02 <len> <clip>.gr2 00 [04 04 <be f32>]` pattern instead. The Rust
  scanner reproduces that pattern and is cross-checked against the pinned reader's slot list.
* Terrain `GCNK` tile metadata beyond coordinates is read but not interpreted: the pinned
  reader's `Position` values for both sampled tiles are denormal/placeholder-looking
  (`[4.45e-41, 4e-45, -1000, 1e-45]`), so this note records the fields and makes no claim
  about their meaning. Terrain-height use stays with the existing artifact-floor work, which
  already validates heights against native observations.

## Community readers used

All three were run without source modification; raw/decompiled material stays in the lab.

| Reader | Pinned revision | Use here |
| --- | --- | --- |
| ForgeLightToolkit `GcnkFile` | `9c06714c5d79d9152c322814bc904abad944e06e` | terrain oracle through the existing private `lab/tools/cave-community-reader` harness |
| FR ADR Toolkit `AdrFile`/`DmeFile`/`DskFile` | `5acfdf8d6006fa6f43d1d3605f076a6be7393e72` | model, skeleton, actor and animation-slot oracle (AGPL-3.0) |
| carbon-granny `reader.py` | `732265cff0bbe4535dacb9297a1c0794a65ba267` | `GR2` container header layout and the two-magic table |

## Evidence

Private bundle: `../lab/reports/piece-098-20261009T051947Z/` (see its `README.md`).

```
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_098.py'
  -> 2 terrain files, 3 actors, 6 clips; oracle.json

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_098.py" -v
  -> Ran 8 tests, OK

& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_098.py' --verify <report>\oracle.json
  -> "mismatches": [] over 76 derived checks

scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_098
  -> 10 passed, 0 failed, 1 ignored

$env:OSFR_RENDER_SLICE = "<report>\oracle.json"
scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_098 -- --ignored
  -> 1 passed, 0 failed (rust-slice-check.json)

scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --lib
  -> 24 passed, 0 failed
```

The private gate loads the same preserved bytes the community readers did and asserts every
recorded field. Matched values include `FabledRealms_8_-32.gcnk` (v6, 16 tiles, 8,481
vertices, 47,394 indices, 285 placed objects, 520,032 collision bytes, 16 render batches) and
`chatdy` (v3 mesh, 4,256 vertices, 6,317 triangles, 92 bones, 32 animation slots,
`CHATDY.DDS`), `wolf_generic` (833 vertices, 1,106 triangles, 48 bones, 33 slots) and
`gloamling_m_default` (1,117 vertices, 1,480 triangles, 51 bones, 32 slots). Sampled clips
cover both Granny magic variants (4-byte pointer chatdy/wolf, 8-byte pointer gloamling) and
all six report version 7 with 8 sections.

The whole ordinary suite does not compile right now because other workers' targets
(`piece_044`, `piece_058`, `piece_068`, `piece_083`) are mid-flight; that failure is recorded
verbatim in the bundle and is unrelated to this item. Only piece 098's four assigned code/test
paths and its private report directory were written.

## Original, inferred and reconstructed boundary

* **Original/recovered**: the container layouts, magic values, version ranges, counts and the
  preserved bytes themselves, all re-derived with the pinned readers above.
* **Inferred**: the meaning of `GCNK` tile `Position`, the `DSKE` hierarchy blob, most `DMOD`
  chunk tags and `DMAT` parameters stay unnamed.
* **Reconstructed**: this loader slice, its validation policy, its limits and its error model.
  No historic engine behaviour is claimed from them.
* **Native acceptance**: none claimed. Nothing here runs `FreeRealms.exe`.

## Remaining integration and native gates

1. Granny transform-curve evaluation (community readers exist; not implemented here), then
   pose sampling and skeleton binding.
2. `GCNK` collision semantics; the slice only bounds-checks the decompressed blob.
3. `GCNK` v2-v4 runtime objects (88 files are v2-v4) and height-map pixels.
4. A renderer: GPU upload, materials and a real zone view — the "stage 6" acceptance in
   [[Design/Rust Revival Roadmap]].
5. Lead registration of `pub mod piece_098;` if this slice is adopted; the owned test uses
   `#[path]` until then.

## Skills applied

* `game-recon` for the actual format families and corpus version counts.
* `game-research-websearch` for community readers (the Granny magic table came from a pinned
  community repository, not from guessing).
* `reverse-engineering` discipline for separating recovered schema from inferred meaning.
* `ponytail` for reusing the pinned readers as the oracle instead of writing a second parser
  for every format, and for keeping the Rust slice to the smallest real loading path.
* `caveman` for terse reporting; persisted evidence stays complete.

No `game-automation` work was needed: this item owns no native window control.
