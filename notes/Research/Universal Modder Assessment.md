# Universal Modder assessment

Reviewed October 7, 2026 for the faithful Free Realms revival. The installed skills are helpful now, especially for establishing a repeatable run of the supplied client and using its behavior to verify Rust replacements. The existing [[Free Realms Recovery]] and [[Design/Rust Revival Roadmap]] remain the implementation status records.

## Skill selection

| Installed skill | When to use it here | Concrete application |
| --- | --- | --- |
| `mod-any-game` | Now | Keep the recovery, implementation, actual-game verification, and evidence loop together; select one playable slice at a time. |
| `game-recon` | Now, then when inputs change | Corroborate architecture, dependencies, launch configuration, and profile locations. Treat filename detections as leads. |
| `reverse-engineering` | Now | Follow protocol strings and cross-references in the saved Ghidra database; use recovered managed serializers and actual packets to verify Rust behavior. |
| `game-automation` | Now | Make a disposable client run reproducible: fixed window size, local launch settings, game-window screenshots, logs, and exact process cleanup. |
| `game-research-websearch` | Now | Find existing format readers and protocol work before reversing each format from scratch; record versions and confidence. |
| `share-field-notes` | Now for research and private notes | Search the shared knowledge base and preserve confirmed facts and dead ends locally. Public contribution is a later, separate action. |
| `mashup-mods` | Its reimplementation pattern applies | Keep the existing client as a behavioral reference while replacing services in Rust. Its engine-rewrite examples become more relevant to a later Rust client. |
| `asset-pipeline` | Later, once a reader or conversion is needed | Verify original texture/model dimensions, axes, skeletons, materials, and alpha. Round-trip one representative asset before converting batches. |
| `fal-assets` | Later, if missing or new content needs original art | Generative assets are optional; recovered original content and working gameplay are the immediate priority. No generation service is needed for the next server slice. |
| `showcase-video` | After a repeatable playable scene exists | Record evidence of zone entry, movement, and persistence, then make a demo when useful. |
| `publish-mod` | When preparing a release | Package original code and tools, retain provenance, and inspect the deliverable before distribution. |

Installed skill sources are under `%USERPROFILE%\.codex\skills`. The native-engine playbook explicitly describes reimplementations that read the user's own files and use the original binary as a reference: [native playbook](https://github.com/rehan-remade/universal-modder/blob/baff1e5d01f63ae7cc81a049b6f4cf60e6c91dec/skills/mod-any-game/references/engines/native.md). This supports the Rust direction; it does not provide a Free Realms server or an automatic language conversion.

## Knowledge-base result

