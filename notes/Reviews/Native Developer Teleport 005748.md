# Native Developer Teleport - 005748

October 9, 2026. Bounded read-only audit of the live developer-teleport run `client-baseline-20261009T005748Z` (release `e1b9f8def10e10a53b7dada29df0547af330edd187f42a5f8e2545485f1954d0`, `--dev-tools`, branch `opening-developer-20261009T004212Z`). The audit froze the first 1702 frames into a private snapshot and decoded them with the shared analyzer; 28 automated checks pass (`../../../lab/reports/native-devteleport-20261009T010236Z/audit.json`, SHA-256 `89db5071…`). No shared run, save, source or UI was modified.

**Supplement:** the run was later finalized at 9524 frames. The closed-capture verification and the native Mae/stage-13 acceptance audit are appended at the end of this note (`../../../lab/reports/native-devteleport-005748-final-20261009T013314Z/`).

## What was verified

**Snapshot and decode.** The frozen live index (`4a327997…`, 1702 frames = 1685 gateway + 17 login) was copied into this report together with every referenced frame file; all frame lengths and SHA-256s match. The shared `lab/reports/analyze-native-created-character.py` (`3146515e…`) was run read-only against a private copy in `--live` mode: 1685 gateway frames, 1683 CRC-verified (two session frames skipped), crc_length 3, complete reassembly, no trailing partial or queued bytes. The run was still recording, so this is explicitly non-authoritative and covers nothing after frame 1701.

**The developer command and its real delivery.** Five chat commands reached the server as client applications (inner opcode 194, `ChatLog sendChatMessage`):

| Time (run seconds) | Command | Server reply observed on the wire |
| ---: | --- | --- |
| 94.484 | `realm goto Mae` | `Developer teleport: Mae Farnum (GUID 5715144128442925060) at 4.00m. Position applied; movement rules resume normally` |
| 202.563 | `realm goto definitely_missing` | `Unknown zone 0 entity: definitely_missing. Use realm goto list` |
| 222.563 | `realm gotomae` | `Guide: Mae Farnum at -2112.8, -56.5, 208.5` (the guide path, not a teleport) |
| 223.563 | `realm goto artifact` | `Ambiguous entity artifact: Gloam Artifact 5715144128442925077 … 5078 … 5079. Use the GUID` |
| 224.484 | `realm goto list` | `targets in zone 0: …` (27 zone-0 entities) |

The later mouse/chat batch was really delivered, but the earlier rapid multi-command batch and the text typed without a trailing Enter were **not** delivered: they produced no chat application on the wire. Input-helper `ok` alone would have hidden that; the wire shows only the five commands above, exactly one of which mentions `definitely_missing`.

**Packet, coordinates and persistence.** `application-0312.bin` (server to client, frame 001398, 94.516 s, 45 bytes) is the native `38/12` packet: inner opcode `0x26` = 38 with sub-opcode `0x0C` = 12, carrying Mae Farnum's exact actor position `(-2111.12060546875, -56.521751403808594, 212.12879943847656)`. It is the only sub-12 packet among the nine inner-38 packets in the prefix. The client replied with inner opcode 125: first a stale origin report `(-1999.9139, -42.6550, 445.8578)` at 94.531 s, then the height-adjusted arrival `(-2111.0854, -56.19644, 212.10196)` at 94.734 s. The persisted state is the second report: `teleport-after.json` (`dc558942…`) matches it, and `teleport-invalid-after.json` and `teleport-invalid-mouse-after.json` are byte-identical to it, so the unknown/ambiguous attempts moved nothing. Whole `gameplay` is identical before and after (259 coins, 30 items, Brawler 43, stages 1-12 rewarded, stage 13 still unaccepted, zone 0 unchanged), the computed distance to Mae is `4.004286676` m 3D (3.991 m planar), and `character-before.json` is byte-identical to the previous run's finalized `character-after.json` (`44e5f207…`), preserving the Queen-side origin.

**Bounded visual samples.** `teleport-mae.png` shows the character beside the Cooking Table with Mae Farnum visible and the red teleport-applied line; `teleport-invalid.png` shows the unchanged position; `chat-mouse.png` shows the chat window with the teleport line and the red `Unknown zone 0 entity: definitely_missing` response.

## What this proves and what it does not

Proves: with `--dev-tools` opted in, the ordinary chat command `realm goto Mae` was delivered and answered by a real server `38/12` position packet carrying Mae's exact coordinates; the client applied it with a native height adjustment and reported the result, which is what persisted; the unknown, ambiguous and `gotomae` variants were delivered but changed nothing (each answered explicitly); whole gameplay and the Queen origin were preserved. It also shows the difference between "the input helper reported ok" and "the client actually sent the command".

