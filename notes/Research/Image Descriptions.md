# Image Descriptions

Visual descriptions of the client's pictures, so art can be found by what it shows rather than by file name (the owner's request, 2026-10-15).

- **Coverage:** 13,313 images.
  - 552 UI pictures. Names start with `ui`/`hud`/`loading`/`emblem`/`portrait`/`enc`/`minigame`/`notification`, or contain `_notification`/`loading`.
  - 12,761 distinct icons. Each icon appears once, at its largest size variant (`_32/_64/_128` are merged).
- **Not covered:** the ~50,000 world and model textures. Models are better documented by rendering them.
- **Pipeline:**
  1. `fl_pack.py` (with `lab/references/dbg-pack` on `PYTHONPATH`) unpacks all 22 packs to `lab/extracted/named/` (3.0 GB).
  2. `project/scripts/image_sheets.py` builds labelled contact sheets in `lab/reports/image-sheets/`: icons 4×4 at 192 px, UI 2×2 at 384 px. Each `*.txt` lists the cells.
  3. Ten Sonnet agents (2 at a time) described the 936 sheets, about 3M tokens in total. Output: `lab/reports/image-descriptions/<sheet>.jsonl`, merged into `index.jsonl`. Fields: number, file, size, description, kind, tags, text, sheet.
- **Search:** `python -I project/scripts/image_search.py <words...> [--kind KIND] [--masks]`. Example: `image_search.py sword crossed`. Mask/silhouette variants (4,745) are hidden unless `--masks` is given.
- **Kinds:** mask/silhouette 4,745 · clothing 2,689 · furniture 989 · item 982 · ui symbol 562 · armor 370 · weapon 348 · food 329 · resource 318 · ability 272 · pet 261 · emblem 198 · others.
- **Caveats:**
  - Many 32 px icons are tiny on the sheets, so their descriptions are coarse; descriptions say "file name suggests" when the name was used.
  - The UI set is mostly UI pieces (borders, buttons, minimap parts).
  - Encounter start-screen portraits (e.g. the Troll Summoner in a battle stance) were not found under these prefixes. They are probably image-set textures with other names; still to locate.
