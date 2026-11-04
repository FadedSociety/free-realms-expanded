# Free Realms revival: start here

A Rust server for the original Free Realms client (the 2014 PC build): the gateway, login, accounts, gameplay and minigames, with a launcher (separate repo) that installs the client and the server for players.

- **[[Project Map]]**: where the code lives. It is generated from the code by `scripts/project_map.py`; rerun it after a merge.
- **`notes/Native Map/`**: the decoded client functions. Read these before tracing client behaviour by hand.
- **`notes/Research/` and `notes/Design/`**: evidence and design references written up to 2026-10-16. They are snapshots: check every claim against the code and fresh evidence before relying on it.
- **Cited notes:** the content data, tests and audit scripts cite some older pages as provenance: `notes/Build Guides/Content/` (NPCs, dungeons, jobs), a few `Build Guides/Gameplay` pages, and some `Autopilot` and `Reviews` notes. They stay while something cites them. They are snapshots too.
- **Releases** go out on the repository's Releases page and install through the launcher. A release is proven only by a clean launcher install with real play (client downloads from the community host; no lab-only files).

## How work is done
1. **Refresh first:** re-index the code graph, and read the current code and fresh evidence (the client's decoded functions, the fan wikis, gameplay video). Never plan from notes older than the code.
2. **Minigames and systems go top down,** from the launcher's assets to the loading screen's Go button, the Games-window rows, enter and quit with the UI intact, the match board, the arena and actor assets, the UI elements, and real input. Each step is proven natively.
3. **Every change** has tests, an independent review and the full test suite before it merges.
