# Recovery workflow evidence review

2026-10-08. **Author self-review only: this is not an independent review and does not satisfy the independent-review gate.** It records what the author re-verified while writing [[Research/Verified Recovery Workflow]] so the lead can accept or re-derive it. Evidence bundle: `lab/reports/recovery-workflow-20261008T234338Z/` (private, untracked, `manifest.json` lists every file hash).

## Claims checked today

| Claim | Evidence type | Result | Limit |
| --- | --- | --- | --- |
| Preserved archive hashes | Full read, both files (`archive-identity.txt`) | Match intake manifests exactly | None observed |
| Toolchain paths/versions usable | Direct execution (`tool-versions.txt`) | dotnet 10.0.401; ilspycmd 11.1.0.9782; Temurin 21.0.12.1; cargo 1.99.0; um 0.2.0; Python 3.12.14 | Version drift possible over time |
| Ghidra headless launcher works | Usage output + version (`ghidra-headless-usage.txt`) | 12.1.4 PUBLIC, usage printed (exit 1 is expected with no arguments) | Analysis itself not re-run |
| Asset catalog consistency | Deterministic 24-row sample, stride 4522 (`catalog-sample-oracle.json`) | 24/24 rows: exported bytes size and CRC-32 match catalog; sampled offsets inside pack bounds | Sample, not the full 108,547-entry pass |
| Community readers still run | Import + read (`community-reader-smoke.json`) | DbgPack opens `Assets_008.pack` (6,074 assets); FR ADR Toolkit loads `chatdy.adr` (material/mesh/skeleton) | Read path only; no extraction/conversion |
| Rust test inventory | Listing only, timestamped (`rust-test-list.txt`, `rust-ignored-test-list.txt`) | 183 listed at 2026-10-08T23:46Z (154 ordinary + 29 ignored); independent review's fresh listing 2026-10-09T00:07Z: 208 (179 ordinary + 29 ignored) after concurrent `ordinary_quit` additions | Not a run of the full suite; re-list before quoting any count |
| Pack-reader unit behavior | 5 named tests (`rust-pack-reader-subset.txt`) | 5/5 pass (big-endian directory, CRC32 vector, bounds rejection, name safety, linked-block cycle) | Subset only |
| Wire-logic subset | 3 transport tests (`rust-pack-subset.txt`) | 3/3 pass (zlib expansion limits, selective-ACK holes, sequence rollover) | Subset only |
| Export artifacts present | File metadata | `pseudocode.c` 95,176,801 bytes; `functions.tsv` 5,231,774 bytes; 769 `.cs` files; matches manifests | Sizes checked, hashes taken from manifest |

## Commands run (read-only)

All runs used private toolchains under `lab/tools`; nothing was installed, downloaded, committed, launched, or overwritten. Exact commands and outputs are in the bundle files named above.

- `Get-FileHash -Algorithm SHA256` on both originals (0.08 s / 2.0 s).
- `dotnet --version`, `ilspycmd --version`, `java -version`, `cargo --version`, `um --version`, `python --version`.
- `analyzeHeadless.bat` with no arguments (usage; exit 1 by design).
- `cargo test -- --list` and `--list --ignored` through `scripts/cargo.ps1 -CargoArgs`.
- Two bounded `cargo test` filtered runs (5 pack-reader tests, 3 transport tests).
- `sample-oracle.py` and `community-reader-smoke.py` against `lab/extracted/client-assets` and `lab/references`.

## Not verified today

The full Rust ordinary suite (154/0 is the recorded milestone result, not re-run today), release build, ILSpy re-decompilation, Ghidra analysis/export, full pack extraction, native client launch, and any independent-review receipt for this note. Earlier manifests (`project/recovery-manifest.json`, repo root) still carry older snapshot fields (`unit_tests_passed: 72`, `ordinary_tests_passed: 73`) that later milestone notes supersede with 154 pass / 29 ignored; the manifest was left untouched because it is outside this task's ownership. `decompiled (reconstructed) C#` and `native pseudocode` remain inferred/unverified artifacts, and 46 private initialization templates are still required by the Rust gateway. The independent review of this workflow completed separately ([[Reviews/Recovery Workflow Independent Review]]); this note remains author-only and does not satisfy that gate.

## Corrections applied after independent review

All seven findings from [[Reviews/Recovery Workflow Independent Review]] were applied to both owned notes on 2026-10-09: decompiled-not-original C# wording, community-script output-path caveat plus exact full-path read-only smoke commands, `project/recovery-manifest.json` path, timestamped test counts (recorded 183/154+29 preserved beside the 208/179+29 recount), gate wording for evidence-only edits, and the 545-external citation nuance. Disposition table and document hashes: `lab/reports/recovery-workflow-corrections-20261009T000722Z/correction-receipt.md`.

## Suggested lead verification

Rerun the full ordinary suite once and capture the log; spot-check one ignored oracle; if the workflow note drives action, confirm the `Recovery Commands` harness forms still match `scripts/` before use. No claim above depends on an unstated private input.
