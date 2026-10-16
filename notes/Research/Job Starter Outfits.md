---
type: research
updated: 2026-10-15
task: W82
verdict: PARTLY PROVEN
---

# Job Starter Outfits

The six existing starter lists reproduce exactly. Fourteen jobs have a usable **reconstructed (R)** starter list from pinned Legacy data. Three jobs remain unresolved. The supplied profile equipment rows are **equipped inventory instances, not native default outfits**. No native-default selection rule was recovered, so none of the proposed starter selections is presented as an original retail rule.

## Outputs and reproduction

- `project/scripts/extract_job_outfits.py`: stdlib only; run `python -I scripts/extract_job_outfits.py` from `project`.
- `lab/reports/job-outfits-20261015/starter_items.json`: `{"starter_items": {"job_id": [definition_id, ...]}}`, matching the field shape in `profiles.json`. This is a proposed field replacement, not a complete profiles package.
- `lab/reports/job-outfits-20261015/evidence.json`: hashes of all source inputs, exact six-list comparisons, every equipped instance join, JSON pointers, source lines, packet offsets, every starter definition's source line/slot/class/gender, missing-catalogue flags, and native labels/pseudocode/bytes. Raw native evidence stays in the lab.
- `lab/reports/job-outfits-20261015/validation.txt` and `content-check.json`: executed gate transcript and scoped content-check report.

The script rebuilds both private JSON files from fixed lab inputs. `--self-check` runs synthetic validation without writing files. No Rust, content, saves, launcher, native client or shared planning files were changed. The lead owns integration and Plan/Ledger updates.

## Mapping rule, proved from bytes and supplied self data

Read the equipment span as `u32 count`, then **count triples** `(u32 slot, u32 item_instance_id, u32 category)`. A zero instance means an empty slot. For every nonzero instance, join `ClientItems[].Guid`; its `Definition` is the item definition ID. Never use the instance number as a definition ID, and never use the category field as an item definition.

Native evidence is for `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`:

| Address | Evidence |
|---|---|
| `008fc960` | Counted equipment reader: disassembly `008fc99b` reads count; `008fc9ca` reads the row key; `008fc9da` inserts it; `008fc9e2` calls `008dbaf0` for the remaining pair. Map label is speculative, so the complete 148-byte body was disassembled before relying on it. |
| `008dbaf0` | Two bounded dword reads: first destination `+0`, second `+4`. Entire 70-byte body inspected. This establishes 12 bytes per equipment row with the preceding key. |
| `008becc0` | Item lookup: `008becd8` selects the bucket, `008bece3` compares the hash, `008becf3` compares **node+0x10** with the requested key. Entire 84-byte body inspected because the map label is unknown. |
| `0103e5e0` | Original format string at `0187e960` explicitly names **Guid, Definition, TintOverride**. `0103e623` loads Definition from `+4`, `0103e626` loads Guid from `+0x10`, then pushes both for formatting. Confirmed map label, independently checked disassembly. |

The supplied recovered serializer corroborates the wire shape: `lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs:1389-1395` writes equipment count, slot, ItemGUID and Category; lines `1472-1478` write inventory count, Definition, Tint and Guid. This is provenance, not source copied into the project.

`lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json` (SHA-256 `e471ab7fd98532328fada3e759bdcb073f2baf626a4650f7334a540718358777`) supplies the join. Existing `prepare-opening-base.py::profile_bytes` and `prepare-gameplay-inventory.py::{item_state,item_bytes,unique_offset}` prove all 16 serialized profiles and the complete 1,386-item inventory against `lab/references/rust-gateway-reference/0007-login.bin` (203,850 bytes; SHA-256 `f4b8b4caeaf0beb52e5d36cee96fefdf5a6249dd70abf68bc999bc57a5f1bec3`). This proves compatibility with the supplied fixture. It does not establish that the fixture is a fresh retail character.

The original job IDs are independently checked against `lab/working/OSFR.Client/OSFR Client/Resources/ItemFilterOptions.txt`, lines 7-23. Slot names and slot 7's weapon flag come from `EquipmentSlotDefinitions.txt`, lines 2-8. No new resource format reader was needed.

## Why applying this join cannot reproduce the six defaults

The six existing lists originate in `lab/references/FreeRealms-Legacy/src/Resources/Profiles.json`, `DefaultItems`. `project/scripts/prepare-opening-base.py:152` selected those six lists directly. They were not inferred from the captured equipment spans. The Legacy `ClientProfileData.cs:25,28-44` declares DefaultItems but does not serialize it in reference profile data. Its presence in that JSON is not native-client proof of the default policy.

