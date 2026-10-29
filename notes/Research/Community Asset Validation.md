# Community asset validation

The owner required community tools and archive research before additional custom format work. The earlier Rust pack implementation preceded that search; this session corrected the workflow and independently checked its output. The project instructions now require the search and a local compatibility check before implementing another reader.

## Packs and historical research

The unmodified ForgeLight Toolbox extractor with DbgPack exported all 6,074 entries of `Assets_008.pack` (45,765,884 bytes). Names, offsets, lengths, stored CRCs, exported SHA-256 values, and recomputed CRCs agree with the Rust output. This comparison covers one pack; the Rust extractor separately CRC-checked all 24 packs. Details, pinned revisions, archive queries, and access failures are in [[Universal Modder Assessment]].

XeNTaX's public backup contains [Free Realms DME format thread 3728](https://github.com/XeNTaXBackup/XeNTaXBackup.github.io/blob/2c6074a1bcb085e77f527fcf620620a4ee1e76da/markdown/Free%20Realms%20DME%20format_3728.md). ZenHAX's [Clone Wars Adventures pack thread](https://www.zenhax.com/viewtopic.php@t=3621.html) documents the matching linked big-endian archive structure and points to QuickBMS. The original BMS host was unavailable; the working community Python extractor supplied the local compatibility check.

## Actor and model readers actually run

Used [FR ADR Toolkit](https://github.com/edenfps/fr-adr-toolkit/tree/5acfdf8d6006fa6f43d1d3605f076a6be7393e72), revision `5acfdf8d6006fa6f43d1d3605f076a6be7393e72`, without writing a new binary format parser. Only source/documentation was fetched; all assets came from the owner's supplied client. Upstream license: AGPL-3.0.

`scripts/check-community-assets.py` loads the existing `AdrFile`, `DmaFile`, `DmeFile`, and `DskFile` readers, maps their references through our asset catalog, checks finite vertex positions and valid indices, and renders a static textured preview.

| Sample | Verified result |
| --- | --- |
| Chatdy actor | ADR connects `treeble.dsk`, `chatdy_lod0.dme`, and `chatdy.dma` |
| Chatdy mesh | DMOD v3; 4,256 vertices; 6,317 triangles; 52-byte vertex stride |
| Skeleton | 92 bones/bind poses; 32 animation slots identified |
| Material/texture | DMA references `CHATDY.DDS`; static preview rendered and visually inspected |
| Warpstone mesh | DMOD v3; two chunks; 998 vertices; 978 triangles |

Output: `lab/extracted/community-asset-check/report.json` (source and input hashes) and `chatdy-preview.png`. These derivatives remain outside Git. The preview uses a static pose and simple sampled texture colors; it is not a game screenshot or an animation check. No GZNE reader, general Rust model importer, loose-file override, or game-level asset replacement has been established.

Reproduction from the project directory with a fresh output path selected in the script:

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\check-community-assets.py'
```

The default completed output already exists and is deliberately protected against overwrite.
