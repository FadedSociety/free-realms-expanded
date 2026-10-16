# Free Realms recovery

Initial recovery and Rust foundation, October 7, 2026 (local time). The project goal is a playable Free Realms revival. **The game is not yet running on the Rust implementation.**

## Verified recovery

Both user-supplied archives are preserved and hash-verified. Their identities and input locations are in [[Target Intake]]. All paths below are relative to `%USERPROFILE%\Documents\Projects\CartoonMMO`.

| Area | Result | Evidence |
| --- | --- | --- |
| Managed server | Seven assemblies recovered to 769 C# files, including third-party dependencies | `lab/decompiled/server`; `lab/manifests/server-decompilation.json` |
| Asset catalog | 24 packs, 108,547 entries, 108,543 distinct case-insensitive names | `lab/manifests/rust-asset-catalog.tsv`; `pack-probe.json` |
| Asset export | Every entry exported and CRC-32 verified; four repeated logical names preserved | `lab/extracted/client-assets/catalog.tsv`; `lab/manifests/rust-extraction.log` |
| Raw asset volume | 3,020,467,857 bytes | `lab/manifests/asset-summary.json` |
| Rust implementation | Pack parser/extractor, SOE transport and local one-character/one-zone gateway; actual client walking and save/restart/restore | `project/prototype/osfr-tools`; [[Rust Gateway Milestone]] |
| Verification | 43 ordinary tests pass; three private-input oracles explicitly exercised across milestones; release build passes; seven live session checks | `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml`; local reports |
| Supplied server probe | Temporary copy starts and returns the same handshake bytes; process stopped after the check | `lab/manifests/legacy-handshake-probe.json` |
| Native client | 153,423 inferred function bodies exported; analysis database saved and reopened successfully | `lab/manifests/native-recovery.json`; `lab/decompiled/client/FreeRealms` |
| Community pack tool | Unmodified ForgeLight Toolbox/DbgPack extracts 6,074 entries; every exported byte matches Rust | `lab/manifests/community-pack-check.json` |
| Community actor tools | FR ADR Toolkit reads Chatdy and warpstone; static textured Chatdy preview inspected | `lab/extracted/community-asset-check/report.json`; [[Community Asset Validation]] |
| Actual client startup | Missing legacy D3DX DLL resolved with signed Microsoft component; matching local asset manifest served; actual window and local connection observed | `lab/manifests/latest-client-baseline.json`; [[Client Runtime Dependencies]] |

Recovered C# projects have not been rebuilt or verified for semantic equivalence. Native exports are inferred pseudocode, not original source. Native DLLs and the alternate FreeRealmsMac.exe have not been exhaustively analyzed.

The current playable result is [[Rust Gateway Milestone]]: actual-client login, reference-zone entry, walking, and saved-position restoration after restarting Rust. The sections below retain earlier recovery checks in their original sequence; their pending-gateway statements are superseded by that milestone.

### Native first-pass result

The initial Ghidra analysis reached its 900-second limit, so analysis is **partial**. The subsequent export completed in 661 seconds: 153,423 internal functions produced pseudocode, with zero attempted exports failing. The 95,176,801-byte `pseudocode.c`, `functions.tsv`, and summary are in `lab/decompiled/client/FreeRealms`. Their sizes and SHA-256 hashes are recorded in `lab/manifests/native-recovery.json`.

The saved project is `lab/decompiled/ghidra-projects/FreeRealms.gpr`. A separate read-only reopening confirmed 153,968 total function records: 153,423 internal and 545 external. External functions are not returned by the internal-function iterator, explaining the difference from the export count; their library implementations were not recovered in this pass. The function index row count matches the successful export count.

The analysis log also contains resource/disassembly and unreadable-address warnings. Successful pseudocode generation does not validate types, original names, control flow, or runtime behavior. Continue targeted analysis from the saved project as protocol and gameplay questions become specific.

## What the files contain

The main client is native x86 Windows code. The supplied OSFR server is managed .NET code. The seven managed assemblies are OSFRServer, Gateway, LibSOE, CommandLine, DotNetZip, log4net, and Newtonsoft.Json. The recovered source count therefore includes dependencies, not 769 files of original gameplay logic.