Below, `slot:instance=definition (line)` cites the **Guid line** in the supplied `PacketSendSelfToClient.json`; the Definition is in that same inventory object. Full JSON pointers and exact equipment-row packet offsets are in `evidence.json` for each ID.

| Job | Verified equipped joins |
|---|---|
| Medic 11 | 1:4273=76123 (67649); 2:4274=76122 (67665); 3:4276=76120 (67697); 4:4277=76124 (67713); 5:4275=76121 (67681); 6:4278=76125 (67729); 7:4279=75089 (67745); 8:1701=75325 (56465); 12:1536=75354 (54977) |
| Miner 14 | 1:1908=76147 (58049); 2:1993=76146 (58337); 3:1906=76144 (58017); 4:1905=76148 (58001); 5:1907=76145 (58033); 7:1213=1434 (52465); 8:3981=75691 (65985); 12:1997=75701 (58401); 13:1686=55343 (56401) |
| Blacksmith 16 | 1:36=21496 (49937); 2:35=21324 (49921); 3:31=21012 (49857); 4:34=21683 (49905); 5:32=21149 (49873); 7:1280=1433 (53009); 13:1686=55343 (56401) |
| Archer 35 | 2:1078=75734 (51729); 3:1269=75780 (52945); 4:1239=75778 (52641); 5:1344=75781 (53265); 6:1270=75785 (52961); 7:3713=4287 (65185); 8:1382=75184 (53489); 12:1381=75228 (53473) |
| Brawler 43 | 1:1663=75886 (56065); 2:2608=75897 (60497); 3:2609=75895 (60513); 4:2607=75899 (60481); 5:2610=75896 (60529); 7:4235=13658 (67345); 8:1561=75256 (55217); 12:2414=75292 (59793) |
| Chef 45 | 1:2740=75951 (61089); 3:2742=75949 (61121); 4:2739=75952 (61073); 5:2741=75950 (61105); 7:2777=75601 (61345); 8:2736=75655 (61025); 12:2942=75665 (61889); 13:2748=75679 (61201) |
| Kart Driver 48 | **1:2747=33324 (61185); 3:443=13496 (50913); 4:64=15017 (50305); 5:65=14105 (50321)** |

All six mapped equipment lists differ from their starter lists. For example, Blacksmith's five clothing definitions match, but its equipped weapon is 1433, not starter 6459, and it has an extra rune. Brawler 2608 is instance 2608 of definition **75897**, not starter definition 7536. Kart's equipped helmet and shirt differ from Legacy's proposed starters. There is no arithmetic conversion from those instance numbers to starter definitions.

## Proposed starters for all 17 jobs

Every nonempty list below is **(R)**: the *selection* comes from Legacy, not a recovered retail default rule. The six existing lists retain their exact order. The same policy is proposed for the other eight nonempty Legacy lists. No inventory snapshot gear was silently substituted for starter gear.

