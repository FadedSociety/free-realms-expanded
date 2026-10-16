# Target intake

Status: **preserved, inventoried, and initially recovered**. See [[Free Realms Recovery]] for results and gaps.

| Field | Observed value |
| --- | --- |
| Target | Free Realms, OSFR client and community server supplied by the owner |
| Server source | `%USERPROFILE%\Downloads\OSFR.Server.zip` |
| Client source | `%USERPROFILE%\Downloads\OSFR.Client.zip` |
| Client build | Exact executable build not established; no populated FileVersion/ProductVersion resource. Server config declares `1.910.1.530630`; main PE timestamp corresponds to March 2014 |
| Main executable | `OSFR Client\FreeRealms.exe`, 25,067,520 bytes |
| Client architecture | Native Windows PE32, x86; not .NET |
| Client components | Scaleform/GFx, Awesomium, Miles audio, Qt4 TCG components, Granny assets observed; exact engine revision unknown |
| Server runtime | Managed .NET; recovered OSFRServer project targets net48 |
| Asset containers | 24 legacy linked-index `.pack` files |
| Local gateway hint | Supplied launcher points to loopback port 20260; recovered protocol string `CGAPI_527` |
| Recovery route | ILSpy for managed server; Ghidra for native main executable; custom Rust pack reader |
| Reimplementation target | Rust |

## Archive identities

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| OSFR.Server.zip | 1,061,437 | `cb6806c12fc26a4a5f1cb28b9b695b2de52a6d33ae35a55b46c2e6bcd3888ff5` |
| OSFR.Client.zip | 1,815,328,743 | `61167cec14879ffe22669c60bc08bc49dad9dc34f075305b27b587e40956609c` |

Copies in `lab/originals` match the supplied archives. Safe extraction produced 13 server files (6,488,668 bytes) and 1,762 client files (3,312,376,439 bytes). File-level SHA-256 and ZIP inventories are in `lab/manifests/OSFR.Server-intake.json`, `OSFR.Client-intake.json`, and the corresponding `*-zip-inventory.json` files.

`lab/manifests/pe-inventory.json` records executable headers. Despite its filename, `FreeRealmsMac.exe` is also an x86 Windows PE in this archive. It has not received the main executable's Ghidra analysis pass.

The community server is not evidence that the original production server code or all server-side content survived. Archive completeness and game completeness are separate questions.