The official [knowledge index](https://github.com/rehan-remade/universal-modder/blob/baff1e5d01f63ae7cc81a049b6f4cf60e6c91dec/knowledge/index.json) contains no Free Realms or SOE entry in this revision. Relevant general guidance is available:

- [Oracles: how an agent knows a mod actually works](https://github.com/rehan-remade/universal-modder/blob/baff1e5d01f63ae7cc81a049b6f4cf60e6c91dec/knowledge/techniques/oracles-how-agents-know-a-mod-works.md): use byte comparisons and trace replay for protocol work, then verify the real client. Check that screenshots are current by observing a deliberate change and comparing captures. Stop repeating an unchanged failed approach after several attempts.
- [Driving real games safely](https://github.com/rehan-remade/universal-modder/blob/baff1e5d01f63ae7cc81a049b6f4cf60e6c91dec/knowledge/techniques/driving-real-games-safely.md): fixed client-area size, foreground-targeted input, DPI-aware coordinate mapping, and cleanup by the PIDs created for the run. These are practical requirements for a reliable Windows test harness.

For this project, a useful reference run should save the exact client/server hashes, copied configuration, elapsed timestamps, raw local datagrams with directions, server logs, and inspected screenshots. The first checkpoints are session negotiation, application login, completed zone initialization, movement observed by the server, and character restoration after restart. Comparing decoded messages accommodates variable session IDs and timing; raw-byte fixtures should identify which fields may vary.

## Scanner verification and a false positive

The parent session installed the CLI privately at `lab/tools/universal-modder-env/Scripts/um.exe`, pinned to Universal Modder revision `baff1e5d01f63ae7cc81a049b6f4cf60e6c91dec`. Use its explicit path; global PATH installation is not assumed. Its scan is recorded in `lab/manifests/universal-modder-scan.json`.

The scan corroborates native x86 executables and indexes all 1,762 client files. Its generic unknown-engine result does not identify Free Realms' engine. No save path was discovered; that result is not evidence that no profile can be written during startup.

**The reported `ASI loader` is a filename heuristic, not a confirmed mod loader.** Installed `um/scan.py` line 344 matches any `*.asi`; line 669 applies that pattern to the file index. The supplied matches are `Miles/mssmp3.asi`, `Miles/mssogg.asi`, and `Miles/mssvoice.asi`. Static strings in the first two identify Miles Sound System MPEG Layer 3 and Ogg Vorbis audio decoders. No matching proxy-DLL path such as `dinput8.dll`, `version.dll`, `winmm.dll`, `d3d9.dll`, or `dxgi.dll` was found in this inspection. Do not plan an ASI mod integration from this scan label.

## Windows automation prerequisites

Universal Modder's window capture uses ffmpeg with the `gfxcapture` filter, which must be verified in the selected ffmpeg build. Ordinary GDI screenshots may miss GPU-rendered frames. Capture the game window while windowed and not minimized; read the image back. A small scaled copy is sufficient for checking progress. DPI scaling can make input coordinates differ from physical screenshot pixels.

The CLI package needs Python 3.10+, Pillow, NumPy, and PyYAML. The parent installed it in an isolated environment. The Windows input and process-loopback audio helpers are bundled PowerShell/C# tools; audio recording and Blender are unnecessary for the immediate login/zone test. The installed [game-automation skill](%USERPROFILE%/.codex/skills/game-automation/SKILL.md) provides the command forms. Avoid changing client binaries merely to improve capture before an ordinary local baseline has been attempted.

## Community work worth checking before future asset readers

Research found two primary-source leads outside the Universal Modder knowledge base. Their documentation and visible open-issue lists were reviewed; neither has been executed or validated against these archives.

| Reference | What its authors report | Next local verification |
| --- | --- | --- |
| [ForgeLightToolkit](https://github.com/EDITzDev/ForgeLightToolkit/tree/9c06714c5d79d9152c322814bc904abad944e06e) | A Unity importer with Free Realms DMA, GCK2, and MAP support; ADR, DME, and GCNK support is marked partial. World loading is a work in progress. | Inspect the relevant reader and compare its headers, versions, offsets, and output with several exported files. Unity installation is unnecessary for reading the format code. |
| [FR ADR Toolkit format notes](https://github.com/edenfps/fr-adr-toolkit/blob/5acfdf8d6006fa6f43d1d3605f076a6be7393e72/ASSET_FORMATS.md) | Actor definitions connect DSK skeletons, DME meshes, DMA materials, GR2 animations, and DDS textures. The notes identify DSKE, DMOD, and DMAT signatures and describe sample-specific vertex layouts. | Verify signatures and material-declared vertex layouts across our files before implementing a general Rust reader. Preserve this AGPL-3.0 repository's provenance if reusing code. |

The second repository also contains scripts named `parse_soe_fishing.py` and `quick_pcap_look.py`. These are leads for later inspection, not verified protocol specifications. Its loose-file override and conversion claims also need an actual-client check. Its standalone `parse_assets.py` is sample-oriented: the default entry point expects Chatdy filenames and the skeleton inspection uses a hard-coded offset of 912. Prefer examining the library readers under `fr_formats` for general asset checks. No assets, templates, binaries, or sample models were downloaded for this review; only source and documentation were retained by the session.

The pinned Sanctuary checkout already supplies a more direct server reference: `src/Sanctuary.UdpLibrary`, gateway/login configuration, packet readers/writers, and application handlers. `Gateway/Program.cs` selects `CGAPI_527`; `Login/Program.cs` selects `LoginUdp_6`. Gateway compression framing and application encryption are separate layers in its implementation. The repository's `Sanctuary.UdpLibrary.Tests/BasicTest.cs` contains unbounded loops, so adapt bounded checks instead of launching that test unchanged. All compatibility claims still require the supplied client's behavior.

## Research record

Queries: Universal Modder index searches for `Free Realms`, `SOE`, `oracles`, and `reverse`; GitHub-focused searches for `"Free Realms" "DME"`, `"Free Realms" "pack" extractor`, and `"GZNE" "FreeRealms"`. Live revisions were checked with read-only `git ls-remote` and cited source documents were reopened at those revisions. The archive and runnable-tool checks below extend this initial review. No relevant GZNE-specific result was established. This review made no client, server, or debugger run and captured no game screenshots.

## Existing community extractor verified

The session retained source-only copies of [forgelight-toolbox](https://github.com/rhettvx/forgelight-toolbox/tree/a8d7fae0c81ed410b43ddba279d645b74ee905a1) and [DbgPack](https://github.com/brhumphe/dbg-pack/tree/8feac971cc21449b20f1cb58d0f1ff900fd6b849) under `lab/references`. `SOURCE.json` files record exact revisions and every downloaded source-file hash. DbgPack's sample/test game files were excluded. Its runtime uses the Python standard library, so no additional package installation was needed.

Read the existing `FLUtils/fl_pack.py`, `DbgPack/pack1.py`, `asset1.py`, and `asset_manager.py` before running them. DbgPack reads the linked big-endian directories and exposes asset metadata and bytes. Toolbox writes those bytes using asset filenames. Because its output uses those names directly and its asset dictionary can collapse duplicate names, the validation wrapper first audits all names and checks directory-entry count against dictionary size and our existing Rust catalog.

The **unmodified community extractor** successfully extracted the supplied `Assets_008.pack`: **6,074 entries and 45,765,884 asset bytes**. Every entry's name, offset, size, and stored CRC matched the Rust catalog. Every exported file's SHA-256 matched the Rust output, and every CRC-32 was checked again. This proves compatibility for this pack; the community extractor has not been run across all 24 packs in this check.

Evidence: `lab/manifests/community-pack-check.json`, `lab/extracted/community-pack-check/tool.log`, and the newly extracted files under `lab/extracted/community-pack-check/Assets_008`. The pack SHA-256 is `e12d749ef4b655ad7dec2a4e770fd75534c7ac287ac0f4b458a42ec65f5caafd`.

Reproduction from the project directory:

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\check-community-pack.py'
```

The wrapper implements no pack parser. It loads the existing reader, audits names, invokes the existing extractor, and compares results. It deliberately refuses existing output/report paths. Preserve the evidence from this run and select new output paths before repeating it.

## XeNTaX, ZenHAX, and QuickBMS investigation

The archives were searched before recommending additional format implementation:

- XeNTaX's [public post index](https://github.com/XeNTaXBackup/XeNTaXBackup.github.io/blob/2c6074a1bcb085e77f527fcf620620a4ee1e76da/PostIndex.md) identifies thread 3728, [Free Realms DME format](https://github.com/XeNTaXBackup/XeNTaXBackup.github.io/blob/2c6074a1bcb085e77f527fcf620620a4ee1e76da/markdown/Free%20Realms%20DME%20format_3728.md). Posts from 2009 discuss DMOD/DMAT, mesh offsets, vertex sizes, and indices. A 2011 author post reports improved DME loading in 3D Object Converter 4.80. These historical findings are useful cross-checks, not complete modern schemas. No linked attachment or sample archive was fetched.
- ZenHAX's [Clone Wars Adventures (.pack) thread 3621](https://www.zenhax.com/viewtopic.php@t=3621.html) contains aluigi's January 3, 2017 reference to `clone_wars_adventures.bms`, plus a printed variant. Its big-endian next-block, count, length-prefixed name, offset, size, and CRC structure agrees with the supplied pack reader. This is a neighboring game's documented algorithm; the actual DbgPack extraction above establishes compatibility with our input.
- The post index has no `GZNE` or `ForgeLight` title match. `FR Online` thread 9991 concerns another game with MPQ/IDX files and was excluded. No Free Realms DMA/GZNE-specific archived tool was established by the queries.

Additional exact searches: `site:zenhax.com "Free Realms"`; `site:xentax.com "Free Realms" "pack"`; `site:github.com/XeNTaXBackup "Free Realms"`; `"Free Realms" "QuickBMS"`; `"FreeRealms" "zenhax"`; `"GZNE" "Free Realms"`; `"DMA" "Free Realms" "Xentax"`; `"Free Realms" ".pack" "aluigi"`; `"ps2ls" github pack`. Broad web results often ignored the rare terms, so they were not treated as evidence.

Access outcomes: the web tool could not access `https://forum.xentax.com/` or a Wayback CDX request for `forum.xentax.com/*`; the rendered GitHub Pages thread was inaccessible, while its raw Markdown in the public backup worked. `https://aluigi.org/bms/clone_wars_adventures.bms` was inaccessible via the web tool. A direct request to the author's Altervista script path returned an archived-site page instead of a script, so no BMS file or executable was fetched. The first DbgPack `main` branch lookup returned 404; its actual default is `master`, revision `8feac971cc21449b20f1cb58d0f1ff900fd6b849`, and the source-only fetch then succeeded. The successful Python community tool was chosen for validation.

**Immediate application:** establish the real-client baseline and record its local protocol sequence, then use those observations to verify the next Rust transport and login slice. This gives the existing recovery work a repeatable test of whether the game actually accepts the rebuilt behavior.