Sources are pinned to FreeRealms-Legacy commit `9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0), data use only. `Profiles.json` SHA-256: `334feb47ca36f4f429fc1646e6d75e776de24175ecf1ac880f217accecba8ea0`. `ClientItemDefinitions.json` SHA-256: `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`.

In the table, **definition@line** refers to that definition's `Id` line in `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`. The separate DefaultItems line is in its sibling `Profiles.json`. This gives file and line provenance for every proposed ID. The JSON evidence also records each item's slot, class, gender, model, texture and source record index.

| Job | ID | Definition IDs @ source lines, in output order | DefaultItems line | Confidence |
|---|---:|---|---:|---|
| Adventurer | 1 | `[]` | 127 | Unresolved fixed outfit; creation clothing is a separate path |
| Ninja | 2 | 19522@349877, 19431@347884, 19197@500417, 19787@355614, 19253@344020, **75091@1206973** | 242 | R |
| Postman | 4 | 13137@245387, 12827@437870, 13197@246659, 13317@249260, 12957@241673 | 352 | R |
| Medic | 11 | 18783@336332, 18695@334403, 510@10955, 18882@338478, 18482@329726, **4622@86451** | 469 | R |
| Wizard | 12 | 12178@225933, 12060@436534, 12709@238171, 12416@231423, 12574@235073, 12252@227649, **4914@92999** | 579 | R |
| Miner | 14 | 21665@383741, 21060@508902, 21784@386208, 21219@374418, **6561@121372** | 694 | R |
| Blacksmith | 16 | 21496@380248, 21324@376616, 21012@507908, 21683@384123, 21149@372931, **6459@119172** | 807 | R |
| Warrior | 32 | 46644@959792, 46389@954148, 46137@643963, 46732@961739, 46224@950497, 47019@968087, **29963@617951** | 921 | R |
| Archer | 35 | 1238@25078, 11530@215062, 353@7542, 11709@218886, 354@7565, 11828@221479, **4266@78718** | 1032 | R |
| Brawler | 43 | 7676@144820, 7536@141706, 7495@420842, 7939@150544, 7571@142484, 7781@147140, **75030@1205631** | 1159 | R |
| Chef | 45 | 14794@272858, 13597@442877, 14826@273512, 13821@252673 | 1274 | R |
| Kart Driver | 48 | 14599@268851, 13419@439223, 15017@277532, 14105@258634 | 1362 | R |
| Demo Derby Driver | 49 | 14481@266447, 616@13300, 623@13450, 13998@256379 | 1450 | R |
| Soccer Star | 52 | 61907@1141116, 61353@694358, 62169@1146638, 61609@1134827, 62473@1153043 | 1550 | R |
| Card Duelist | 120 | `[]` | 1675 | Unresolved; Legacy has no defaults, snapshot is personal equipment |
| Pet Trainer | 121 | `[]` | absent | Unresolved; absent from supplied self profiles and Legacy profiles |
| Fisherman | 137 | 68430@1164487, 68360@695410, 68465@1165260, 68395@1163715 | 1766 | R |

Bold IDs are slot 7 weapons/tools. Fisherman's snapshot includes a rod, but Legacy's default list does not; no rod was invented or appended. Empty lists mean **no fixed list recovered**, not proof of a historically empty outfit. Never integrate these three empty lists as a policy to strip clothing.

Gender variants remain in the list to reproduce the existing data shape: Postman 13197 is male legs, 13317 female legs; Wizard 12709/12416 are male/female legs, 12574/12252 male/female feet. The caller must respect gender eligibility when equipping. This task does not change equip routing.

### Assets corroborate models, not default selection

All 77 starter definitions have referenced model ADRs in the original extracted catalogue. The extractor expands only `<race>` and `<gender>`, verifies **264 unique ADRs** by catalogue length and CRC-32, and records SHA-256, pack, entry, and extracted path. This is file-identity evidence, not visual/native acceptance or proof of historical item-to-job defaults.

The three requested starter weapons have direct original model evidence in `lab/extracted/client-assets/catalog.tsv`:

| Job | Proposed definition (R) | Original model | Pack / entry / CRC-32 |
|---|---:|---|---|
| Ninja 2 | 75091 | `sword_ar_ag_weapon_trainingsword.adr` | `Assets_000.pack`, 5796, `2c86d886`; extracted `pack_12/005796.adr` |
| Warrior 32 | 29963 | `club_ar_ag_weapon_cudgel.adr` | `Assets_002.pack`, 1332, `59506255`; extracted `pack_14/001332.adr` |
| Wizard 12 | 4914 | `wand_ar_ag_weapon_crude.adr` | `Assets_007.pack`, 1432, `b0687f80`; extracted `pack_19/001432.adr` |

Ninja 75091 already exists in the frozen server catalogue. Its missing starter grant is not a missing-definition problem. Warrior 29963 and Wizard 4914 require catalogue coverage.

## W84 handoff: seven missing definitions

The frozen `round69-native-20261015T224835Z/content/inventory.json` has **1,703 rows but only 1,649 distinct IDs**. The 54 duplicate rows are byte-for-byte equal as decoded JSON records. This resolves the two different counts in the Ledger. Seven of the 77 proposed starter definitions are absent:

| Job | Missing definition | Legacy description | Slot / gender |
|---|---:|---|---|
| Ninja 2 | 19431 | Student Ninja Armwraps | 2 / any |
| Postman 4 | 13317 | Student Postman Short Shorts | 4 / female |
| Wizard 12 | 12416 | Student Wizard Skirt | 4 / female |
| Wizard 12 | 12252 | Student Wizard Shoes | 5 / female |
| Wizard 12 | 4914 | Student Wizard Wand | 7 / any |
| Warrior 32 | 29963 | Student Warrior Cudgel | 7 / any |
| Kart Driver 48 | 13419 | Student Kart Driver Jersey | 3 / any |

No catalogue rows were generated or installed by W82. All other proposed starter IDs are present in that frozen catalogue; this says nothing about unrelated content references or later packages.

## Checks, security review and remaining gates

Commands executed from `project`:

```text
python -I scripts/extract_job_outfits.py --self-check
SELF-CHECK PASS: 5 malformed spans rejected; changed starter order rejected (canary)

