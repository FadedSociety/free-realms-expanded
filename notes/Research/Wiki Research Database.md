# Wiki Research Database

Status: collected, indexed and verified, 2026-10-08 06:41 UTC.

The owner requested a dedicated Scrapling crawler and SQLite database with sqlite-vec for restoration research. The database and raw community text stay outside Git at `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\references\freerealms-wiki\wiki.sqlite`. They are research evidence, not authoritative original server definitions.

## Acquisition

The supplied [Special:AllPages](https://freerealms.fandom.com/wiki/Special:AllPages) HTML, robots.txt and Special:Export were unavailable to plain requests (Cloudflare 403). The public [MediaWiki API](https://freerealms.fandom.com/api.php?action=query&format=json&meta=siteinfo) responds with JSON HTTP 200. The crawler uses actual Scrapling `Fetcher.get` with a descriptive research User-Agent, browser impersonation disabled, stealth headers disabled, no authentication, no challenge solving and no proxy rotation. Supported `allpages` and revision queries replace the HTML index. Single-worker requests normally wait two seconds; a small, bounded quest-priority pass was run while the broad crawl proceeded.

Main namespace 0 contains 3,198 pages at enumeration. Quest namespace 112 contains only 52 existing pages. All 3,250 enumerated pages were preserved, including 341 redirects and one empty page. Many Chapter 1 quest links in character articles are redlinks, not recovered quest pages. The main article count is therefore not a count of complete quest definitions. User/talk/forum/file/template namespaces are excluded from the default collection. Empty articles, redirects, missing priority seeds and modern edits remain distinguishable. The 24 missing seed spellings include bare names whose actual article has a `Quest:` prefix; they do not imply 24 missing game concepts.

Every successful response is preserved under `raw`, deduplicated by SHA-256, with exact request URL, HTTP status and UTC retrieval time in `requests`. Each page retains its canonical URL, page ID, revision ID/time, raw wikitext path/hash, clean text and redirect target. Source revision metadata permits checking the community article history. Community content is CC-BY-SA unless otherwise noted; this private research archive retains attribution links. No game assets are downloaded by this crawler.

## Search

Private Python environment: `..\lab\tools\wiki-env`. Source: `scripts\wiki-research.py`; pinned direct dependencies: `scripts\wiki-requirements.txt`. Tables: `pages`, `titles`, `requests`, `chunks`, FTS5 `chunks_fts` and real sqlite-vec `chunk_vectors`. All 3,996 chunks have both keyword and vector indexes. Search returns text, canonical source URL, exact revision URL/time, retrieval time and source SHA-256 for each hit. Hybrid search combines keyword and semantic ranks using reciprocal rank fusion. Search/status/verify open SQLite read-only; verified search leaves the database SHA-256 unchanged. Wikitext cleaning preserves quest/infobox template parameter values rather than silently removing those fields. Scrapling also parses HTML fragments during cleaning.

Dense semantic embeddings run locally through FastEmbed/ONNX. Model: `BAAI/bge-small-en-v1.5`, 384 dimensions, cosine distance; MIT model repository `Qdrant/bge-small-en-v1.5-onnx-Q`, pinned revision `aa8f8b060edb00e03bfdd08813a2949946c8ba55`. `model-manifest.json` records every downloaded model file's SHA-256. This is a semantic model, not a lexical hash-vector substitute. Package versions are captured in `coverage.json` and the installation freeze. No API key or paid embedding service is required.

ONNX originally failed because Windows lacked `MSVCP140_1.dll`, confirmed from PE import tables. Microsoft's official signed x64 Visual C++ redistributable was installed quietly, exit 0, from the documented Microsoft download. Installer/log remain in the lab. ONNX runtime is pinned to 1.19.2; current versions are recorded in the report. The earlier fallback experiment adding the private JDK runtime path alone did not supply the missing DLL.

Run these commands from the project directory:

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\wiki-research.py' crawl --limit 10000 --delay 2
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\wiki-research.py' download-model
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\wiki-research.py' embed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\wiki-research.py' search 'Farnum Hooligan Wolves' --mode hybrid --limit 5
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\wiki-research.py' status
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\wiki-research.py' verify
```

The crawler resumes pending titles and namespace continuation, skips already preserved page content and commits each batch. Embedding resumes only missing vector rows. Re-run embedding after crawling finishes if it began during collection. Existing changed revisions are preserved by stopping rather than overwriting; refreshing to a different snapshot requires deliberate revision handling.

## Evidence useful to the opening playthrough

- [Your First Weapon](https://freerealms.fandom.com/wiki/Quest:Your_First_Weapon), revision 27815 (2013-07-12): previous Learn to Mine & Smelt Ore, next Fighting off the Pack; Cleetus Farnum; forge a Funky Scrapper Hammer. Dialogue inconsistently calls it Spunky Scrapper Hammer. Materials listed: 10 Copper Bars, five Turquoise, five Wooden Planks and five Thorny Fasteners; reward 16 coins. Preserve the inconsistency until client data resolves it.
- [Fighting off the Pack](https://freerealms.fandom.com/wiki/Quest:Fighting_off_the_Pack), revision 28955 (2013-07-21): switch to Brawler, knock out five Hooligan Wolves, return to Cleetus Farnum; reward 18 coins; next Eviction Notice.
- [Dungeons](https://freerealms.fandom.com/wiki/Dungeons), revision 49285 (2025-09-28): difficulty-one list includes Bixie Hive, Cracked Claw Caverns, Cursed Graveyard!, Hot Springs Haven, Sheep Watch and Tavern Cellar; Hewey's Escape and King of the Ring! are marked removed. Combat collections occur inside. This list does not identify the opening quest cave.

Full recovered quest text and revision links are in `quest-pages.json`. Findings were sent directly to the starting-area and dungeon audit agents. Missing Chapter 1 pages require ZAM/archive/client evidence; absence here does not establish absence from the original game.

## Review and acceptance

Completed populated-database verification:

- SQLite `integrity_check=ok`, no foreign-key errors, sqlite-vec v0.1.9 successfully loaded into SQLite 3.53.1.
- All 174 recorded API requests returned HTTP 200; every raw response and every preserved wikitext SHA-256 matches its record.
- All 3,996 chunk hashes pass; FTS and vectors cover exactly the same chunk IDs with no orphan index entries. All vectors have 384 finite values and unit norm within tolerance.
- Four sampled vectors recomputed with the pinned local model match the stored index within the verification tolerance.
- Five search checks pass: wolves by keyword, first hammer by semantic question, character creation by semantic question, Cracked Claw by keyword, and Farnum/wolves by hybrid search. Each returns the expected article first, with revision citations.
- A resumed crawl issues no new HTTP requests or duplicate content when all enumerated titles are complete. Embedding has zero pending chunks.
- Final database: 16,171,008 bytes, SHA-256 `b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`.

The first derived wikitext exports expanded LF to CRLF on Windows. Verification caught this before acceptance. The exact original text bytes were restored from already hash-verified API JSON, and the writer now uses bytes; `wikitext-newline-correction.json` records all 2,909 corrections. Raw API responses were unchanged.

Evidence: `lab/manifests/freerealms-wiki-research.json`, `freerealms-wiki/verification.json`, `search-review.json`, `coverage.json`, `model-manifest.json` and `lab/reports/wiki-*.log`. This is a self-review plus reproducible data/search checks, not an independent gameplay review. Modern community edits and missing original quest articles require corroboration against the supplied client or archival sources before implementation claims.

Cross-links: [[Starting Area Content Audit]], [[Starting Dungeon Audit]], [[../Design/Opening Playthrough]].

## Bounded crafting follow-up, 08:30 UTC

An actual plain Scrapling public-API revision query for `Smelting`, `Forging`, `Quest:Learn to Mine & Smelt Ore` and the alternate `and` title returns HTTP200 with all four explicitly `missing:true`. These are absent community articles, not omitted indexed pages. `lab/reports/opening-crafting-wiki-probe.json` preserves the exact URL, UTC retrieval time, raw-response SHA/path and response. The existing verified SQLite archive remains byte-identical. The follow-up does not recover copper output quantity, success rewards or quest-supplied ingredients; Miner revision46068's five copper ore plus three Singing Coal remains the known recipe-input evidence. Findings were sent to the crafting review agent.