Does not prove: any final-manifest/CRC closure or video for this still-recording run, anything after frame 1701, any later native NPC-menu activity, historical fidelity of the dev-teleport feature, or acceptance of the local opening chain. The `38/12` packet is the server's position application for this opt-in tool; it is not part of the historical client/server protocol surface. This is a bounded acceptance of the observed teleport exchange, not a general claim.

Evidence: `../../../lab/reports/native-devteleport-20261009T010236Z/` (`audit.json` `89db5071…`, `receipt.md`, `audit-devteleport.py` `9d6756c3…`, frozen `snapshot/`, decode proof `be5a6319…`). Shared inputs: `character-before.json` `44e5f207…`, `teleport-before.json` `59c28c18…`, `teleport-after.json`/`invalid-after`/`invalid-mouse-after` `dc558942…`, `teleport-state-proof.json` `383bf0d9…`, `external-inputs.jsonl` (live, hash moved as the run continued). See [[../Current Progress]].

## Final supplement - closed capture and stage-13 acceptance

The run closed at 9524 frames (9507 gateway + 17 login) over 1952.25 s; client, login and server all stopped; the recording ended (`developer-opening-playthrough.mkv` 223983493 bytes plus raw audio). `frames/live-status.json` is `closed` with 9524 written, 0 pending, no error, and the live index equals the final manifest exactly; every one of the 9524 frame files was copied into the private final bundle and matches its recorded length and SHA-256. The shared analyzer run read-only against that copy gives 9505 CRC-verified gateway frames (two session frames skipped), crc_length 3 and complete reassembly. Final bundle: `../../../lab/reports/native-devteleport-005748-final-20261009T013314Z/` (`audit.json` `0cca31e6…`, 26 checks, 0 failures; `receipt.md`; decode proof `c4ad47e9…`; manifest `8547992059…`; manifest index `a87723c6…`; live status `589bdcd5…`).

Native Mae acceptance and the 4/4 harvest were ordinary-menu interactions, and the wire shows the exact packets:

- 421.73 s Mae menu open (client 26/19 + 26/8 to GUID 5715144128442925060), 444.73 s the 26/10 choice; 444.781 s quest add app 4401 (inner 49, sub 3) for quest 910013 with objectives 91001301/91001302/91001303, plus the once-only supplies app 4398 (Bat Meat 2862) and app 4399 (Spice 2900), each a 199-byte inner 38 sub 2 item add.
- Rainleaf node 5715144128442925068: ordinary menus through 493-526 s; stack add app 4406 (2884) at 525.219 s; quest counts 1-4 in apps 4407/4411/4420/4424 (inner 49 sub 9), stack updates 4410/4419/4423, objective completion app 4425 (sub 10).
- Gummyfruit node 5715144128442925069: four ordinary 26/10 choices at 757.22-766.33 s; stack add app 4430 (2875) at 757.281 s; quest counts 1-4 in apps 4431/4435/4439/4443, stack updates 4434/4438/4442, completion app 4444.
- Snapshots confirm the progression: before accept stage 13 `[0,0,0,0]` with 12 rewarded and no supplies; after accept `[1,0,0,0]` with 2862x4 + 2900x4 once; rainleaf final `[1,4,0,0]` with 2884x4; gathered `[1,4,4,0]` with 2875x4. **No recipe success**: the cooking objective stays 0, `cooking.completed` remains 1 with `active:null`, and no Stew is granted. The later cooking instances recorded in the final save (`next_instance_id` 46000005) belong to the separately owned crafting/Quit work and are not evaluated here.

Final identity: `character-after.json` is 30644 bytes `c653cf71b8ae5fb1b3e13d9fde40e960267c47024a36f2686bcaf7b4c0ef4af8` with stage 13 `[1,4,4,0]`, 12 stages rewarded and 259 coins. The standoff branch `opening-developer-standoff-20261009T0131Z` records the exact copy in `branch.json` (`161c5009…`: origin_run `client-baseline-20261009T005748Z`, created 01:30:55Z, `character_sha256` `c653cf71…`); the branch's live `character.json` has since been overwritten by the newer native run, so the record file and this bundle's copy are the preserved evidence. The source origin `44e5f207…` and the Queen checkpoint `3a9d3807…` are both unchanged.

This supplement accepts the closed capture and the observed Mae acceptance / 4 Rainleaf + 4 Gummyfruit ordinary-menu harvest chain only. It does not accept stage-13 completion or the recipe, does not evaluate the separately owned crafting/Quit economy details, and makes no walking/collision claim for the developer relocation branch.