python -I scripts/extract_job_outfits.py
PASS 11 Medic: [18783, 18695, 510, 18882, 18482, 4622] (Legacy exact; snapshot differs)
PASS 14 Miner: [21665, 21060, 21784, 21219, 6561] (Legacy exact; snapshot differs)
PASS 16 Blacksmith: [21496, 21324, 21012, 21683, 21149, 6459] (Legacy exact; snapshot differs)
PASS 35 Archer: [1238, 11530, 353, 11709, 354, 11828, 4266] (Legacy exact; snapshot differs)
PASS 43 Brawler: [7676, 7536, 7495, 7939, 7571, 7781, 75030] (Legacy exact; snapshot differs)
PASS 45 Chef: [14794, 13597, 14826, 13821] (Legacy exact; snapshot differs)
JOIN PASS: 16 profiles; all nonzero instance IDs resolved
OUTPUT: 17 jobs; 14 reconstructed lists; 3 unresolved; 77 starter definitions
MISSING from 1703-row/1649-distinct catalogue: [4914, 12252, 12416, 13317, 13419, 19431, 29963]
REAL CANARY PASS: changed Brawler starter rejected
SECURITY: fixed paths, read-only inputs; no open findings in scoped review
```

Native investigation commands: `python -I scripts/native_map/nmq.py show 008fc960`, `dis 008fc960 420` (the first 148 bytes are the relevant function), `show 008becc0`, `dis 008becc0 84`, `show 0103e5e0`, `dis 0103e5e0 170` (first 97 bytes relevant), `dis 008dbaf0 103` (first 70 bytes relevant), plus `show`/`dis 00921460` and read-only SQLite label queries. The generator saves exact function bytes and native record provenance in the private evidence file.

The first extraction attempt correctly stopped on a catalogue-count assertion: 1,703 rows were incorrectly assumed to mean 1,703 unique IDs. The assertion was corrected to require 1,703 rows/1,649 distinct IDs and reject conflicting duplicates; reruns pass.

`python -I scripts/content_check.py` returned exit 0: **0 errors, 17,276 warnings across 134 files**. That initial invocation also refreshed the checker's ignored default `project/prototype/osfr-tools/studio/report.json`, an incidental generated-file write outside the assigned output paths. It was not hand-edited or reverted over other workers' work. The saved scoped run uses `python -I scripts/content_check.py --out ../lab/reports/job-outfits-20261015/content-check.json`. The warnings include 16,551 unresolved runtime item references; W82 did not change those content files.

The final extractor was run twice consecutively: both JSON outputs were byte-identical. All commands exited 0; the exact printed output and SHA-256 comparison are retained in `validation.txt`. An AST/import and credential-pattern scan passed before the manual security review. `git diff --check` on the owned source and note found no whitespace errors; both files are untracked additions awaiting lead review, not committed changes.

Security scope: offline extractor reading fixed local research inputs. No network, credentials, saves, roles or remote player input. Applied `secure-code-guardian` while building and a scoped `security-reviewer` pass before finishing. Controls: read-only SQLite; no shell; fixed output filenames with resolved junction/symlink checks; counted span bounds and hashes; asset paths confined to extraction root; duplicate-ID conflict checks; no player names/chat in outputs. Review found no open vulnerabilities. Dynamic imports use only the two existing fixed repository helper scripts; bytecode writes are disabled. Inputs are local trusted research files, not a general upload service.

Remaining gates:

- **Not proved:** a native retail default-outfit table or the historical correctness of Legacy DefaultItems. Captured equipped gear cannot prove it, as the six counterexamples demonstrate.
- **Unresolved:** fixed starters for Adventurer, Card Duelist and Pet Trainer. Do not borrow personal snapshot equipment as a historical default.
- **Lead/W84:** provide the seven missing definitions; review gender-specific equip behavior; register the chosen lists in a new private content package. Native outfit/weapon/grant acceptance was not run and is outside worker authorization.
- Native self-data run `011719` confirms Kart Driver's missing equipped rows in the current save and separate creation clothing at character select. That symptom does not establish retail starter IDs.

Ponytail: reused both existing serializers, the frozen self-data fixture, the asset catalogue, and nmq. Added one stdlib extractor; no new serializer, dependency, asset decoder, runtime rule or client automation.
