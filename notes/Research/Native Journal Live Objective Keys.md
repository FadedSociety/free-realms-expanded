# Native journal live objective keys

The live farm journal kept blue pending markers after the authoritative server counters reached[1,1,1,1]. The original native client **rekeys objectives when copying a decoded quest into its runtime quest manager**. Incremental49/9 and49/10 must use the objective definition ID. The prior analysis stopped at the raw wire-reader hash key and incorrectly applied its ordinal key to live updates.

## Exact normalization chain

| Stage | Native proof |
| --- | --- |
| Temporary raw quest | `0092b050` reads tuple key, `00922810` stores that key at objective+0xf0, and `008fd770` reads the separate definition ID at+0x00. |
| Runtime quest creation | Full self `00940c60` and QuestAdd `00c7bd10` both insert through `009225d0` → `0091bb40` → `00baf490` → `00baf380`. |
| Runtime objectives | `00baf380` loops the raw ObjectiveData nodes and calls `00baba60(iVar1,iVar1)`: both arguments point to ObjectiveData itself, not its old+0xf0 key. |
| Runtime key assignment | `00baba60` → `00bab950`; that constructor writes runtime objective+0xf0 = *keyptr = **definition ID at raw ObjectiveData+0x00**. |
| Live update/completion | `00c7a990` and `00c7aa10` look up the supplied key against runtime+0xf0; an ordinal1 cannot find definition12001. The latter writes state2 only after a match. |
| Book datasource | `00cf7c30` builds rows from definition IDs and `00cf7b70` resolves those against runtime+0xf0. `00cf7fd0` reads current runtime state+d0. This explains why full self/restart displayed completed goals while ordinal-key updates silently missed them. |

Independent original-executable disassembly confirms the decisive call arguments:00baf458 `push edi`,00baf459 `push edi`,00baf45c `call00baba60`;00bab98e loads `[ecx]`, and00bab990 writes `[esi+0xf0]`. Thus the rekeying is not merely a pseudocode inference. The optional update/completion booleans control toast/chat behavior after mutation; they do not repair a failed objective lookup.

## Narrow repair and verification

`opening_journal.rs` now uses `goal.id` for incremental subtype9 and10. Full self and QuestAdd keep their existing tuple ordinals, which the client normalizes. No save, content, schema or rewards changed. The regression fixture explicitly distinguishes raw tuple1/definition12001 from live update+completion key12001, and tuple2/definition12002 from live key12002.

All5 focused journal tests pass, including the entire patched self payload compared with the original supplied Gateway.dll `ReadFromJSON` serializer output. That compiled oracle proves full-record bytes and the untouched surrounding self payload remain compatible; original native constructor/disassembly proves the incremental lookup semantics. The compiler oracle alone does not prove runtime journal acceptance.

Reproducible static extractor and proof: `lab/reports/trace-journal-incremental.py`, `journal-incremental-trace.json`, and `journal-incremental-disassembly.txt`. They hash the preserved executable, native pseudocode and Lua, keep original function line numbers, and assert the normalization chain. The prior reader-only conclusion in [[Native Opening Quest Journal]] and its historic proof is superseded by this runtime-constructor finding.

The151602 screenshot `farm-objectives-complete-journal_small.png` was independently viewed and has four blue pending markers. The finalized capture audit confirms exact49/9+10 ordinal keys1..4 despite authoritative counts[1,1,1,1]; see [[../Reviews/Native Farm Completion 151602]]. The independent154503 audit now accepts the repaired live path: QuestAdd includes Mae complete with tuple ordinals1..6, then Chef selection sends49/9+10 definition keys91000402/03, and the original journal visibly shows all three checks green in the same session without restart. Rainleaf/fruit live keys91000404/05 each count1→4 and complete at4; the authoritative stage ends[1,1,1,4,4,0]. The entire initial self and native QuestAdd still match the original compiled writer. All4780 raw datagram hashes and4759 gateway CRCs pass; see [[../Reviews/Native Cooking Preparation 154503]]. ID0 text labels remain a separate presentation failure in this release. This investigation sent no input, launched no game, and changed no current save or executable.