The asset corpus includes 55,400 DDS textures, 7,783 GR2 entries, 5,969 DME entries, 3,403 XML files, and 174 GZNE entries, among many other formats. These are raw exports: models, animations, UI, and world formats still need readers or conversion. Asset presence does not establish complete gameplay data or runtime compatibility.

A strict XML inspection found 2,474 ParticleEntity roots, 51 AreaDefinition roots, eight Definitions roots describing effects, and one Window root. Another 869 XML files fail strict parsing despite matching their archive CRC; samples include invalid tokens in otherwise recognizable particle XML. Preserve these original bytes and investigate the game's parser before normalizing them. All 174 GZNE entries have a `GZNE` header; the following little-endian version word is 2 in 146 files and 3 in 28 files. This identifies format variants, not working zone counts. Detailed results are in `lab/manifests/content-inspection.json`.

The pack reader follows big-endian directory blocks: absolute next-block offset and entry count, then length-prefixed UTF-8 names, asset offset, byte length, and CRC-32. Extraction uses numbered paths and a mapping catalog so duplicates remain intact and asset names cannot escape the export directory. Rust counts match an independent initial Python inventory. Every exported asset passed CRC verification.

The first Rust pack reader was written before the community archive search, which did not follow the intended community-first process. The subsequent XeNTaX/ZenHAX search and actual community extractor/model-reader runs are recorded in [[Universal Modder Assessment]] and [[Community Asset Validation]]. Future format work requires those searches and compatibility checks first.

## Community references

