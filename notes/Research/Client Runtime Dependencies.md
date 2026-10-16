# Client runtime dependencies

The first supplied-client baseline exited with `0xC0000135` before sending UDP, as reported by the baseline runner. Microsoft defines this value as `STATUS_DLL_NOT_FOUND`. The status identifies a loader dependency failure; it does not name the missing module. Parent analysis is collecting actual executable import and Windows event evidence. [Microsoft NTSTATUS reference](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/596a1078-e883-4972-9bbc-49e60bebca55).

## Community evidence first

The pinned OSFR [Launcher Dx9Helper.cs](https://github.com/Open-Source-Free-Realms/Launcher/blob/e6a2674c516fe6062698440a1f37163132b4abca/src/Launcher/Helpers/Dx9Helper.cs) explicitly checks `d3d9.dll` and **`d3dx9_31.dll`** in Windows `SysWOW64`. Its comments identify the game as 32-bit and distinguish that directory from the 64-bit `System32` used by the launcher. [Launcher Constants.cs](https://github.com/Open-Source-Free-Realms/Launcher/blob/e6a2674c516fe6062698440a1f37163132b4abca/src/Launcher/Helpers/Constants.cs) directs users to Microsoft's DirectX End-User Runtime download. Copies of both pinned source files, source URLs, and retrieval timestamps are preserved in `lab/references/Launcher-runtime-evidence.json`.

The archived walking-emulator README only instructs users to start its server and the supplied batch file. The newer launcher's prerequisite check supplies more specific client guidance. Sanctuary's Visual Studio/.NET development instructions describe building its server and should not be treated as a substitute for native game-client dependencies.

## Local read-only findings

`lab/manifests/client-runtime-presence.json` records the checked paths and versions:

| Component | Observed presence |
| --- | --- |
| `C:\Windows\SysWOW64\d3d9.dll` | Present, version 10.0.26100.8972 |
| `C:\Windows\SysWOW64\d3dx9_31.dll` | Absent |
| `C:\Windows\System32\d3dx9_31.dll` | Absent |
| Supplied client's root `d3dx9_31.dll` | Absent |
| Client `Microsoft.VC80.CRT\msvcr80.dll` | Present, version 8.00.50727.5592 |
| Client `Microsoft.VC80.CRT\Microsoft.VC80.CRT.manifest` | Present |

The archive also contains private `msvcp80.dll` and `msvcm80.dll`, plus CRT files for its TCG component. Presence of these private files does not by itself validate all side-by-side assembly bindings. The file inventory found no bundled DirectX runtime installer or D3DX DLL. Existing archive manifests remain the source of supplied-file identity; no supplied files were changed during this inspection.

**Initial inference:** missing x86 `d3dx9_31.dll` was the strongest candidate because the community launcher checks it and it was absent locally. A subsequent baseline advanced beyond the loader after an officially signed x86 copy was added to the copied client, as recorded by the parent baseline investigation. That runtime step exposed the separate asset-delivery failure below.

## Official runtime route

The community launcher links the [Microsoft DirectX End-User Runtime Web Installer](https://www.microsoft.com/en-us/download/details.aspx?id=35). Microsoft's [DirectX End-User Runtimes June 2010 offline package](https://www.microsoft.com/en-us/download/details.aspx?id=8109) includes that release and preceding legacy side-by-side components. Microsoft explains that these optional libraries are separate from the OS's modern core DirectX runtime. Thus an installed modern DirectX version does not establish D3DX availability.

If loader evidence confirms this dependency, use Microsoft's signed official runtime package and verify the resulting x86 component, then repeat the existing bounded local baseline. Keep installer provenance and hashes in the lab. This research step performed no runtime installation, DLL substitution, or client launch.

For further loader diagnosis, Microsoft's [C/C++ side-by-side troubleshooting guidance](https://learn.microsoft.com/en-us/cpp/build/troubleshooting-c-cpp-isolated-applications-and-side-by-side-assemblies) recommends examining dependencies and manifests. Prefer those diagnostics and the community prerequisite code before broader runtime installations.

## Next startup failure: asset manifest delivery

The copied-client run `lab/runs/client-baseline-20261008T015537Z` exited with code 27 after 51.235 seconds, without sending UDP or finding a game window. Its manifest records both processes stopped. The parent investigation found the corresponding `FreeRealms.log`: initialization reached asset delivery, four requests for the configured loopback `manifest.crc` failed, and the client shut down. This diagnoses this observed code-27 exit as a startup asset-delivery failure; there is no evidence here that it represents a minimum-hardware check or general runtime exception.

Community evidence explains the required manifest pair. The archived [OFR v1.0 release notes](https://github.com/Open-Source-Free-Realms/OpenSourceFreeRealms/releases/tag/v1.0) document two missing files for asset-server launch failures: host `manifest.crc` under the local web server's OSFR directory and put `Assets_manifest.txt` in the client directory. These are native client's asset-delivery metadata, separate from the modern launcher's XML updater manifest.

Parent analysis preserved those published metadata files under `lab/references/osfr-asset-manifest`. Its published CRC/size pair is `2676693903,9173508` and matches the published manifest using standard zlib CRC-32. The supplied archive's manifest is a different version: **9,479,368 bytes, CRC-32 123886846**. `lab/manifests/asset-manifest-identity.json` records an independent computation of that local identity. Preserve the original manifest/asset relationship instead of silently substituting the older published manifest.

The current pinned Sanctuary [development client runner](https://github.com/Open-Source-Free-Realms/Sanctuary/blob/4f028dcc274f628b65ef665fd59b184f5f3d3e3e/run_client.py) and [WebAPI launch configuration](https://github.com/Open-Source-Free-Realms/Sanctuary/blob/4f028dcc274f628b65ef665fd59b184f5f3d3e3e/src/Sanctuary.WebAPI/appsettings.json) both specify `AssetDelivery:IndirectServerAddress=http://osfr.editz.dev/assets`. The newer launcher forwards server-provided launch arguments. The supplied client instead names the older `osfr.editz.co.uk/assets` endpoint and has `PreDownloadAssets=1`.

A bounded metadata-only endpoint check found:

| Endpoint | Observed result |
| --- | --- |
| Current community HTTP and HTTPS `osfr.editz.dev/assets/manifest.crc` | HTTP 200, 18 bytes, `3387611285,9479368` |
| Older supplied `osfr.editz.co.uk/assets/manifest.crc` | DNS lookup failed |
| Community launcher `opensourcefreerealms.com/clientmanifest.xml` | HTTP 200, advertised length 154,354 bytes; only a bounded prefix was read |

Full probe metadata and the small CRC response are in `lab/manifests/asset-delivery-endpoints.json`. Even though the current asset endpoint advertises the same byte length as the supplied manifest, its CRC differs. Size alone does not establish identity.

The parent runner is testing a loopback HTTP service that supplies a matching CRC/size pair for the preserved archive manifest and records actual HTTP requests. Its outcome remains to be verified. No checked community source documents `PreDownloadAssets=0` as a way to bypass manifest initialization, so that setting should remain an unverified hypothesis rather than an assumed fix. This research step performed no client launches, config edits, installations or binary patches.
