# IDA Cross-check

**W109 status: PARTLY PROVEN.** The checker now supports in-memory function creation, full accepted-label batching, grouped rates, and a ranked review queue. The full client pass and the real-IDA `where.exe` tests did not run: the prescribed `uv` child exits with status 1 during IDALib initialization and prints no diagnostic. No full-run directory or client `.id0` sidecar was created. Root cause is unconfirmed.

## Method

`scripts/native_map/ida_check.py` reads accepted `confirmed` and `likely` labels plus function metadata from the native-map SQLite database. IDA supplies function boundaries, resolved direct-call xrefs, C-string data references, and Hex-Rays pseudocode. When a label address is code with no containing IDA function, the checker calls `ida_funcs.add_func` in memory and records `ida.function_created_on_demand`. The client path closes with `close_database(False)`; the map is read-only.

Run `--all-accepted --batch-size 250 --run-name full-<UTC stamp>` to inspect every accepted confirmed and likely label. It writes `labels.jsonl`, `summary.json`, `review-queue.jsonl`, and an address-only `review-queue.txt` under `lab/native-map/run/ida-check/full-<UTC stamp>/`. The queue ranks confirmed before likely, then high before medium severity. It includes an existing-function boundary mismatch, a caller **and** callee graph disjoint while both sources have edges, or required strings missing from an exact IDA function. `review-queue.txt` has one address per line for `recheck.py make`; no reviewer or live-map update was run.

Subsystem prefix means the part before the first dot. Boundary and edge differences are machine-generated review signals, not label verdicts. Missing string references can also reflect indirect or compiler-generated access.

Queue severity is triage policy: boundary and paired caller/callee graph disagreements are high; missing strings alone are medium. Confidence comes from the accepted map record, and confirmed entries sort before likely entries.

## Available baseline: lead's 300-label run

Source: `lab/native-map/run/ida-check/lead-20261016.summary.json` and `.jsonl`. This sample predates W109's on-demand function change and is **not** the full accepted-label pass.

| Group | Labels | Exact boundary rate | Caller map-edge coverage | Callee map-edge coverage | Required strings present |
|---|---:|---:|---:|---:|---:|
| All | 300 | 214/300 (71.33%) | 77.19% | 70.37% | 45/102 (44.12%) |
| Confirmed | 200 | 136/200 (68.00%) | 77.22% | 65.07% | 37/87 (42.53%) |
| Likely | 100 | 78/100 (78.00%) | 76.71% | 83.33% | 8/15 (53.33%) |

| Subsystem prefix | Labels | Exact boundary rate | Caller map-edge coverage | Callee map-edge coverage | Required strings present |
|---|---:|---:|---:|---:|---:|
| asset | 1 | 100.00% | n/a | 100.00% | n/a |
| crt | 97 | 88.66% | 96.89% | 91.73% | 80.00% |
| engine | 118 | 47.46% | 43.86% | 49.14% | 14.29% |
| game | 5 | 100.00% | 100.00% | 100.00% | n/a |
| lib | 45 | 84.44% | 70.55% | 75.73% | 58.33% |
| net | 12 | 100.00% | 100.00% | 100.00% | 50.00% |
| platform | 15 | 73.33% | 42.86% | 49.18% | 43.75% |
| render | 1 | 100.00% | n/a | n/a | n/a |
| ui | 1 | 100.00% | 100.00% | 100.00% | n/a |
| unknown | 4 | 50.00% | 90.91% | 0.00% | n/a |
| world | 1 | 100.00% | 100.00% | n/a | n/a |

The old run found 216 IDA functions, 214 exact starts, and 214 decompilations. Its 84 missing-function addresses were separately reviewed by the lead as code not wrapped in functions, including vtable-only destructors such as `004024e0` and `00403ad1`. The old `function_boundary_mismatch` total therefore is not the W109 review-queue count.

## Baseline queue indicators

The updated queue predicate applied to the lead's old 300-label JSONL yields 16 entries. These are sample-only indicators; W109's full top 20 is unavailable.

- `00405d3d` confirmed medium — `GApplication::InitializeControlManager`: required string reference missing.
- `004094c7` confirmed medium — `SaveLayoutTheme`: required string references missing.
- `00411f68` confirmed medium — `GCtrlTheme_Infinity::Load`: required string reference missing.
- `00413fc3` confirmed medium — `LoadPlacement`: required string references missing.
- `00414098` confirmed medium — `StorePlacement`: required string reference missing.
- `0041f653` confirmed medium — `_Grow`: required string references missing.
- `00420ff1` confirmed medium — `std::basic_string::assign_substr`: required string reference missing.
- `00422034` confirmed medium — `std::basic_string::basic_string`: required string reference missing.
- `004224b1` confirmed medium — `std::basic_string::assign_cstr`: required string references missing.
- `00440428` confirmed medium — `InitMouseDeviceEntry`: required string reference missing.
- `004432c0` confirmed medium — `InitKeyboardDeviceEntry`: required string reference missing.
- `00445f20` confirmed medium — `DeviceKindRecordAndName`: required string references missing.
- `0044c4d0` likely high — `CxxThrowExceptionNullArgsB`: IDA function starts at `0044c39c`.
- `00598ebf` likely high — `ThrowVectorInsertExceptionPath`: IDA function starts at `00598d93`.
- `0053f306` likely medium — `RegisterGObjectDesc`: required string reference missing.
- `00f820c0` likely medium — `ParseFundWalletPreviewChildren`: required string reference missing.

## Heuristic false positives

W97's two `packet_writer_without_labeled_serializer_callee` flags were substring matches, not serializer evidence: `00422756` is `std::length_error::length_error` (`overwrites` contains `write`; `requested` contains `request`), and `0054d407` is allocator `StampAlloc_0054d407` (its description says it writes a magic pair). The revised heuristic reads label and callee **names** only; tests preserve both cases and a positive packet/serializer-name case.

## W109 run blocker

The full command was attempted with `--binary ../lab/working/ida/FreeRealms.exe --all-accepted --batch-size 250`. `uv run --project %USERPROFILE%/.claude/plugins/cache/mrexodia/ida-pro-mcp/0.1.0 python -I scripts/native_map/ida_check.py ...` exited 1 without an IDA diagnostic. The same IDALib initialization exit stopped the real-IDA test after its two pure path/sampling tests. Pure tests, including a fake-API in-memory creation check, pass; the required `where.exe` add-function check and full 68,411-label pass remain unverified.