| Repository | Pinned revision | Role |
| --- | --- | --- |
| [OpenSourceFreeRealms](https://github.com/Open-Source-Free-Realms/OpenSourceFreeRealms) | `ace6a82e0ab07111be0fc8e899b5a0716724e137` | Archived walking-emulator reference; not a complete production server |
| [Sanctuary](https://github.com/Open-Source-Free-Realms/Sanctuary) | `4f028dcc274f628b65ef665fd59b184f5f3d3e3e` | Newer C# server emulator; broader protocol and system reference |

Both checkouts are under `lab/references`. Sanctuary contains login, character, zone, inventory, NPC, reward, and persistence code; it has not been built or tested in this workspace. Its source still contains explicit quest/achievement TODOs and incomplete paths, so no completeness claim follows from the available classes. The actual LICENSE is AGPL-3.0; preserve that provenance despite the README's MIT badge. The supplied binary server and current Git checkouts are distinct versions until equivalence is established.

## First Rust protocol slice

`soe-probe` implements only the initial `CGAPI_527` SOE session request/response. Synthetic fixtures were generated with the supplied LibSOE.dll's SOEWriter. The Rust response matches those bytes, echoes session and buffer fields, rejects malformed requests, and succeeds over a real loopback UDP socket.

Evidence locations in recovered code: `LibSOE/SOE.Core/SOEProtocol.cs` (HandleSessionRequest), `SOEClient.cs` (StartSession), and `LibSOE/SOE.Interfaces/SOEWriter.cs`. The probe binds only loopback, exits after one valid response, and has a ten-second idle timeout. It does not handle reliable data, application login, characters, zones, or gameplay. Synthetic fixtures are not evidence of a successful original-client connection.

An additional runtime check launched a temporary copy of the supplied OSFRServer.exe, changed only its copied configuration to an available test port, sent the synthetic request locally, and confirmed a byte-for-byte match with the fixture. The process was then stopped. Logs and copied inputs are in the run directory named in `legacy-handshake-probe.json`. This validates the supplied executable's initial transport response; no game client or game login was exercised.

## Transport and runtime continuation

The reusable Rust `soe` module now handles seeded CRC trailers of zero through four bytes, strict bounded zlib compression, channel-zero sequence rollover, reordered receive buffering, duplicate delivery suppression, and fragmented messages. Oversized future DATA is rejected before acknowledgement. A semantically invalid packet discovered after buffering poisons the receiver; callers must terminate that session. A persistent socket server, reliable sender/retransmission, multi-message application decoding, other channels, encryption, and gateway logic remain unimplemented.

Nine initial checks plus ten new transport checks pass. The new checks include 105 synthetic vectors generated by the supplied compiled serializer, corrupted/truncated input, expansion bounds, and 131,104 ordered deliveries across repeated sequence rollover. The explicit compiled .NET decoder oracle accepts five Rust-generated frames, including a 200,000-byte expanded payload. See `tests/fixtures/TRANSPORT.md` for provenance.

A bounded oracle captured the supplied server's login and ready responses using synthetic requests: 1,253 post-session received frames, 1,237 compressed, and 46 application messages totaling 627,566 bytes. Rust replay SHA-verified all 1,305 referenced files, decoded every frame, reassembled every message byte for byte, and rejected 1,253 CRC-mutated variants. Evidence: `lab/manifests/soe-rust-replay-20261008T0156090999173Z/summary.json`; [[Legacy Gateway Oracle]]. This verifies the supplied emulator's data, not original-client acceptance of a Rust server.

The actual supplied client was subsequently launched in disposable copies. Its log confirms version `1.910.1.530630`. Missing `d3dx9_31.dll` caused the first loader failure; official Microsoft imports/signatures and a local run verify that diagnosis. The next startup failed on the asset manifest. Following the community release's documented local hosting route with the supplied manifest's matching CRC/size let initialization complete. The client negotiated a **three-byte CRC trailer**, rather than the synthetic oracle's two-byte request. Initial traced traffic had missing fragments; an 8-MiB relay receive buffer and moving disk writes after forwarding resolved the stalled initialization. The corrected run reached World Ready, and a subsequent user-authorized short walking check changed the actual view and produced changed position updates in the supplied server log. Persistence, gameplay systems, and Rust gateway compatibility remain unverified. Detailed run outcomes are in [[Client Runtime Dependencies]] and [[Client Baseline]].

## What still needs recovery or reconstruction

- Replace captured gateway initialization with typed serialization while retaining the verified client behavior.
- Identify actual authoritative content definitions: NPCs, quests, jobs, rewards, combat, persistence, and activities.
- Separate complete definitions from presentation assets and canned emulator packets.
- Map native functions to meaningful systems, resolve decompiler failures, and analyze additional modules as required.
- Build Rust replacements in tested slices; use the existing client as the compatibility reference first.

The supplied archives do not contain verified original production-server source. Decompiling the client cannot recover logic that only ran on that server. See [[Design/Rust Revival Roadmap]] for the staged restoration plan and acceptance criteria.

Reproducible command forms are in [[Recovery Commands]].

[[SOE Transport Notes]] records the initial wire format and legacy CRC/sequence checks that need independent verification before porting.

## Verified Rust gateway continuation

The preceding sections preserve the earlier recovery sequence. Their statements that reliable sending, persistent sockets, Rust gateway entry, and character persistence remain unimplemented are superseded by [[Rust Gateway Milestone]]. The actual client now connects to Rust, enters the reference zone, walks with saved state updates, and restores its saved position after a fresh Rust process/client. Deliberately dropped reliable sequence 6 was resent byte-identically and acknowledged; 1,477 post-session CRCs independently validate. Seven live session checks also pass.

Initialization still uses 46 private captured emulator messages. No C# server process runs during these checks, but the original native client remains in use. The first gateway acceptance is recorded in `lab/manifests/rust-gateway-milestone.json`.

## Verified local gameplay continuation

[[Gameplay Milestone]] supersedes earlier statements that all inventory/reward/gameplay mutation is unimplemented. Rust now supports a reconstructed three-Robgoblin quest, authoritative local damage/defeat, atomic coin/item rewards, native equipment and an ordered checkpoint activity. Actual gameplay yielded 35 coins, increased Wheeler on the Cob from 9 to 10, equipped Small Stitched Pants and recorded an 18.456-second trial. A fresh server/client restored all gameplay exactly; the native inventory and equipment screens were inspected.

Community category packet44/2 fixed the previously empty Stuff UI; its 2,675 bytes match the unmodified community serializer. Inventory search then caused one native access violation in a separate run; category browsing is verified, search remains unresolved. Some obsolete icon URLs return 404. Original quest1617's Petty Yeti mapping, original abilities/AI, original minigames and broad content restoration remain outstanding. [[Gameplay Coverage]] distinguishes those gaps from reconstructed behavior. Aggregate evidence: `lab/manifests/rust-gameplay-milestone.json`.
